//! The fast path: alpha-beta over the transition table, with make and unmake
//! on a dense archive.
//!
//! The inner loop runs no rule. It reads `superko_rules::table::RuleTable`,
//! whose only constructor calls the transliteration of `Defs.lean`, exactly as
//! `superko_graph::enumerate` does, and it archives and un-archives one key per
//! move rather than cloning a history.
//!
//! # What alpha-beta does and does not change
//!
//! Alpha-beta returns the minimax value of the tree whatever order it visits
//! moves in, so [`solve`] is order-independent in its value and
//! order-dependent in its node count. The order is the one
//! `superko_rules::reference::all_moves` fixes — the pass, then each point
//! row-major — which is the order `superko_graph::enumerate` counts in and the
//! order `tests/rules_walk.rs` pins.
//!
//! The window is the whole score range, `-(m · n) - 1` to `m · n + 1`, so the
//! value returned is exact rather than a bound. Scores lie in
//! `-(m · n) ..= m · n` because area scoring counts every point for at most
//! one color.
//!
//! # No memoization
//!
//! Two states with the same position and different archives do not have the
//! same continuations. A transposition table keyed on the position is the
//! graph-history-interaction error and would silently return a wrong value;
//! one keyed on the whole archive is about the size of the tree it prunes.
//! There is therefore no table here at all.
//!
//! # The node budget
//!
//! A search may be given a budget. Exceeding it abandons the search and
//! returns no value: a partial alpha-beta result is a bound whose direction
//! depends on where the search stopped, and reporting one as a value is how a
//! sweep comes to claim it resolved a root it did not. [`Solution::value`] is
//! `None` in that case, and the sweep counts the root as unresolved.

use superko_rules::archive::{Archive, ArchiveKey};
use superko_rules::code::{PosCode, encode};
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::{Color, Move, Position, all_moves};
use superko_rules::table::{RuleTable, TooLarge};

/// What a score search found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Solution {
    /// The minimax area difference, or `None` when the node budget stopped the
    /// search before it had one.
    pub value: Option<i32>,
    /// States visited, the root included.
    pub nodes: u64,
    /// The greatest number of moves from the root the search reached.
    pub max_depth: usize,
    /// Plays the search took that positional superko would have refused —
    /// always zero under positional superko, and the measure of whether the
    /// two rules met at all in this tree.
    pub ssk_only: u64,
}

/// What a verdict search found.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Decision {
    /// Whether the named color has a winning strategy, or `None` when the node
    /// budget stopped the search.
    pub wins: Option<bool>,
    /// States visited, the root included.
    pub nodes: u64,
    /// The greatest number of moves from the root the search reached.
    pub max_depth: usize,
}

/// One search over one board under one repetition rule.
///
/// Reusable across roots: a completed or abandoned search leaves the archive
/// as it found it, because every make is paired with an unmake, so a sweep
/// builds one of these and calls it once per root.
#[derive(Clone, Debug)]
pub struct Solver<'t> {
    table: &'t RuleTable,
    rep: Repetition,
    archive: Archive,
    moves: Vec<Move>,
    span: i32,
    code: PosCode,
    to_move: Color,
    passes: u32,
    depth: usize,
    budget: Option<u64>,
    nodes: u64,
    max_depth: usize,
    ssk_only: u64,
    over_budget: bool,
}

/// What undoes a move.
#[derive(Clone, Copy, Debug)]
struct Undo {
    key: ArchiveKey,
    was_new: bool,
    code: PosCode,
    to_move: Color,
    passes: u32,
}

impl<'t> Solver<'t> {
    /// A solver over a board's transition table under a repetition rule.
    ///
    /// # Panics
    ///
    /// Panics when the board is too large for a position code.
    #[must_use]
    pub fn new(table: &'t RuleTable, rep: Repetition) -> Self {
        let dims = table.dims();
        Self {
            table,
            rep,
            archive: Archive::new(dims),
            moves: all_moves(dims),
            span: i32::try_from(dims.point_count()).expect("point count fits an i32"),
            code: PosCode(0),
            to_move: Color::Black,
            passes: 0,
            depth: 0,
            budget: None,
            nodes: 0,
            max_depth: 0,
            ssk_only: 0,
            over_budget: false,
        }
    }

    /// Stop a search that visits more than this many nodes, and report it
    /// unresolved rather than returning a partial answer.
    #[must_use]
    pub const fn with_budget(mut self, budget: Option<u64>) -> Self {
        self.budget = budget;
        self
    }

    /// Visit moves in the reverse of `all_moves` order — each point in reverse
    /// row-major order, then the pass.
    ///
    /// This exists for the tests. Alpha-beta returns the minimax value of the
    /// tree whatever order it visits moves in, and its cutoffs depend on that
    /// order entirely, so a second order agreeing on the value and disagreeing
    /// on the node count is evidence that the cutoffs preserve the answer —
    /// evidence available on boards the naive arbiter of [`crate::naive`]
    /// cannot reach.
    #[must_use]
    pub fn with_reversed_moves(mut self) -> Self {
        self.moves.reverse();
        self
    }

    /// The board this solver is over.
    #[must_use]
    pub const fn dims(&self) -> Dims {
        self.table.dims()
    }

    /// The archive key of a situation under the rule in force.
    ///
    /// Positional superko reads a position's two bits together, so the archive
    /// is indexed by position and the color is fixed — the projection
    /// `superko_rules::divergence::Divergence::PskArchiveProjection` registers.
    fn key(&self, code: PosCode, to_move: Color) -> ArchiveKey {
        match self.rep {
            Repetition::Ssk => ArchiveKey::new(code, to_move),
            Repetition::Psk => ArchiveKey::new(code, Color::Black),
        }
    }

    /// The successor of a move the rules permit, or `None` when it is refused.
    ///
    /// `RuleTable::playable` is the whole of `Superko.PlayableAt` — the point
    /// being empty included — so the repetition read is the only other
    /// conjunct.
    ///
    /// Under situational superko this also counts the plays **positional
    /// superko would have refused**: the same archive answers both reads, so
    /// the count costs one bit test. A search that never met such a play has
    /// searched a tree the two rules agree on, and a report of no separation
    /// from it would say nothing at all, and [`Solution::ssk_only`] is what
    /// makes that distinguishable.
    fn legal_succ(&mut self, mv: Move) -> Option<PosCode> {
        match mv {
            Move::Pass => Some(self.code),
            Move::Play(p) => {
                if !self.table.playable(self.code, self.to_move, p) {
                    return None;
                }
                let succ = self.table.succ(self.code, self.to_move, p);
                if self
                    .archive
                    .contains(self.key(succ, self.to_move.other()), self.rep)
                {
                    return None;
                }
                if matches!(self.rep, Repetition::Ssk) && self.archive.contains_psk(succ) {
                    self.ssk_only += 1;
                }
                Some(succ)
            }
        }
    }

    /// Take a move whose legality has been decided.
    fn make(&mut self, mv: Move, succ: PosCode) -> Undo {
        let key = self.key(succ, self.to_move.other());
        let undo = Undo {
            key,
            was_new: self.archive.insert(key),
            code: self.code,
            to_move: self.to_move,
            passes: self.passes,
        };
        self.code = succ;
        self.to_move = self.to_move.other();
        self.passes = match mv {
            Move::Pass => self.passes.checked_add(1).expect("pass count overflowed"),
            Move::Play(_) => 0,
        };
        self.depth += 1;
        undo
    }

    /// Take a move back, clearing only what its insert set.
    fn unmake(&mut self, undo: Undo) {
        self.archive.undo(undo.key, undo.was_new);
        self.code = undo.code;
        self.to_move = undo.to_move;
        self.passes = undo.passes;
        self.depth -= 1;
    }

    /// Seat the solver at a root of play — `Superko.start`, which archives the
    /// root situation and nothing else.
    fn seat(&mut self, code: PosCode, to_move: Color) -> Undo {
        self.code = code;
        self.to_move = to_move;
        self.passes = 0;
        self.depth = 0;
        self.nodes = 0;
        self.max_depth = 0;
        self.ssk_only = 0;
        self.over_budget = false;
        let key = self.key(code, to_move);
        let was_new = self.archive.insert(key);
        assert!(was_new, "a solver was seated on a dirty archive");
        Undo {
            key,
            was_new,
            code,
            to_move,
            passes: 0,
        }
    }

    /// Count this node, and say whether the budget has run out.
    fn enter(&mut self) -> bool {
        self.nodes = self.nodes.checked_add(1).expect("a node count overflowed");
        self.max_depth = self.max_depth.max(self.depth);
        if self.budget.is_some_and(|b| self.nodes > b) {
            self.over_budget = true;
        }
        self.over_budget
    }

    /// The area difference of the position standing now, read off the table
    /// rather than off a decoded position.
    fn leaf(&self) -> i32 {
        let black =
            i32::try_from(self.table.area(self.code, Color::Black)).expect("area fits an i32");
        let white =
            i32::try_from(self.table.area(self.code, Color::White)).expect("area fits an i32");
        black - white
    }

    /// The leaf test `superko_rules::reference::winner_z` makes, on the
    /// table's two area scores: Black wins when the komi floor is below the
    /// difference, and a tie goes to White.
    ///
    /// This is that function's body rather than a call to it, because a call
    /// would decode a position at every leaf. `tests/agreement.rs` holds the
    /// two equal by checking every verdict against [`crate::naive`], which
    /// does call it.
    fn winner_at(&self, komi_floor: i64) -> Color {
        if komi_floor < i64::from(self.leaf()) {
            Color::Black
        } else {
            Color::White
        }
    }

    /// Fail-soft alpha-beta from the state standing now.
    ///
    /// The value returned is exact when the window contains the whole score
    /// range, which is what [`Solver::solve_root`] passes.
    fn alphabeta(&mut self, mut alpha: i32, mut beta: i32) -> i32 {
        if self.enter() {
            return alpha;
        }
        if self.passes >= 2 {
            return self.leaf();
        }
        let maximizing = self.to_move == Color::Black;
        let mut best = if maximizing { alpha - 1 } else { beta + 1 };
        // Indexed rather than iterated: a borrow of `self.moves` cannot be held
        // across the recursive call, which takes `self` mutably. The sentinel
        // above is never returned, because a pass is always legal and so a
        // state that has not ended always visits a child.
        let mut i = 0;
        while i < self.moves.len() {
            let mv = self.moves[i];
            i += 1;
            let Some(succ) = self.legal_succ(mv) else {
                continue;
            };
            let undo = self.make(mv, succ);
            let v = self.alphabeta(alpha, beta);
            self.unmake(undo);
            if self.over_budget {
                return best;
            }
            if maximizing {
                best = best.max(v);
                alpha = alpha.max(best);
            } else {
                best = best.min(v);
                beta = beta.min(best);
            }
            if beta <= alpha {
                break;
            }
        }
        best
    }

    /// The verdict recursion: whether `c` wins, by the branch structure
    /// `Superko.decideWins` uses.
    fn verdict(&mut self, komi_floor: i64, c: Color) -> bool {
        if self.enter() {
            return false;
        }
        if self.passes >= 2 {
            return self.winner_at(komi_floor) == c;
        }
        let mover = self.to_move == c;
        let mut i = 0;
        while i < self.moves.len() {
            let mv = self.moves[i];
            i += 1;
            let Some(succ) = self.legal_succ(mv) else {
                continue;
            };
            let undo = self.make(mv, succ);
            let won = self.verdict(komi_floor, c);
            self.unmake(undo);
            if self.over_budget {
                return false;
            }
            if mover {
                if won {
                    return true;
                }
            } else if !won {
                return false;
            }
        }
        // The mover found no winning move; the waiter found no losing one.
        !mover
    }

    /// The minimax area difference from a root of play.
    ///
    /// # Panics
    ///
    /// Panics when the code is not a position of this solver's board.
    pub fn solve_root(&mut self, code: PosCode, to_move: Color) -> Solution {
        let seed = self.seat(code, to_move);
        let value = self.alphabeta(-self.span - 1, self.span + 1);
        self.unmake_seed(seed);
        Solution {
            value: (!self.over_budget).then_some(value),
            nodes: self.nodes,
            max_depth: self.max_depth,
            ssk_only: self.ssk_only,
        }
    }

    /// Whether `c` wins from a root of play at this komi floor.
    ///
    /// # Panics
    ///
    /// Panics when the code is not a position of this solver's board.
    pub fn decide_root(
        &mut self,
        code: PosCode,
        to_move: Color,
        komi_floor: i64,
        c: Color,
    ) -> Decision {
        let seed = self.seat(code, to_move);
        let wins = self.verdict(komi_floor, c);
        self.unmake_seed(seed);
        Decision {
            wins: (!self.over_budget).then_some(wins),
            nodes: self.nodes,
            max_depth: self.max_depth,
        }
    }

    /// Undo the root seeding, leaving the archive empty for the next root.
    fn unmake_seed(&mut self, seed: Undo) {
        self.archive.undo(seed.key, seed.was_new);
        assert!(
            self.archive.is_empty(),
            "a search left keys archived: make and unmake are not paired"
        );
    }
}

/// The minimax area difference from a position taken as the root of play.
///
/// # Errors
///
/// Refuses a board the transition table refuses.
///
/// # Panics
///
/// Panics when the board is too large for a position code.
pub fn solve(
    b: &Position,
    to_move: Color,
    rep: Repetition,
    suicide: Suicide,
    budget: Option<u64>,
) -> Result<Solution, TooLarge> {
    let table = RuleTable::build(b.dims(), suicide)?;
    let mut solver = Solver::new(&table, rep).with_budget(budget);
    Ok(solver.solve_root(encode(b), to_move))
}

/// Whether `c` wins from a position taken as the root of play at this komi
/// floor.
///
/// # Errors
///
/// Refuses a board the transition table refuses.
///
/// # Panics
///
/// Panics when the board is too large for a position code.
pub fn decide(
    b: &Position,
    to_move: Color,
    rep: Repetition,
    suicide: Suicide,
    komi_floor: i64,
    c: Color,
    budget: Option<u64>,
) -> Result<Decision, TooLarge> {
    let table = RuleTable::build(b.dims(), suicide)?;
    let mut solver = Solver::new(&table, rep).with_budget(budget);
    Ok(solver.decide_root(encode(b), to_move, komi_floor, c))
}
