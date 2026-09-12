//! The fast enumerator: depth-first over history-carrying states, with make
//! and unmake over a dense archive.
//!
//! There is **no memoization**. The 1×3 scratch run of 2026-09-11 held 1646
//! memo states for 907 games (`computed`), so an archive-keyed memo is about
//! the size of the tree it would prune, and a memo keyed on less than the
//! archive is the graph-history-interaction error: two states with the same
//! position and different histories do not have the same continuations.
//!
//! The inner loop runs no rule. It reads the transition table of
//! `superko_rules::table`, whose only constructor calls the transliteration of
//! `Defs.lean`, and a per-code occupancy mask read off the same positions. A
//! play whose successor is archived is *skipped*, not removed from a move
//! list: superko forbids a resulting situation, not a move, and the same point
//! may be legal again later in the same game.
//!
//! # What a count means
//!
//! A **game** is a legal alternating sequence of moves from the root, ending at
//! the second consecutive pass — the notion `Superko.Ended` induces, and the
//! notion Tromp–Farnebäck's Table 7 counts (C-9, `cited`). A **node** is a
//! state the search visited. Nothing in this module is proved; every number it
//! returns is `computed`.
//!
//! # The archive key, and why PSK fixes the color
//!
//! Under situational superko the archive key is the situation: the successor's
//! position code and the player to move. Under positional superko the rule
//! reads a position's two bits together — `Superko.PSK` quantifies over
//! archived situations and compares boards alone — so the archive is indexed by
//! position and the key's color is fixed to Black. The two readings of the same
//! structure agree with `superko_rules::reference::psk` because the set of
//! positions the projection archives is the set of positions the reference's
//! `seen` carries: a play archives a position the rule has just checked is
//! absent, and a pass archives the position already standing. That is the
//! divergence `psk-archive-projection` registered in
//! `superko_rules::divergence`, whose Lean license is not written; the evidence
//! here is the agreement of this enumerator with [`crate::walk`], which is
//! `computed` on the boards the tests reach.
//!
//! The fixed color is what makes the pass invariant testable: under PSK a pass
//! must archive a key that is already set, and a `debug_assert` in [`Search::make`]
//! says so on every pass of every debug run.

use std::cmp::max;

use superko_rules::archive::{Archive, ArchiveKey};
use superko_rules::code::{PosCode, code_space, decode, encode};
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::{Color, Move, Position, all_moves};
use superko_rules::table::{RuleTable, TooLarge};

/// The greatest depth [`count_games`] will split a parallel run at.
///
/// A split deeper than this buys little: the prefix list is already far larger
/// than any thread count this workspace uses, and the stem walked to collect it
/// is itself serial.
const MAX_SPLIT_DEPTH: usize = 6;

/// Add one, or panic rather than wrap.
fn inc(counter: &mut u64) {
    *counter = counter.checked_add(1).expect("a count overflowed u64");
}

/// What an enumeration found.
///
/// Every field is deterministic: two runs of the same configuration produce
/// the same report, whatever the thread count. Wall time is not here on
/// purpose — a results body is diffed by `tools/verify-results.sh`, and a
/// timing in it would make every run differ from every other.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// Games: states reached with two consecutive passes behind them.
    pub games: u64,
    /// States visited, the root included.
    pub nodes: u64,
    /// States abandoned at a depth cap, counted separately from games because
    /// a truncated leaf is not a finished game.
    pub truncated: u64,
    /// The greatest number of moves from the root the search reached.
    pub max_depth: usize,
    /// The largest archive the search held, in keys: situations under SSK,
    /// positions under PSK.
    pub max_archive: usize,
    /// Plays refused because the point carried a stone.
    pub refused_occupied: u64,
    /// Plays refused because the player's own chain would have no liberty and
    /// the convention in force forbids that.
    pub refused_suicide: u64,
    /// Plays refused because the resulting situation (SSK) or position (PSK)
    /// was archived.
    pub refused_repetition: u64,
}

impl Report {
    /// The body lines a results file carries, in a fixed order.
    ///
    /// The depth cap and the truncated-leaf count are not here: a capped run is
    /// not a finished count, and [`crate::enumerate::Options::depth_cap`] is the
    /// caller's to print alongside.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        vec![
            format!("games={}", self.games),
            format!("nodes={}", self.nodes),
            format!("max-depth={}", self.max_depth),
            format!("max-archive={}", self.max_archive),
            format!("refused-occupied={}", self.refused_occupied),
            format!("refused-suicide={}", self.refused_suicide),
            format!("refused-repetition={}", self.refused_repetition),
        ]
    }

    /// Two reports of disjoint parts of one search, added.
    ///
    /// Counts add with `checked_add`; the two maxima take the larger. Addition
    /// of `u64` is exact and associative, so a parallel total equals the serial
    /// one whatever order the parts arrive in — [`count_games`] sums them in
    /// prefix order regardless, so that a future inexact accumulator would not
    /// silently make the answer depend on thread scheduling.
    ///
    /// # Panics
    ///
    /// Panics when a count overflows `u64`.
    #[must_use]
    pub fn merge(self, other: Self) -> Self {
        let add = |a: u64, b: u64| a.checked_add(b).expect("a count overflowed u64");
        Self {
            games: add(self.games, other.games),
            nodes: add(self.nodes, other.nodes),
            truncated: add(self.truncated, other.truncated),
            max_depth: max(self.max_depth, other.max_depth),
            max_archive: max(self.max_archive, other.max_archive),
            refused_occupied: add(self.refused_occupied, other.refused_occupied),
            refused_suicide: add(self.refused_suicide, other.refused_suicide),
            refused_repetition: add(self.refused_repetition, other.refused_repetition),
        }
    }
}

/// How an enumeration is run. Neither field changes what is counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// Stop at this many moves from the root, counting the abandoned states as
    /// [`Report::truncated`] rather than as games. `None` runs to the end of
    /// every game.
    pub depth_cap: Option<usize>,
    /// How many threads to count on. One is the serial search, and is the only
    /// figure the plan permits a headline count to be quoted from without a
    /// serial run agreeing.
    pub threads: usize,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            depth_cap: None,
            threads: 1,
        }
    }
}

/// Everything about a board that does not change during a search: the
/// transition table, an occupancy mask per position code, the move order, and
/// the root.
///
/// Built once and shared by every thread of a parallel run.
#[derive(Clone, Debug)]
pub struct Board {
    dims: Dims,
    suicide: Suicide,
    table: RuleTable,
    occupied: Vec<u32>,
    moves: Vec<Move>,
    root: PosCode,
}

impl Board {
    /// Build the table and the occupancy masks for a board.
    ///
    /// The occupancy mask of a code is read off the position that
    /// `superko_rules::code::decode` gives for it, one bit per point in the
    /// row-major order `Dims::index` fixes. It is not a rule: the refusal
    /// census needs to tell an occupied point from a suicide, and
    /// `PlayableAt` is a conjunction that the table stores already conjoined.
    ///
    /// # Errors
    ///
    /// Refuses a board the transition table refuses.
    ///
    /// # Panics
    ///
    /// Panics when the board is too large for a position code.
    pub fn new(dims: Dims, suicide: Suicide) -> Result<Self, TooLarge> {
        let table = RuleTable::build(dims, suicide)?;
        let occupied = (0..code_space(dims))
            .map(|code| {
                let b = decode(dims, PosCode(code));
                let mut mask = 0u32;
                for (i, cell) in b.cells().iter().enumerate() {
                    if cell.is_some() {
                        mask |= 1u32 << i;
                    }
                }
                mask
            })
            .collect();
        Ok(Self {
            dims,
            suicide,
            table,
            occupied,
            moves: all_moves(dims),
            root: encode(&Position::empty(dims)),
        })
    }

    /// The board.
    #[must_use]
    pub const fn dims(&self) -> Dims {
        self.dims
    }

    /// The suicide convention the table was built under.
    #[must_use]
    pub const fn suicide(&self) -> Suicide {
        self.suicide
    }

    /// The root of play: the empty position, Black to move.
    #[must_use]
    pub fn root(&self) -> Position {
        decode(self.dims, self.root)
    }
}

/// What the archive is undone with after a move is taken back.
#[derive(Clone, Copy, Debug)]
struct Undo {
    key: ArchiveKey,
    was_new: bool,
    code: PosCode,
    to_move: Color,
    passes: u32,
}

/// One depth-first search over one board.
struct Search<'b> {
    board: &'b Board,
    rep: Repetition,
    archive: Archive,
    code: PosCode,
    to_move: Color,
    passes: u32,
    depth_cap: Option<usize>,
    split_at: Option<usize>,
    path: Vec<Move>,
    prefixes: Vec<Vec<Move>>,
    report: Report,
}

impl<'b> Search<'b> {
    /// A search at the root of play.
    fn new(board: &'b Board, rep: Repetition, depth_cap: Option<usize>) -> Self {
        let mut search = Self {
            board,
            rep,
            archive: Archive::new(board.dims),
            code: board.root,
            to_move: Color::Black,
            passes: 0,
            depth_cap,
            split_at: None,
            path: Vec::new(),
            prefixes: Vec::new(),
            report: Report::default(),
        };
        // The root is seeded, as `Superko.start` seeds it. The seed is
        // observable: on 1×1 under `RemoveOwn` and PSK it is what holds the
        // count at the published 1.
        let root_key = search.key(board.root, Color::Black);
        let was_new = search.archive.insert(root_key);
        assert!(was_new, "an empty archive holds nothing");
        search
    }

    /// The archive key of a situation, under the rule in force.
    fn key(&self, code: PosCode, to_move: Color) -> ArchiveKey {
        match self.rep {
            Repetition::Ssk => ArchiveKey::new(code, to_move),
            // Positional superko reads the two bits of a code together, so the
            // archive is indexed by position and the color is fixed.
            Repetition::Psk => ArchiveKey::new(code, Color::Black),
        }
    }

    /// The successor position of a move the rules permit, or `None` when the
    /// move is refused — in which case the refusal has been censused.
    ///
    /// The conjuncts are tested in the priority
    /// `superko_rules::reference::legality` fixes: occupied, then suicide, then
    /// repetition.
    fn classify(&mut self, mv: Move) -> Option<PosCode> {
        let Move::Play(p) = mv else {
            return Some(self.code);
        };
        let point = self.board.dims.index(p);
        if self.board.occupied[self.code.0 as usize] & (1u32 << point) != 0 {
            inc(&mut self.report.refused_occupied);
            return None;
        }
        if !self.board.table.playable(self.code, self.to_move, p) {
            inc(&mut self.report.refused_suicide);
            return None;
        }
        let succ = self.board.table.succ(self.code, self.to_move, p);
        if self
            .archive
            .contains(self.key(succ, self.to_move.other()), self.rep)
        {
            inc(&mut self.report.refused_repetition);
            return None;
        }
        Some(succ)
    }

    /// Take a move whose legality has been decided, and return what undoes it.
    fn make(&mut self, mv: Move, succ: PosCode) -> Undo {
        let undo = Undo {
            key: self.key(succ, self.to_move.other()),
            was_new: false,
            code: self.code,
            to_move: self.to_move,
            passes: self.passes,
        };
        let was_new = self.archive.insert(undo.key);
        match mv {
            // `now ∈ seen` is an invariant, and a pass does not change the
            // position, so under positional superko the key a pass archives is
            // already set. Under situational superko it may be new: the pass
            // hands the turn to the other color, and that situation need not
            // have occurred.
            Move::Pass => debug_assert!(
                !matches!(self.rep, Repetition::Psk) || !was_new,
                "under PSK a pass archived a position that was not already archived"
            ),
            Move::Play(_) => debug_assert!(
                was_new,
                "a legal play archived a key that was already archived"
            ),
        }
        self.code = succ;
        self.to_move = self.to_move.other();
        self.passes = match mv {
            Move::Pass => self.passes.checked_add(1).expect("pass count overflowed"),
            Move::Play(_) => 0,
        };
        Undo { was_new, ..undo }
    }

    /// Take a move back, clearing only what its insert set.
    fn unmake(&mut self, undo: Undo) {
        self.archive.undo(undo.key, undo.was_new);
        self.code = undo.code;
        self.to_move = undo.to_move;
        self.passes = undo.passes;
    }

    /// Replay a prefix of legal moves without counting anything: what a worker
    /// thread does before it starts counting at the state its prefix names.
    fn seed(&mut self, path: &[Move]) {
        for &mv in path {
            let succ = self
                .classify(mv)
                .expect("a split prefix is a sequence of legal moves");
            let _ = self.make(mv, succ);
        }
        self.report = Report::default();
    }

    /// Visit the state now standing, `depth` moves from the root.
    fn visit(&mut self, depth: usize) {
        if self.split_at == Some(depth) {
            self.prefixes.push(self.path.clone());
            return;
        }

        inc(&mut self.report.nodes);
        self.report.max_depth = max(self.report.max_depth, depth);
        self.report.max_archive = max(self.report.max_archive, self.archive.len());

        // `Superko.Ended`: two consecutive passes.
        if self.passes >= 2 {
            inc(&mut self.report.games);
            return;
        }

        if self.depth_cap == Some(depth) {
            inc(&mut self.report.truncated);
            return;
        }

        // Copied out of `self` so that the move order can be read while the
        // search mutates its own counters.
        let board = self.board;
        for &mv in &board.moves {
            let Some(succ) = self.classify(mv) else {
                continue;
            };
            let undo = self.make(mv, succ);
            if self.split_at.is_some() {
                self.path.push(mv);
            }
            self.visit(depth + 1);
            if self.split_at.is_some() {
                self.path.pop();
            }
            self.unmake(undo);
        }
    }
}

/// Count the games of a board under a repetition rule and a suicide
/// convention, from the empty position with Black to move.
///
/// # Errors
///
/// Refuses a board the transition table refuses.
///
/// # Panics
///
/// Panics when a count overflows `u64`, or when a counting thread panics.
pub fn count_games(
    dims: Dims,
    rep: Repetition,
    suicide: Suicide,
    opts: Options,
) -> Result<Report, TooLarge> {
    let board = Board::new(dims, suicide)?;
    Ok(count_on(&board, rep, opts))
}

/// Count the games of a board already built.
///
/// # Panics
///
/// Panics when a count overflows `u64`, or when a counting thread panics.
#[must_use]
pub fn count_on(board: &Board, rep: Repetition, opts: Options) -> Report {
    if opts.threads <= 1 {
        return serial(board, rep, opts.depth_cap);
    }
    parallel(board, rep, opts)
}

/// The single-threaded search.
fn serial(board: &Board, rep: Repetition, depth_cap: Option<usize>) -> Report {
    let mut search = Search::new(board, rep, depth_cap);
    search.visit(0);
    search.report
}

/// Walk the tree to `depth` and return what was counted above that depth
/// together with the legal prefixes reaching it, in `all_moves` order.
fn split(
    board: &Board,
    rep: Repetition,
    depth_cap: Option<usize>,
    depth: usize,
) -> (Report, Vec<Vec<Move>>) {
    let mut search = Search::new(board, rep, depth_cap);
    search.split_at = Some(depth);
    search.visit(0);
    (search.report, search.prefixes)
}

/// The prefix-split parallel search.
///
/// The tree is cut at the shallowest depth that yields at least four prefixes
/// per thread, to a fixed ceiling. Each prefix is counted independently on its
/// own state, and the parts are summed **in prefix order**. Nothing about the
/// total depends on the thread count, and the tests hold the parallel report
/// equal to the serial one.
fn parallel(board: &Board, rep: Repetition, opts: Options) -> Report {
    let threads = opts.threads;
    let mut depth = 1;
    let (mut stem, mut prefixes) = split(board, rep, opts.depth_cap, depth);
    while !prefixes.is_empty() && prefixes.len() < threads * 4 && depth < MAX_SPLIT_DEPTH {
        depth += 1;
        let deeper = split(board, rep, opts.depth_cap, depth);
        stem = deeper.0;
        prefixes = deeper.1;
    }

    // Every game ends above the split depth: there is nothing to hand out, and
    // the stem is already the whole count.
    if prefixes.is_empty() {
        return stem;
    }

    let counted: Vec<(usize, Report)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|worker| {
                let prefixes = &prefixes;
                scope.spawn(move || {
                    let mut out = Vec::new();
                    let mut i = worker;
                    while i < prefixes.len() {
                        let prefix = &prefixes[i];
                        let mut search = Search::new(board, rep, opts.depth_cap);
                        search.seed(prefix);
                        search.visit(prefix.len());
                        out.push((i, search.report));
                        i += threads;
                    }
                    out
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|h| h.join().expect("a counting thread panicked"))
            .collect()
    });

    let mut parts: Vec<Option<Report>> = vec![None; prefixes.len()];
    for (i, report) in counted {
        parts[i] = Some(report);
    }
    let mut total = stem;
    for (i, part) in parts.into_iter().enumerate() {
        total = total.merge(part.unwrap_or_else(|| panic!("prefix {i} was never counted")));
    }
    total
}
