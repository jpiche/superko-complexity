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
//! order-dependent in its node count. The default order,
//! [`MoveOrder::Heuristic`], keeps the pass where
//! `superko_rules::reference::all_moves` puts it — first — and sorts the plays
//! by a one-ply area count, with `all_moves` order breaking ties.
//! [`MoveOrder::Static`] is the `all_moves` order itself, which
//! `superko_graph::enumerate` counts in and `tests/rules_walk.rs` pins.
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
//!
//! # Mirrored moves
//!
//! [`Solver::with_mirrored_moves`], off by default, skips plays whose subtree
//! is the image of one already on the move list under a symmetry of the
//! board ([`superko_rules::symmetry`]). A board symmetry `s` fixes a state when
//! it fixes the position, `s(code) = code`, and maps the set `A` of archived
//! keys onto itself; a key is moved by moving its position and keeping its
//! color, and under positional superko a key's color is always Black
//! ([`Solver`]'s `key`). The states a node's position and archive are fixed by
//! form a subgroup `H` of the board's group. At a node where `H` has more than
//! the identity, a play `p` is skipped when some `h` in `H` maps a play `q`
//! earlier in the ordered move list to `p`, so the first play of each
//! `H`-orbit is searched and the rest are not. The pass is never skipped.
//! Both the value search and the verdict search skip, and nothing else about
//! either recursion changes.
//!
//! The skip is sound if the rules commute with every board symmetry: then the
//! state after `h(q)` is the image under `h` of the state after `q`, the two
//! subtrees are images of each other move for move, and they have the same
//! value and the same verdicts, so a maximum, a minimum, an any or an all over
//! the children is unchanged by dropping one of them. That commutation is not
//! proved; it is the `board-symmetry` divergence, and a result produced with
//! mirrored moves on names it. That values and verdicts with the skip equal
//! those without it is `computed` in `tests/mirrored.rs` on the boards and
//! budgets that file names.
//!
//! Whether `s` maps `A` onto itself is kept incrementally. For each
//! non-identity element `s` the solver holds `u_s`, the number of keys `k` in
//! `A` whose image `s(k)` is not in `A`; `A` is finite and `s` is injective,
//! so `s` maps `A` onto itself exactly when `u_s = 0`. Archiving a key `k` that
//! was not already archived changes `u_s` by `[s(k) ≠ k and s(k) ∉ A]` (the new
//! key's own image is missing) less `[s⁻¹(k) ≠ k and s⁻¹(k) ∈ A]` (a key whose
//! image was missing now has it), with `A` taken before the insert. Un-archiving
//! it reverses that change exactly, and an insert that found the key already
//! archived changes nothing. The count does not assume `s` is its own inverse,
//! which the quarter turns of a square board are not.
//! [`Solver::with_unmatched_self_check`] recounts every `u_s` from the archive
//! at every node and panics on a disagreement; the tests run with it on.

use superko_rules::archive::{Archive, ArchiveKey};
use superko_rules::code::{PosCode, code_space, encode};
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::{Color, Move, Position, all_moves};
use superko_rules::symmetry::Symmetries;
use superko_rules::table::{MAX_TABLE_POINTS, RuleTable, TooLarge};

/// The most moves a board can offer: a pass and a play at each point of the
/// largest board the transition table covers.
const MAX_MOVES: usize = MAX_TABLE_POINTS + 1;

/// The most symmetries a board has: the eight of the square.
const MAX_SYMMETRIES: usize = 8;

/// The order a search visits the moves of a node in.
///
/// Alpha-beta returns the minimax value whatever order it visits moves in, and
/// its cutoffs depend on the order entirely, so this is a performance choice
/// and an agreement between the two settings is evidence about the cutoffs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MoveOrder {
    /// The pass where the move list puts it, then the plays best for the
    /// mover first by a one-ply area count. The default, and the only order a
    /// headline number is produced under.
    #[default]
    Heuristic,
    /// The `all_moves` order — the pass, then each point row-major — unsorted.
    /// For the tests: it is a genuinely different order, not a reshuffling of
    /// ties.
    Static,
}

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
    /// Plays positional superko would have refused that the search made —
    /// counted when made, so a play a cutoff pruned is not — always zero under
    /// positional superko, and the measure of whether the two rules met at all
    /// in this tree.
    pub ssk_only: u64,
    /// Plays the search skipped because a symmetry fixing the state mapped an
    /// earlier play onto them — counted when the search reaches them in the
    /// move list, so a play a cutoff pruned first is not. Always zero without
    /// [`Solver::with_mirrored_moves`].
    pub mirrored_skips: u64,
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
    /// Plays skipped as mirror images, as [`Solution::mirrored_skips`] counts
    /// them.
    pub mirrored_skips: u64,
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
    order: MoveOrder,
    budget: Option<u64>,
    nodes: u64,
    max_depth: usize,
    ssk_only: u64,
    over_budget: bool,
    mirror: Option<Mirror<'t>>,
    mirrored_skips: u64,
    self_checks: u64,
}

/// What mirrored moves need beside the search state: the board's symmetry
/// maps, and for each element the count of archived keys whose image is not
/// archived (the module docs).
#[derive(Clone, Debug)]
struct Mirror<'t> {
    sym: &'t Symmetries,
    /// `inverse[g]`: the element that undoes `g`.
    inverse: [usize; MAX_SYMMETRIES],
    /// `unmatched[g]`: archived keys whose image under `g` is not archived.
    /// Element 0, the identity, is never read.
    unmatched: [u32; MAX_SYMMETRIES],
    /// Recount `unmatched` from the archive at every node.
    self_check: bool,
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
            order: MoveOrder::Heuristic,
            budget: None,
            nodes: 0,
            max_depth: 0,
            ssk_only: 0,
            over_budget: false,
            mirror: None,
            mirrored_skips: 0,
            self_checks: 0,
        }
    }

    /// Stop a search that visits more than this many nodes, and report it
    /// unresolved rather than returning a partial answer.
    #[must_use]
    pub const fn with_budget(mut self, budget: Option<u64>) -> Self {
        self.budget = budget;
        self
    }

    /// Visit moves in the named order.
    #[must_use]
    pub const fn with_order(mut self, order: MoveOrder) -> Self {
        self.order = order;
        self
    }

    /// Reverse the underlying move list: each point in reverse row-major
    /// order, then the pass.
    ///
    /// Under [`MoveOrder::Static`] this is a wholly different visit order;
    /// under [`MoveOrder::Heuristic`] it reverses the tie-break and also moves
    /// the pass from first to last, which costs alpha-beta its cheapest bound.
    /// Either way the value must not move, and that a reversed search agrees on the
    /// value while disagreeing on the node count is evidence that the cutoffs
    /// preserve the answer — evidence available on boards the naive arbiter of
    /// [`crate::naive`] cannot reach.
    #[must_use]
    pub fn with_reversed_moves(mut self) -> Self {
        self.moves.reverse();
        self
    }

    /// Skip a play at a state a board symmetry fixes when the symmetry maps an
    /// earlier play onto it (the module docs).
    ///
    /// Off by default. A result searched with it on is under the
    /// `board-symmetry` divergence. It never uses the color swap.
    ///
    /// # Panics
    ///
    /// Panics when the maps are of another board.
    #[must_use]
    pub fn with_mirrored_moves(mut self, sym: &'t Symmetries) -> Self {
        assert!(
            sym.dims() == self.dims(),
            "symmetry maps of {} given to a solver of {}",
            sym.dims(),
            self.dims()
        );
        assert!(
            sym.order() <= MAX_SYMMETRIES,
            "a board has at most eight symmetries"
        );
        let mut inverse = [0; MAX_SYMMETRIES];
        for (g, slot) in inverse.iter_mut().enumerate().take(sym.order()) {
            *slot = sym.inverse(g);
        }
        self.mirror = Some(Mirror {
            sym,
            inverse,
            unmatched: [0; MAX_SYMMETRIES],
            self_check: false,
        });
        self
    }

    /// For the tests: at every node, and after every unmake, recount each
    /// element's unmatched keys by reading every bit of the archive, and panic
    /// when the count kept incrementally differs.
    ///
    /// The recount reads `2 · 3^(m·n)` bits per element per node, so it is for
    /// small boards only. [`Solver::self_checks`] says how often it ran.
    ///
    /// # Panics
    ///
    /// Panics when mirrored moves are off: a self-check of nothing would pass.
    #[must_use]
    pub fn with_unmatched_self_check(mut self) -> Self {
        self.mirror
            .as_mut()
            .expect("the self-check needs mirrored moves on")
            .self_check = true;
        self
    }

    /// How many times [`Solver::with_unmatched_self_check`]'s recount has run
    /// over this solver's life, across roots.
    #[must_use]
    pub const fn self_checks(&self) -> u64 {
        self.self_checks
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
    /// It also says whether the play is one **positional superko would have
    /// refused** while situational superko permits it: the same archive answers
    /// both reads, so the answer costs one bit test. The search counts such a
    /// play when it makes it, not when it generates it, so a play a cutoff
    /// pruned is not counted. A search that made no such play has searched a
    /// tree on which the two rules agree, and a report of no separation from it
    /// would say nothing at all; [`Solution::ssk_only`] is what makes that
    /// distinguishable.
    fn legal_succ(&self, mv: Move) -> Option<(PosCode, bool)> {
        match mv {
            Move::Pass => Some((self.code, false)),
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
                let gap = matches!(self.rep, Repetition::Ssk) && self.archive.contains_psk(succ);
                Some((succ, gap))
            }
        }
    }

    /// Take a move whose legality has been decided.
    fn make(&mut self, mv: Move, succ: PosCode) -> Undo {
        let key = self.key(succ, self.to_move.other());
        let undo = Undo {
            key,
            was_new: if self.mirror.is_some() {
                self.archive_insert(key)
            } else {
                self.archive.insert(key)
            },
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
        // Mirrored moves are tested for here, and at each call site below,
        // rather than inside the helpers, so that a solver without them makes
        // no extra call per node: a debug build does not inline them, and the
        // tests run in one.
        if self.mirror.is_some() {
            self.archive_undo(undo.key, undo.was_new);
        } else {
            self.archive.undo(undo.key, undo.was_new);
        }
        self.code = undo.code;
        self.to_move = undo.to_move;
        self.passes = undo.passes;
        self.depth -= 1;
        if self.mirror.is_some() {
            self.check_unmatched();
        }
    }

    /// Archive a key, keeping the unmatched counts when mirrored moves are on,
    /// and say whether it was new.
    fn archive_insert(&mut self, key: ArchiveKey) -> bool {
        if self.mirror.is_some() && !self.archive.contains_ssk(key) {
            self.track(key, true);
        }
        self.archive.insert(key)
    }

    /// Undo [`Solver::archive_insert`], counts included.
    fn archive_undo(&mut self, key: ArchiveKey, was_new: bool) {
        self.archive.undo(key, was_new);
        if was_new && self.mirror.is_some() {
            self.track(key, false);
        }
    }

    /// Apply, or with `inserting` false reverse, the change archiving a key
    /// makes to every element's unmatched count. The archive must be the set
    /// before the insert: without `key`, whichever way the change goes.
    fn track(&mut self, key: ArchiveKey, inserting: bool) {
        let archive = &self.archive;
        let Some(m) = self.mirror.as_mut() else {
            return;
        };
        for g in 1..m.sym.order() {
            let image = ArchiveKey::new(m.sym.code(g, key.code), key.to_move);
            let preimage = ArchiveKey::new(m.sym.code(m.inverse[g], key.code), key.to_move);
            // The new key's own image is missing.
            let gained = u32::from(image != key && !archive.contains_ssk(image));
            // A key whose image was the new key stops being unmatched.
            let lost = u32::from(preimage != key && archive.contains_ssk(preimage));
            let u = &mut m.unmatched[g];
            *u = if inserting {
                (*u + gained)
                    .checked_sub(lost)
                    .expect("an unmatched count went below zero on insert")
            } else {
                (*u + lost)
                    .checked_sub(gained)
                    .expect("an unmatched count went below zero on undo")
            };
        }
    }

    /// When the self-check is on, recount every element's unmatched keys from
    /// the archive and hold the incremental counts to it.
    fn check_unmatched(&mut self) {
        let Some(m) = self.mirror.as_ref() else {
            return;
        };
        if !m.self_check {
            return;
        }
        let codes = code_space(self.dims());
        for g in 1..m.sym.order() {
            let mut recount = 0u32;
            for raw in 0..codes {
                let code = PosCode(raw);
                for color in [Color::Black, Color::White] {
                    let key = ArchiveKey::new(code, color);
                    if self.archive.contains_ssk(key)
                        && !self
                            .archive
                            .contains_ssk(ArchiveKey::new(m.sym.code(g, code), color))
                    {
                        recount += 1;
                    }
                }
            }
            assert_eq!(
                recount,
                m.unmatched[g],
                "the unmatched count of element {g} ({}) is {} incrementally and {recount} \
                 recounted",
                m.sym.transform(g).name(),
                m.unmatched[g]
            );
        }
        self.self_checks += 1;
    }

    /// The non-identity board symmetries that fix the state standing now:
    /// the position is its own image and no archived key's image is missing.
    fn fixing(&self) -> ([usize; MAX_SYMMETRIES], usize) {
        let mut out = [0; MAX_SYMMETRIES];
        let mut len = 0;
        if let Some(m) = &self.mirror {
            for g in 1..m.sym.order() {
                if m.unmatched[g] == 0 && m.sym.code(g, self.code) == self.code {
                    out[len] = g;
                    len += 1;
                }
            }
        }
        (out, len)
    }

    /// Which entries of an ordered move list a mirrored search skips, as a bit
    /// per index: each play that an element fixing the state maps an earlier
    /// play onto. Zero when mirrored moves are off or nothing but the identity
    /// fixes the state.
    ///
    /// The elements fixing a state form a group, so marking the images of the
    /// plays kept is enough: a skipped play's images are images of the kept
    /// play it was the image of.
    fn mirrored(&self, moves: &[(Move, PosCode, bool)]) -> u32 {
        let (fixing, len) = self.fixing();
        let Some(m) = &self.mirror else {
            return 0;
        };
        if len == 0 {
            return 0;
        }
        let dims = self.dims();
        let mut covered = 0u32;
        let mut skip = 0u32;
        for (i, (mv, _, _)) in moves.iter().enumerate() {
            let Move::Play(p) = *mv else {
                continue;
            };
            if covered & (1 << dims.index(p)) != 0 {
                skip |= 1 << i;
            } else {
                for &g in &fixing[..len] {
                    covered |= 1 << dims.index(m.sym.point(g, p));
                }
            }
        }
        skip
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
        self.mirrored_skips = 0;
        self.over_budget = false;
        if let Some(m) = &self.mirror {
            assert!(
                m.unmatched.iter().all(|&u| u == 0),
                "a solver was seated with unmatched counts left over"
            );
        }
        let key = self.key(code, to_move);
        let was_new = self.archive_insert(key);
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
        if self.mirror.is_some() {
            self.check_unmatched();
        }
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

    /// The legal moves from the state standing now, in the order the search
    /// visits them, each with its successor.
    ///
    /// The **pass keeps its place** in the underlying move list, and only the
    /// plays are sorted, by the area difference of the position they lead to,
    /// best for the mover first, with the `all_moves` order breaking ties.
    ///
    /// The pass is exempt because it is the mover's cheapest child — the
    /// opponent's own pass ends the game — so searching it first bounds the
    /// value of stopping here before any play is entered, and that bound is
    /// worth more than any ordering of the plays. Sorting the pass in with the
    /// rest was tried and is worse: on the empty 1×5 board under positional
    /// superko it costs 63 917 nodes, against 35 121 for the unsorted
    /// `all_moves` order and 2 115 for the order adopted here (`computed`,
    /// 2026-09-12, `superko solve --board 1x5 --rule psk --suicide forbid
    /// --root .....`), because a play raising the mover's own area always
    /// sorts ahead of standing still.
    ///
    /// The order changes which cutoffs fire and cannot change the value.
    /// `tests/agreement.rs` holds that by comparing this order against the
    /// unsorted move list reversed.
    ///
    /// Returned in a fixed array rather than a `Vec` so that the recursion
    /// allocates nothing per node, and so that no borrow of `self.moves` is
    /// held across the recursive call.
    fn ordered_moves(&self) -> ([(Move, PosCode, bool); MAX_MOVES], usize) {
        let mut out = [(Move::Pass, self.code, false); MAX_MOVES];
        let mut len = 0;
        let mut i = 0;
        while i < self.moves.len() {
            let mv = self.moves[i];
            i += 1;
            if let Some((succ, gap)) = self.legal_succ(mv) {
                out[len] = (mv, succ, gap);
                len += 1;
            }
        }
        if matches!(self.order, MoveOrder::Heuristic) {
            let maximizing = self.to_move == Color::Black;
            let key = |&(_, succ, _): &(Move, PosCode, bool)| {
                let diff = self.diff_at(succ);
                if maximizing { -diff } else { diff }
            };
            // The pass is always legal, so it is always in `out[..len]`, and
            // the plays form the runs on either side of it. Sorting each run
            // leaves the pass exactly where the move list put it. Both sorts
            // are stable, so `all_moves` order survives as the tie-break.
            let at = out[..len]
                .iter()
                .position(|(mv, _, _)| matches!(mv, Move::Pass))
                .expect("a pass is always legal");
            out[..at].sort_by_key(key);
            out[at + 1..len].sort_by_key(key);
        }
        (out, len)
    }

    /// The area difference of a position named by its code.
    fn diff_at(&self, code: PosCode) -> i32 {
        let black = i32::try_from(self.table.area(code, Color::Black)).expect("area fits an i32");
        let white = i32::try_from(self.table.area(code, Color::White)).expect("area fits an i32");
        black - white
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
        // Never returned: a pass is always legal, so a state that has not
        // ended always visits a child.
        let mut best = if maximizing { alpha - 1 } else { beta + 1 };
        let (moves, len) = self.ordered_moves();
        let mut skip = if self.mirror.is_some() {
            self.mirrored(&moves[..len])
        } else {
            0
        };
        for &(mv, succ, gap) in &moves[..len] {
            let skipped = skip & 1 != 0;
            skip >>= 1;
            if skipped {
                self.mirrored_skips += 1;
                continue;
            }
            let undo = self.make(mv, succ);
            if gap {
                self.ssk_only += 1;
            }
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
        let (moves, len) = self.ordered_moves();
        let mut skip = if self.mirror.is_some() {
            self.mirrored(&moves[..len])
        } else {
            0
        };
        for &(mv, succ, gap) in &moves[..len] {
            let skipped = skip & 1 != 0;
            skip >>= 1;
            if skipped {
                self.mirrored_skips += 1;
                continue;
            }
            let undo = self.make(mv, succ);
            if gap {
                self.ssk_only += 1;
            }
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
            mirrored_skips: self.mirrored_skips,
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
            mirrored_skips: self.mirrored_skips,
        }
    }

    /// Undo the root seeding, leaving the archive empty for the next root.
    fn unmake_seed(&mut self, seed: Undo) {
        self.archive_undo(seed.key, seed.was_new);
        assert!(
            self.archive.is_empty(),
            "a search left keys archived: make and unmake are not paired"
        );
        if let Some(m) = &self.mirror {
            assert!(
                m.unmatched.iter().all(|&u| u == 0),
                "a search left unmatched counts behind: their updates are not paired"
            );
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use superko_rules::reference::Position;

    /// The unmatched counts on 2×2 for archives built by hand, against numbers
    /// worked out here and against the recount, with the quarter turns — the
    /// elements that are not their own inverses — named. A count that assumed
    /// every symmetry an involution, pairing each key with its image, would
    /// give a quarter turn the wrong count on the second archive below.
    #[test]
    fn unmatched_counts_follow_non_involutions() {
        let dims = Dims::new(2, 2);
        let table = RuleTable::build(dims, Suicide::Forbid).unwrap();
        let sym = Symmetries::new(dims).unwrap();
        let mut solver = Solver::new(&table, Repetition::Ssk)
            .with_mirrored_moves(&sym)
            .with_unmatched_self_check();
        let quarter: Vec<usize> = (1..sym.order())
            .filter(|&g| {
                let t = sym.transform(g);
                t.transpose && t.flip_rows != t.flip_cols
            })
            .collect();
        assert_eq!(quarter.len(), 2);
        let half = (1..sym.order())
            .find(|&g| {
                let t = sym.transform(g);
                !t.transpose && t.flip_rows && t.flip_cols
            })
            .unwrap();
        let r = quarter[0];
        assert_ne!(sym.inverse(r), r);

        // One Black stone in a corner, and its images under r.
        let mut cells = vec![None; 4];
        cells[0] = Some(Color::Black);
        let c0 = encode(&Position::from_cells(dims, &cells));
        let turn = |c: PosCode, times: usize| (0..times).fold(c, |c, _| sym.code(r, c));
        let key = |c: PosCode| ArchiveKey::new(c, Color::White);
        let counts = |s: &Solver<'_>| s.mirror.as_ref().unwrap().unmatched;

        // {c0}: every element that moves the corner leaves one key unmatched.
        let seen = solver.archive_insert(key(c0));
        assert!(seen);
        solver.check_unmatched();
        assert_eq!(counts(&solver)[r], 1);
        assert_eq!(counts(&solver)[sym.inverse(r)], 1);
        assert_eq!(counts(&solver)[half], 1);

        // {c0, r(c0)}: r maps c0 into the set and r(c0) out of it, so its count
        // stays at one; r⁻¹ likewise; the half-turn maps both out.
        let c1 = turn(c0, 1);
        assert!(solver.archive_insert(key(c1)));
        solver.check_unmatched();
        assert_eq!(counts(&solver)[r], 1);
        assert_eq!(counts(&solver)[sym.inverse(r)], 1);
        assert_eq!(counts(&solver)[half], 2);

        // An insert of a key already there changes nothing.
        assert!(!solver.archive_insert(key(c1)));
        solver.archive_undo(key(c1), false);
        solver.check_unmatched();

        // All four corners: closed under every element, and the empty board is
        // fixed, so every element fixes the state.
        let c2 = turn(c0, 2);
        let c3 = turn(c0, 3);
        // {c0, r(c0), r²(c0)}: r leaves only r²(c0) unmatched; the half-turn
        // pairs c0 with r²(c0) and leaves only r(c0) unmatched.
        assert!(solver.archive_insert(key(c2)));
        solver.check_unmatched();
        assert_eq!(counts(&solver)[r], 1);
        assert_eq!(counts(&solver)[half], 1);
        assert!(solver.archive_insert(key(c3)));
        solver.check_unmatched();
        assert!(counts(&solver).iter().all(|&u| u == 0));
        solver.code = PosCode(0);
        assert_eq!(solver.fixing().1, sym.order() - 1);

        // Take them back in reverse, and the counts retrace their steps.
        solver.archive_undo(key(c3), true);
        solver.check_unmatched();
        assert_eq!(counts(&solver)[r], 1);
        assert_eq!(counts(&solver)[half], 1);
        solver.archive_undo(key(c2), true);
        solver.check_unmatched();
        assert_eq!(counts(&solver)[r], 1);
        assert_eq!(counts(&solver)[half], 2);
        solver.archive_undo(key(c1), true);
        solver.check_unmatched();
        assert_eq!(counts(&solver)[half], 1);
        solver.archive_undo(key(c0), true);
        solver.check_unmatched();
        assert!(counts(&solver).iter().all(|&u| u == 0));
        assert!(solver.archive.is_empty());
        // Five inserts, one of them not new, and four undos, each recounted.
        assert_eq!(solver.self_checks(), 9);
    }

    /// At the empty 2×2 root every element fixes the state and the four plays
    /// form one orbit, so the pass and one play are searched and three plays
    /// are skipped. At the empty 1×4 root the reversal pairs the four plays.
    #[test]
    fn the_empty_root_keeps_one_play_per_orbit() {
        for (rows, cols, kept) in [(2, 2, 2u32), (1, 4, 3)] {
            let dims = Dims::new(rows, cols);
            let table = RuleTable::build(dims, Suicide::Forbid).unwrap();
            let sym = Symmetries::new(dims).unwrap();
            let mut solver = Solver::new(&table, Repetition::Psk).with_mirrored_moves(&sym);
            let seed = solver.seat(PosCode(0), Color::Black);
            let (moves, len) = solver.ordered_moves();
            let skip = solver.mirrored(&moves[..len]);
            let len = u32::try_from(len).unwrap();
            assert_eq!(len - skip.count_ones(), kept, "{dims}");
            let pass = moves
                .iter()
                .position(|(mv, _, _)| *mv == Move::Pass)
                .unwrap();
            assert_eq!(skip & (1 << pass), 0, "{dims}: the pass was skipped");
            solver.unmake_seed(seed);
        }
    }
}
