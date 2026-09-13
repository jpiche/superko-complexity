//! The separation sweep: every root of a board, solved under both repetition
//! rules, looking for one whose value differs.
//!
//! This is the search behind claim C-17. A root is a **separating position**
//! when the minimax area difference under positional superko differs from the
//! one under situational superko. Positional superko is the stricter rule —
//! every play it permits, situational superko permits — so a separation is a
//! place where the extra plays situational superko allows change the outcome
//! and not merely the move list.
//!
//! # What the sweep ranges over
//!
//! Every one of the `3^(m·n)` colorings `Superko.Position` admits, with each
//! color to move: `docs/formal-model.md` §5 takes a position as the root of
//! play, and §7's language carries the color to move as input, so a coloring
//! no play can reach is an instance of the problem and is swept. The report
//! separates out the ones carrying no libertyless chain, which are the
//! positions that could stand in a game (`superko_graph::census::is_legal`).
//!
//! # The order minimality is taken in
//!
//! Within one board: **stones ascending, then position code ascending, then
//! Black to move before White.** The stone count is the substantive part — a
//! position with fewer stones is the simpler witness, and the empty board is
//! the simplest of all. The code order is an arbitrary deterministic tiebreak
//! and carries no meaning.
//!
//! Across boards the order is `m · n` ascending, which is the caller's to
//! iterate; the sweep answers about one board.
//!
//! # Scores are the search, verdicts are the evidence
//!
//! The sweep compares scores because one score search answers about every
//! komi at once. The witness it then names is a **verdict at one komi floor**,
//! produced by [`verdicts`] from the recursion `Superko.decideWins` uses, so
//! the witness itself never routes through the threshold agreement between
//! scores and winners (see the crate docs). A report of *no* separation, and
//! the minimality of a witness, do: they are statements about values, and
//! reach winners only through that agreement.
//!
//! # Threads
//!
//! Every root is an independent pair of searches, so [`sweep_with`] spreads
//! the roots over a fixed number of threads, each owning one positional and
//! one situational [`Solver`] over the shared table. A thread takes roots in
//! whatever order they come to it and records each root's two [`Solution`]s
//! against its place in the sweep order; once every thread has finished, the
//! results are added up in that order by the one function that assembles a
//! [`Sweep`], the same one the one-thread sweep uses.
//!
//! A root's two solutions depend on the root, the rule and the budget, and not
//! on which roots its solver searched before — a solver resets its counters
//! and empties its archive at every root. So nothing in a [`Sweep`], node
//! counts included, depends on the thread count, and neither does anything
//! [`Sweep::lines`] prints: its witness verdict searches run afterwards, on
//! the calling thread. That is a property of this code, not of Go, and
//! `tests/threads.rs` holds it on the boards it names at one, two and fourteen
//! threads.

use core::fmt;
use std::panic;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use superko_graph::census::is_legal;
use superko_rules::code::{PosCode, code_space, decode, render};
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::Color;
use superko_rules::table::{RuleTable, TooLarge};

use crate::search::{Solution, Solver};

/// One root whose value differs between the two repetition rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Separating {
    /// The root position.
    pub code: PosCode,
    /// Who moves first from it.
    pub to_move: Color,
    /// The minimax area difference under positional superko.
    pub psk: i32,
    /// The minimax area difference under situational superko.
    pub ssk: i32,
    /// Stones on the board, both colors.
    pub stones: u32,
    /// Whether no chain of the position lacks a liberty.
    pub liberties: bool,
}

/// The key minimality is taken in: stones, then position code, then Black to
/// move before White.
#[must_use]
pub const fn rank_of(stones: u32, code: PosCode, to_move: Color) -> (u32, u32, u8) {
    let color = match to_move {
        Color::Black => 0,
        Color::White => 1,
    };
    (stones, code.0, color)
}

impl Separating {
    /// The key minimality is taken in — [`rank_of`] of this root.
    #[must_use]
    pub const fn rank(&self) -> (u32, u32, u8) {
        rank_of(self.stones, self.code, self.to_move)
    }

    /// The komi floors at which the two rules give different winners, if the
    /// threshold agreement holds: the integers from the lower value up to one
    /// below the higher.
    ///
    /// Advisory. [`verdicts`] is what checks a komi floor, and the crate docs
    /// say why the distinction matters.
    #[must_use]
    pub fn komi_floors(&self) -> Vec<i64> {
        let lo = i64::from(self.psk.min(self.ssk));
        let hi = i64::from(self.psk.max(self.ssk));
        (lo..hi).collect()
    }
}

/// Both rules' verdicts for both colors at one komi floor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verdicts {
    /// The komi floor the four verdicts are at.
    pub komi_floor: i64,
    /// Black has a winning strategy under positional superko.
    pub psk_black: bool,
    /// White has a winning strategy under positional superko.
    pub psk_white: bool,
    /// Black has a winning strategy under situational superko.
    pub ssk_black: bool,
    /// White has a winning strategy under situational superko.
    pub ssk_white: bool,
}

impl Verdicts {
    /// Whether the two rules disagree about who wins.
    #[must_use]
    pub const fn separates(&self) -> bool {
        self.psk_black != self.ssk_black
    }

    /// Whether exactly one color wins under each rule — determinacy (C-28,
    /// `proved`) seen in the verdicts rather than assumed.
    #[must_use]
    pub const fn determined(&self) -> bool {
        self.psk_black != self.psk_white && self.ssk_black != self.ssk_white
    }
}

/// Both rules' verdicts for both colors at one komi floor, each by the
/// recursion `Superko.decideWins` uses.
///
/// Four searches, not one score search read four ways.
///
/// # Panics
///
/// Panics when the code is not a position of the board, or when a search
/// exceeds `budget` — an unresolved verdict is not a verdict, and this
/// function exists to produce the evidence a claim cites.
#[must_use]
pub fn verdicts(
    table: &RuleTable,
    code: PosCode,
    to_move: Color,
    komi_floor: i64,
    budget: Option<u64>,
) -> Verdicts {
    let ask = |rep: Repetition, c: Color| {
        let mut solver = Solver::new(table, rep).with_budget(budget);
        solver
            .decide_root(code, to_move, komi_floor, c)
            .wins
            .expect("a verdict search ran out of budget")
    };
    Verdicts {
        komi_floor,
        psk_black: ask(Repetition::Psk, Color::Black),
        psk_white: ask(Repetition::Psk, Color::White),
        ssk_black: ask(Repetition::Ssk, Color::Black),
        ssk_white: ask(Repetition::Ssk, Color::White),
    }
}

/// What a sweep of one board found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Sweep {
    /// The board.
    pub dims: Dims,
    /// The size of the domain: `2 · 3^(m·n)`, every coloring with each color to
    /// move, so that `resolved + unresolved + skipped == roots`.
    pub roots: u64,
    /// Roots both searches resolved within the node budget.
    pub resolved: u64,
    /// Roots a search abandoned. These are named, not absorbed: a sweep that
    /// reported a minimum over an unstated subset would read as exhaustive.
    pub unresolved: u64,
    /// The **least** unresolved root in [`Separating::rank`] order — the
    /// order minimality is taken in, not the order the sweep ran in. This is
    /// what decides whether the minimum below is a minimum: a root of lower
    /// rank that no search resolved could be a smaller separating position.
    pub unresolved_least: Option<(PosCode, Color, u32)>,
    /// The least unresolved root carrying no libertyless chain, in the same
    /// order — what decides whether [`Sweep::minimal_liberties`] is a minimum.
    pub unresolved_least_liberties: Option<(PosCode, Color, u32)>,
    /// Separating roots among the resolved ones.
    pub separating: u64,
    /// Separating roots carrying no libertyless chain.
    pub separating_liberties: u64,
    /// The least separating root in [`Separating::rank`] order.
    pub minimal: Option<Separating>,
    /// The least separating root carrying no libertyless chain.
    pub minimal_liberties: Option<Separating>,
    /// The empty board's value under positional superko, when resolved.
    pub empty_psk: Option<i32>,
    /// The empty board's value under situational superko, when resolved.
    pub empty_ssk: Option<i32>,
    /// Nodes visited across every search of the sweep.
    pub nodes: u64,
    /// Plays positional superko would have refused that the
    /// situational-superko searches made — counted when made, so a play a
    /// cutoff pruned is not. **A sweep reporting no separation with this at zero
    /// reports nothing**: the two rules never met in the trees it searched.
    pub ssk_only_plays: u64,
    /// Roots whose situational-superko search made at least one such play.
    pub roots_with_ssk_only: u64,
    /// **Resolved** roots whose situational-superko search made at least one
    /// such play. This is the number a null result stands on, and it is not
    /// the one above: on a budgeted sweep the roots with repetition cycles in
    /// them are exactly the expensive ones, so a sweep can meet thousands of
    /// such plays and resolve none of the roots that made them.
    pub resolved_with_ssk_only: u64,
    /// Roots excluded by a stone-count floor, which restricts the sweep's
    /// domain rather than failing to resolve it.
    pub skipped: u64,
    /// The stone-count floor the sweep ran under; zero sweeps everything.
    pub min_stones: u32,
}

impl Sweep {
    /// Whether the minimum reported is a minimum over the whole board rather
    /// than over the roots that happened to resolve.
    ///
    /// False when some root of lower rank than the reported minimum went
    /// unresolved, and false when nothing separated and anything at all went
    /// unresolved — in that second case the board has not been shown free of
    /// separating positions. False whenever a stone-count floor excluded any
    /// root, since the sweep then ranged over part of the board.
    #[must_use]
    pub fn unconditional(&self) -> bool {
        self.unconditional_over(self.minimal, self.unresolved_least)
    }

    /// Whether the minimum among roots carrying no libertyless chain is a
    /// minimum over every such root of the board: the test of
    /// [`Sweep::unconditional`], restricted to those roots. The two can
    /// differ under a budget, when the least unresolved root overall carries a
    /// dead chain and a root with liberties of lower rank than
    /// [`Sweep::minimal_liberties`] went unresolved.
    #[must_use]
    pub fn unconditional_liberties(&self) -> bool {
        self.unconditional_over(self.minimal_liberties, self.unresolved_least_liberties)
    }

    fn unconditional_over(
        &self,
        minimal: Option<Separating>,
        least_unresolved: Option<(PosCode, Color, u32)>,
    ) -> bool {
        if self.skipped > 0 {
            return false;
        }
        match (minimal, least_unresolved) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(best), Some((code, to_move, stones))) => {
                best.rank() < rank_of(stones, code, to_move)
            }
        }
    }

    /// The empty board's value as a body prints it: the value, `unresolved`
    /// when a budget stopped its search, or `skipped` when a stone-count floor
    /// kept it out of the domain.
    fn empty_value(&self, v: Option<i32>) -> String {
        match v {
            Some(n) => n.to_string(),
            None if self.min_stones > 0 => "skipped".to_string(),
            None => "unresolved".to_string(),
        }
    }

    /// The body lines a results file carries, in a fixed order.
    ///
    /// The `minimal-` block is present when a separating root was found and
    /// absent when none was, and likewise `witness-`: a body that printed
    /// placeholder values for a witness that does not exist would read as one
    /// that does.
    #[must_use]
    pub fn lines(&self, table: &RuleTable, budget: Option<u64>) -> Vec<String> {
        let mut out = vec![
            format!("roots={}", self.roots),
            format!("resolved={}", self.resolved),
            format!("unresolved={}", self.unresolved),
            format!(
                "unresolved-least={}",
                match self.unresolved_least {
                    None => "none".to_string(),
                    Some((code, c, _)) => format!("{}:{c}", render(&decode(self.dims, code))),
                }
            ),
            format!("minimum-unconditional={}", self.unconditional()),
            format!(
                "minimum-liberties-unconditional={}",
                self.unconditional_liberties()
            ),
            format!("empty-psk={}", self.empty_value(self.empty_psk)),
            format!("empty-ssk={}", self.empty_value(self.empty_ssk)),
            format!("separating={}", self.separating),
            format!("separating-liberties={}", self.separating_liberties),
            format!("ssk-only-plays={}", self.ssk_only_plays),
            format!("roots-with-ssk-only={}", self.roots_with_ssk_only),
            format!("resolved-with-ssk-only={}", self.resolved_with_ssk_only),
            format!("min-stones={}", self.min_stones),
            format!("skipped={}", self.skipped),
        ];
        if let Some(sep) = self.minimal {
            out.extend(self.witness_lines("minimal", &sep, table, budget));
        }
        if let Some(sep) = self.minimal_liberties {
            out.extend(self.witness_lines("minimal-liberties", &sep, table, budget));
        }
        out
    }

    /// The lines describing one separating root and the verdicts that confirm
    /// it at the lowest komi floor the two rules differ at.
    fn witness_lines(
        &self,
        prefix: &str,
        sep: &Separating,
        table: &RuleTable,
        budget: Option<u64>,
    ) -> Vec<String> {
        let mut out = vec![
            format!("{prefix}-root={}", render(&decode(self.dims, sep.code))),
            format!("{prefix}-to-move={}", sep.to_move),
            format!("{prefix}-stones={}", sep.stones),
            format!("{prefix}-liberties={}", sep.liberties),
            format!("{prefix}-psk={}", sep.psk),
            format!("{prefix}-ssk={}", sep.ssk),
        ];
        let floors = sep.komi_floors();
        out.push(format!(
            "{prefix}-komi-floors={}",
            floors
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ));
        if let Some(&floor) = floors.first() {
            let v = verdicts(table, sep.code, sep.to_move, floor, budget);
            out.push(format!("{prefix}-witness-komi-floor={}", v.komi_floor));
            out.push(format!("{prefix}-witness-psk-black-wins={}", v.psk_black));
            out.push(format!("{prefix}-witness-psk-white-wins={}", v.psk_white));
            out.push(format!("{prefix}-witness-ssk-black-wins={}", v.ssk_black));
            out.push(format!("{prefix}-witness-ssk-white-wins={}", v.ssk_white));
            out.push(format!("{prefix}-witness-separates={}", v.separates()));
            out.push(format!("{prefix}-witness-determined={}", v.determined()));
        }
        out
    }
}

/// Sweep every root of a board under both repetition rules.
///
/// # Errors
///
/// Refuses a board the transition table refuses.
///
/// # Panics
///
/// Panics when the board is too large for a position code.
pub fn sweep(dims: Dims, suicide: Suicide, budget: Option<u64>) -> Result<Sweep, TooLarge> {
    let table = RuleTable::build(dims, suicide)?;
    Ok(sweep_on(&table, budget))
}

/// Sweep the roots of a board carrying at least `min_stones` stones.
///
/// The floor is a restriction of the domain, not a shortcut: a position with
/// more stones has fewer empty points and a far smaller game tree, so a floor
/// is how a board whose empty root is out of reach can still be searched for a
/// witness. A minimum found under a floor is a minimum of the restricted
/// domain, and [`Sweep::unconditional`] is false whenever a floor excluded
/// anything.
///
/// # Errors
///
/// Refuses a board the transition table refuses.
///
/// # Panics
///
/// Panics when the board is too large for a position code.
pub fn sweep_above(
    dims: Dims,
    suicide: Suicide,
    budget: Option<u64>,
    min_stones: u32,
) -> Result<Sweep, TooLarge> {
    let table = RuleTable::build(dims, suicide)?;
    Ok(sweep_on_above(&table, budget, min_stones))
}

/// Sweep every root of a board whose table is already built.
///
/// # Panics
///
/// Panics when the board is too large for a position code.
#[must_use]
pub fn sweep_on(table: &RuleTable, budget: Option<u64>) -> Sweep {
    sweep_on_above(table, budget, 0)
}

/// Sweep the roots of a built board carrying at least `min_stones` stones.
///
/// The one-thread case of [`sweep_with`].
///
/// # Panics
///
/// Panics when the board is too large for a position code.
#[must_use]
pub fn sweep_on_above(table: &RuleTable, budget: Option<u64>, min_stones: u32) -> Sweep {
    sweep_with(
        table,
        Options {
            budget,
            min_stones,
            threads: 1,
        },
    )
}

/// How a sweep runs.
///
/// `budget` and `min_stones` decide what the [`Sweep`] says. `threads` decides
/// only how long it takes: see the module docs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// The node budget of each search, when there is one.
    pub budget: Option<u64>,
    /// The stone-count floor of the domain; zero sweeps everything.
    pub min_stones: u32,
    /// The number of threads the roots are spread over, at least one. More
    /// threads than roots is allowed; the surplus is not started.
    pub threads: usize,
}

impl Default for Options {
    /// Every root, no budget, one thread.
    fn default() -> Self {
        Self {
            budget: None,
            min_stones: 0,
            threads: 1,
        }
    }
}

/// Sweep the roots of a built board, spread over `opts.threads` threads.
///
/// # Panics
///
/// Panics when `opts.threads` is zero, when the board is too large for a
/// position code, and, with the worker's own payload, when a search panics on
/// any thread.
#[must_use]
pub fn sweep_with(table: &RuleTable, opts: Options) -> Sweep {
    assert!(opts.threads > 0, "a sweep needs at least one thread");
    let dims = table.dims();
    let count =
        usize::try_from(u64::from(code_space(dims)) * 2).expect("the root count fits a usize");
    let roots = solve_indexed(
        count,
        opts.threads,
        || {
            (
                Solver::new(table, Repetition::Psk).with_budget(opts.budget),
                Solver::new(table, Repetition::Ssk).with_budget(opts.budget),
            )
        },
        |(psk, ssk), index| solve_at(psk, ssk, dims, opts.min_stones, index),
    );
    fold(dims, opts.min_stones, &roots)
}

/// What one root of a sweep came to, before anything is added up.
///
/// Everything [`fold`] reads about a root, and nothing that depends on which
/// thread produced it: [`Solver`] resets its counters and leaves its archive
/// empty at every root, so both [`Solution`]s are functions of the root, the
/// rule and the budget.
#[derive(Clone, Copy, Debug)]
enum Root {
    /// Excluded by the stone-count floor.
    Skipped,
    /// Searched under both rules.
    Searched {
        /// Stones on the board, both colors.
        stones: u32,
        /// Whether no chain of the position lacks a liberty.
        liberties: bool,
        /// The search under positional superko.
        psk: Solution,
        /// The search under situational superko.
        ssk: Solution,
    },
}

/// The root at a place in the sweep order: codes ascending, Black to move
/// before White. The place of `(code, to_move)` is `2 · code`, plus one for
/// White.
fn root_at(index: usize) -> (PosCode, Color) {
    let code = PosCode(u32::try_from(index / 2).expect("a root's code fits a u32"));
    let to_move = if index.is_multiple_of(2) {
        Color::Black
    } else {
        Color::White
    };
    (code, to_move)
}

/// Search the root at one place of the sweep order under both rules, unless
/// the stone-count floor excludes it.
fn solve_at(
    psk: &mut Solver<'_>,
    ssk: &mut Solver<'_>,
    dims: Dims,
    min_stones: u32,
    index: usize,
) -> Root {
    let (code, to_move) = root_at(index);
    let board = decode(dims, code);
    let stones = u32::try_from(board.cells().iter().filter(|cell| cell.is_some()).count())
        .expect("stone count fits a u32");
    if stones < min_stones {
        return Root::Skipped;
    }
    Root::Searched {
        stones,
        liberties: is_legal(&board),
        psk: psk.solve_root(code, to_move),
        ssk: ssk.solve_root(code, to_move),
    }
}

/// Run `work` at every index below `count` over `threads` threads, and return
/// the results in index order.
///
/// Each thread builds its own state with `state` and takes the next unclaimed
/// index from a shared counter until none is left, so no thread idles while an
/// index remains. Which thread ran an index, and in what order the threads
/// finished, is invisible in the result. One thread runs on the calling
/// thread, and more threads than indices start only as many as there are
/// indices.
///
/// A panic in `work` stops every thread from claiming further indices, and
/// once all have stopped it is resumed on the calling thread with its own
/// payload.
fn solve_indexed<S, T: Send>(
    count: usize,
    threads: usize,
    state: impl Fn() -> S + Sync,
    work: impl Fn(&mut S, usize) -> T + Sync,
) -> Vec<T> {
    assert!(threads > 0, "work needs at least one thread");
    let next = AtomicUsize::new(0);
    let run = || {
        let _stop = StopOnPanic { next: &next, count };
        let mut own = state();
        let mut out = Vec::new();
        loop {
            let index = next.fetch_add(1, Ordering::Relaxed);
            if index >= count {
                break;
            }
            out.push((index, work(&mut own, index)));
        }
        out
    };

    let workers = threads.min(count).max(1);
    let parts: Vec<Vec<(usize, T)>> = if workers == 1 {
        vec![run()]
    } else {
        thread::scope(|scope| {
            let handles: Vec<_> = (0..workers).map(|_| scope.spawn(run)).collect();
            // Join every thread before resuming any panic, so none is left
            // running a search whose result nobody will read.
            let joined: Vec<_> = handles.into_iter().map(|h| h.join()).collect();
            joined
                .into_iter()
                .map(|part| part.unwrap_or_else(|payload| panic::resume_unwind(payload)))
                .collect()
        })
    };

    let mut slots: Vec<Option<T>> = (0..count).map(|_| None).collect();
    for (index, result) in parts.into_iter().flatten() {
        let slot = &mut slots[index];
        assert!(slot.is_none(), "index {index} was run twice");
        *slot = Some(result);
    }
    slots
        .into_iter()
        .enumerate()
        .map(|(index, result)| result.unwrap_or_else(|| panic!("index {index} was never run")))
        .collect()
}

/// Exhausts the shared counter of [`solve_indexed`] when its thread unwinds,
/// so that the other threads stop claiming work.
struct StopOnPanic<'a> {
    next: &'a AtomicUsize,
    count: usize,
}

impl Drop for StopOnPanic<'_> {
    fn drop(&mut self) {
        if thread::panicking() {
            self.next.store(self.count, Ordering::Relaxed);
        }
    }
}

/// Add up a sweep from its roots, taken in the sweep order.
///
/// The only place a [`Sweep`] is assembled, whatever the thread count.
fn fold(dims: Dims, min_stones: u32, roots: &[Root]) -> Sweep {
    let empty = PosCode(0);
    let mut out = Sweep {
        dims,
        roots: u64::from(code_space(dims)) * 2,
        resolved: 0,
        unresolved: 0,
        unresolved_least: None,
        unresolved_least_liberties: None,
        separating: 0,
        separating_liberties: 0,
        minimal: None,
        minimal_liberties: None,
        empty_psk: None,
        empty_ssk: None,
        nodes: 0,
        ssk_only_plays: 0,
        roots_with_ssk_only: 0,
        resolved_with_ssk_only: 0,
        skipped: 0,
        min_stones,
    };
    assert_eq!(
        u64::try_from(roots.len()).expect("the root count fits a u64"),
        out.roots,
        "a sweep is folded from every root of its board"
    );

    for (index, root) in roots.iter().enumerate() {
        let (code, to_move) = root_at(index);
        let Root::Searched {
            stones,
            liberties,
            psk: a,
            ssk: b,
        } = *root
        else {
            out.skipped += 1;
            continue;
        };
        out.nodes = out
            .nodes
            .checked_add(a.nodes)
            .and_then(|n| n.checked_add(b.nodes))
            .expect("a node count overflowed");
        out.ssk_only_plays = out
            .ssk_only_plays
            .checked_add(b.ssk_only)
            .expect("a count overflowed");
        if b.ssk_only > 0 {
            out.roots_with_ssk_only += 1;
        }
        if code == empty && to_move == Color::Black {
            out.empty_psk = a.value;
            out.empty_ssk = b.value;
        }
        let (Some(pv), Some(sv)) = (a.value, b.value) else {
            out.unresolved += 1;
            let rank = rank_of(stones, code, to_move);
            if out
                .unresolved_least
                .is_none_or(|(c, m, s)| rank < rank_of(s, c, m))
            {
                out.unresolved_least = Some((code, to_move, stones));
            }
            if liberties
                && out
                    .unresolved_least_liberties
                    .is_none_or(|(c, m, s)| rank < rank_of(s, c, m))
            {
                out.unresolved_least_liberties = Some((code, to_move, stones));
            }
            continue;
        };
        out.resolved += 1;
        if b.ssk_only > 0 {
            out.resolved_with_ssk_only += 1;
        }
        if pv == sv {
            continue;
        }
        let found = Separating {
            code,
            to_move,
            psk: pv,
            ssk: sv,
            stones,
            liberties,
        };
        out.separating += 1;
        if out.minimal.is_none_or(|best| found.rank() < best.rank()) {
            out.minimal = Some(found);
        }
        if liberties {
            out.separating_liberties += 1;
            if out
                .minimal_liberties
                .is_none_or(|best| found.rank() < best.rank())
            {
                out.minimal_liberties = Some(found);
            }
        }
    }
    out
}

impl fmt::Display for Separating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "code {} {} to move: psk {} ssk {}",
            self.code, self.to_move, self.psk, self.ssk
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Results come back in index order at every thread count, a surplus of
    /// threads over indices and no indices at all included, and each thread
    /// keeps its own state.
    #[test]
    fn indexed_results_come_back_in_index_order() {
        let expected: Vec<usize> = (0..37).map(|i| i * i).collect();
        for threads in [1, 2, 3, 14, 40] {
            let got = solve_indexed(
                37,
                threads,
                || 0_usize,
                |seen, i| {
                    *seen += 1;
                    i * i
                },
            );
            assert_eq!(got, expected, "{threads} threads");
        }
        assert!(solve_indexed(0, 4, || (), |(), i| i).is_empty());
    }

    /// A panic on a worker reaches the caller with its own message.
    #[test]
    #[should_panic(expected = "index 5 refused")]
    fn a_worker_panic_reaches_the_caller() {
        let _ = solve_indexed(
            20,
            4,
            || (),
            |(), i| {
                assert!(i != 5, "index 5 refused");
                i
            },
        );
    }
}
