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
//!
//! # Symmetry
//!
//! With [`Options::symmetry`] set, the sweep searches one root of each orbit
//! and fills in the others by transport. The orbit of a root `(c, t)` is every
//! `(s(c), t)` and every `(s(swap(c)), t.other)` for `s` a symmetry of the
//! board, `swap` exchanging the color of every stone
//! ([`superko_rules::symmetry`]). Every member of an orbit has the same number
//! of stones, so its **representative** — its least member in the sweep order,
//! code ascending and Black to move before White — is also its least member in
//! the order minimality is taken in. Representatives are searched under both
//! rules, over the threads the options name, exactly as every root is without
//! symmetry. Every other root takes its representative's two [`Solution`]s:
//! the value negated when the map between them exchanges the colors and
//! unchanged when it does not, and the resolution, the depth and the count of
//! plays only situational superko permits copied. Its stones and liberties are
//! read off the root itself.
//!
//! Then every root, searched or transported, goes through the one `fold` in
//! the sweep order, so each field of the [`Sweep`] — resolved and unresolved
//! counts, the least unresolved roots, the separating counts and minima, the
//! empty board's values, the ssk-only counts, the unconditional flags — is
//! what the fold makes of those per-root results. Two fields count the work
//! rather than the roots: [`Sweep::nodes`] adds up the searches that ran, and a
//! transported root adds nothing to it; [`Sweep::searched`] and
//! [`Sweep::transported`] say how many roots were of each kind.
//!
//! **The status of the transport.** It rests on the value of a root being
//! unchanged by a board symmetry and negated by the color swap. That is not
//! proved. It is `computed` root by root with the plain solver
//! (`tests/symmetry.rs`), under both rules, at every root of every board of at
//! most four points and of 2×2 under both suicide conventions and of 1×5 and
//! 5×1 under the no-suicide rule. On 1×5 and 5×1 with suicide removing its own
//! stones it is `computed` only at the roots resolved within the budgets that
//! file names, which leave most of them uncompared (1 628 of 2 916 value pairs
//! on each at 10⁵ nodes a search). On 2×3 and 3×2 under the no-suicide rule,
//! whose group has four elements, the symmetric sweep's values are `computed`
//! equal to the plain sweep's only at the root-and-rule pairs both resolved
//! within 10⁵ nodes a search, 1 504 of 2 916 on each, and no verdict is
//! compared there (`tests/symmetry.rs`, ignored in debug). The facts about the
//! rules it would follow
//! from are the divergences `board-symmetry` and `color-swap`, which a body
//! produced under it names.
//!
//! What the transport copies besides the value is a property of the
//! representative's search, not of the root's. A board symmetry permutes the
//! points, and with them the `all_moves` tie-break of the move order, so the
//! root's own search could spend a different number of nodes, make a different
//! number of ssk-only plays before its cutoffs, and — under a node budget —
//! resolve where the representative's did not, or the reverse. So under a
//! budget the resolved and unresolved counts of a symmetric sweep can differ
//! from a plain one's; a transported root is resolved exactly when its
//! representative is. The color swap alone permutes nothing: the transported
//! search is the same search, node for node, which `tests/symmetry.rs` checks
//! on the boards it names.
//!
//! The verdict searches [`Sweep::lines`] adds for a witness are not
//! transported: they run on the witness root itself, whether its values were
//! searched or transported, and a witness block says which.
//!
//! # Mirrored moves
//!
//! With [`Options::mirrored_moves`] set, every solver the sweep builds — the
//! two per thread and the four of each witness's verdicts — skips the plays
//! [`Solver::with_mirrored_moves`] skips: at a state a board symmetry fixes,
//! archive included, a play the symmetry maps an earlier play onto. It uses
//! the board symmetries only, never the color swap. It is independent of
//! [`Options::symmetry`]: it changes how each root is searched, not which
//! roots are. A root's value and verdicts do not change if the rules commute
//! with the board's symmetries, which is the `board-symmetry` divergence; the
//! node count and the count of ssk-only plays made can, and so, under a
//! budget, can whether a root resolves. [`Sweep::mirrored_skips`] adds up the
//! plays skipped over the searches that ran.

use core::fmt;
use std::panic;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use superko_graph::census::is_legal;
use superko_rules::code::{PosCode, code_space, decode, render};
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::Color;
use superko_rules::symmetry::Symmetries;
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
    /// Whether a symmetric sweep took the two values from the root's orbit
    /// representative rather than searching the root. Always false without
    /// symmetry.
    pub transported: bool,
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
    /// Plays the four searches skipped as mirror images, added up
    /// ([`crate::search::Decision::mirrored_skips`]). Zero when [`verdicts`]
    /// was given no maps. Printed in no body: it is what a test reads to see
    /// that the maps reached the searches. The derived `PartialEq` compares
    /// it, so a check that plain and mirrored `Verdicts` agree must set it
    /// aside first, as `tests/mirrored.rs` does by zeroing it.
    pub mirrored_skips: u64,
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
/// Four searches, not one score search read four ways. With `mirror` given,
/// each search skips mirrored plays ([`Solver::with_mirrored_moves`]).
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
    mirror: Option<&Symmetries>,
) -> Verdicts {
    let ask = |rep: Repetition, c: Color| {
        let mut solver = Solver::new(table, rep).with_budget(budget);
        if let Some(sym) = mirror {
            solver = solver.with_mirrored_moves(sym);
        }
        let d = solver.decide_root(code, to_move, komi_floor, c);
        (
            d.wins.expect("a verdict search ran out of budget"),
            d.mirrored_skips,
        )
    };
    let (psk_black, a) = ask(Repetition::Psk, Color::Black);
    let (psk_white, b) = ask(Repetition::Psk, Color::White);
    let (ssk_black, c) = ask(Repetition::Ssk, Color::Black);
    let (ssk_white, d) = ask(Repetition::Ssk, Color::White);
    Verdicts {
        komi_floor,
        psk_black,
        psk_white,
        ssk_black,
        ssk_white,
        mirrored_skips: a + b + c + d,
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
    /// Nodes visited across every search the sweep ran. A transported root
    /// ran none and adds nothing.
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
    /// Whether the sweep searched orbit representatives only and transported
    /// the rest (the module docs).
    pub symmetry: bool,
    /// Roots whose two searches ran: every root the floor did not skip without
    /// symmetry, the representatives among them with it.
    pub searched: u64,
    /// Roots whose values were transported from their representative; zero
    /// without symmetry. `searched + transported + skipped == roots`.
    pub transported: u64,
    /// Whether every search skipped mirrored plays (the module docs).
    pub mirrored_moves: bool,
    /// Plays skipped as mirror images across every search the sweep ran;
    /// zero without mirrored moves. Like [`Sweep::nodes`], a transported root
    /// adds nothing.
    pub mirrored_skips: u64,
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
    ///
    /// With either half of symmetry on, `symmetry=` follows `skipped=` and
    /// names the [`SymmetryMode`]; then `symmetry-searched=` and
    /// `symmetry-transported=` when canonical roots are on, and
    /// `symmetry-mirrored-moves=on` when mirrored moves are. Each witness
    /// block carries a `-transported=` line when canonical roots are on. With
    /// neither half on, none of these lines appears.
    #[must_use]
    pub fn lines(&self, table: &RuleTable, budget: Option<u64>) -> Vec<String> {
        self.lines_with_verdicts(table, budget).0
    }

    /// [`Sweep::lines`], with the witness verdicts those lines were printed
    /// from, the `minimal` block's first: none when nothing separated, and one
    /// per witness block otherwise.
    ///
    /// `lines` is this with the verdicts dropped, so a test that reads
    /// [`Verdicts::mirrored_skips`] here reads what the printed body's searches
    /// did — in particular that with mirrored moves on they were handed the
    /// board's maps.
    #[must_use]
    pub fn lines_with_verdicts(
        &self,
        table: &RuleTable,
        budget: Option<u64>,
    ) -> (Vec<String>, Vec<Verdicts>) {
        let mut witnessed = Vec::new();
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
        // Only a symmetric sweep prints these, so that a body without symmetry
        // is the body every earlier results file records.
        let mode = SymmetryMode::from_halves(self.symmetry, self.mirrored_moves);
        if mode != SymmetryMode::Off {
            out.push(format!("symmetry={}", mode.name()));
        }
        if self.symmetry {
            out.push(format!("symmetry-searched={}", self.searched));
            out.push(format!("symmetry-transported={}", self.transported));
        }
        if self.mirrored_moves {
            out.push("symmetry-mirrored-moves=on".to_string());
        }
        let sym = (self.mirrored_moves
            && (self.minimal.is_some() || self.minimal_liberties.is_some()))
        .then(|| Symmetries::new(self.dims).expect("a swept board has symmetry maps"));
        for (prefix, found) in [
            ("minimal", self.minimal),
            ("minimal-liberties", self.minimal_liberties),
        ] {
            if let Some(sep) = found {
                let (lines, v) = self.witness_lines(prefix, &sep, table, budget, sym.as_ref());
                out.extend(lines);
                witnessed.extend(v);
            }
        }
        (out, witnessed)
    }

    /// The lines describing one separating root and the verdicts that confirm
    /// it at the lowest komi floor the two rules differ at, with those
    /// verdicts.
    fn witness_lines(
        &self,
        prefix: &str,
        sep: &Separating,
        table: &RuleTable,
        budget: Option<u64>,
        mirror: Option<&Symmetries>,
    ) -> (Vec<String>, Option<Verdicts>) {
        let mut out = vec![
            format!("{prefix}-root={}", render(&decode(self.dims, sep.code))),
            format!("{prefix}-to-move={}", sep.to_move),
            format!("{prefix}-stones={}", sep.stones),
            format!("{prefix}-liberties={}", sep.liberties),
            format!("{prefix}-psk={}", sep.psk),
            format!("{prefix}-ssk={}", sep.ssk),
        ];
        if self.symmetry {
            out.push(format!("{prefix}-transported={}", sep.transported));
        }
        let floors = sep.komi_floors();
        out.push(format!(
            "{prefix}-komi-floors={}",
            floors
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(",")
        ));
        let mut found = None;
        if let Some(&floor) = floors.first() {
            let v = verdicts(table, sep.code, sep.to_move, floor, budget, mirror);
            found = Some(v);
            out.push(format!("{prefix}-witness-komi-floor={}", v.komi_floor));
            out.push(format!("{prefix}-witness-psk-black-wins={}", v.psk_black));
            out.push(format!("{prefix}-witness-psk-white-wins={}", v.psk_white));
            out.push(format!("{prefix}-witness-ssk-black-wins={}", v.ssk_black));
            out.push(format!("{prefix}-witness-ssk-white-wins={}", v.ssk_white));
            out.push(format!("{prefix}-witness-separates={}", v.separates()));
            out.push(format!("{prefix}-witness-determined={}", v.determined()));
        }
        (out, found)
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
            symmetry: false,
            mirrored_moves: false,
        },
    )
}

/// How a sweep runs.
///
/// `budget` and `min_stones` decide what the [`Sweep`] says. `threads` decides
/// only how long it takes. `symmetry` decides how many roots are searched, and
/// `mirrored_moves` how each is searched, and with them the node count and,
/// under a budget, which roots resolve: see the module docs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Options {
    /// The node budget of each search, when there is one.
    pub budget: Option<u64>,
    /// The stone-count floor of the domain; zero sweeps everything.
    pub min_stones: u32,
    /// The number of threads the roots are spread over, at least one. More
    /// threads than roots is allowed; the surplus is not started.
    pub threads: usize,
    /// Search one root of each orbit under the board's symmetries and the
    /// color swap, and transport the values to the rest.
    pub symmetry: bool,
    /// Skip mirrored plays at states a board symmetry fixes, in every search
    /// of the sweep and of its witness verdicts.
    pub mirrored_moves: bool,
}

impl Default for Options {
    /// Every root searched in full, no budget, one thread.
    fn default() -> Self {
        Self {
            budget: None,
            min_stones: 0,
            threads: 1,
            symmetry: false,
            mirrored_moves: false,
        }
    }
}

/// Which halves of the symmetry feature are on, under the four names
/// `superko separate --symmetry` and `superko bench --symmetry` take and a
/// body's `symmetry=` line prints: `off`, `roots` ([`Options::symmetry`]),
/// `moves` ([`Options::mirrored_moves`]) and `on` (both).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SymmetryMode {
    /// Neither half.
    #[default]
    Off,
    /// Canonical roots only: one root of each orbit searched, the rest
    /// transported.
    Roots,
    /// Mirrored moves only: every search skips mirrored plays.
    Moves,
    /// Both halves.
    On,
}

impl SymmetryMode {
    /// Every value, in the order a usage text names them.
    pub const ALL: [Self; 4] = [Self::Off, Self::Roots, Self::Moves, Self::On];

    /// The value with the two halves given.
    #[must_use]
    pub const fn from_halves(roots: bool, moves: bool) -> Self {
        match (roots, moves) {
            (false, false) => Self::Off,
            (true, false) => Self::Roots,
            (false, true) => Self::Moves,
            (true, true) => Self::On,
        }
    }

    /// The spelling a command line takes and a body or `flags` line prints.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Roots => "roots",
            Self::Moves => "moves",
            Self::On => "on",
        }
    }

    /// Whether a sweep searches orbit representatives only.
    #[must_use]
    pub const fn roots(self) -> bool {
        matches!(self, Self::Roots | Self::On)
    }

    /// Whether every search skips mirrored plays.
    #[must_use]
    pub const fn moves(self) -> bool {
        matches!(self, Self::Moves | Self::On)
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
    let roots = solve_roots(table, opts);
    fold(table.dims(), opts, &roots)
}

/// What a sweep recorded for one root, before anything is added up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RootOutcome {
    /// The root position.
    pub code: PosCode,
    /// Who moves first from it.
    pub to_move: Color,
    /// The search under positional superko, or `None` when the stone-count
    /// floor skipped the root. For a transported root, the representative's
    /// search with its value transported.
    pub psk: Option<Solution>,
    /// The search under situational superko, likewise.
    pub ssk: Option<Solution>,
    /// Whether the values were transported from the root's representative.
    /// A transported root's `nodes` and `max_depth` are the representative's
    /// too, so summing `nodes` over outcomes counts each orbit's search once
    /// per member; [`Sweep::nodes`] counts it once.
    pub transported: bool,
}

/// Every root of a sweep with what the sweep recorded for it, in the sweep
/// order: the per-root results [`sweep_with`] folds, for a caller that checks
/// them one by one.
///
/// # Panics
///
/// As [`sweep_with`].
#[must_use]
pub fn root_outcomes(table: &RuleTable, opts: Options) -> Vec<RootOutcome> {
    solve_roots(table, opts)
        .iter()
        .enumerate()
        .map(|(index, root)| {
            let (code, to_move) = root_at(index);
            match *root {
                Root::Skipped => RootOutcome {
                    code,
                    to_move,
                    psk: None,
                    ssk: None,
                    transported: false,
                },
                Root::Searched {
                    psk,
                    ssk,
                    transported,
                    ..
                } => RootOutcome {
                    code,
                    to_move,
                    psk: Some(psk),
                    ssk: Some(ssk),
                    transported,
                },
            }
        })
        .collect()
}

/// A root's value from the value of the root a symmetry maps it to: negated
/// when the map exchanges the colors, unchanged when it does not.
///
/// This is the transport the module docs describe, and its status is theirs.
#[must_use]
pub const fn transported_value(value: i32, swapped: bool) -> i32 {
    if swapped { -value } else { value }
}

/// The komi floor a verdict is transported to.
///
/// A leaf is won by Black at komi floor `k` exactly when `k` is below Black's
/// area less White's (`superko_rules::reference::winner_z`), and a tie goes to
/// White. Exchanging the colors negates the difference, so "`x` wins at floor
/// `k`" on a root becomes "`x.other` wins at floor `-k - 1`" on the swapped
/// root: `k < d` exactly when `-k - 1 >= -d`, the tie included. A board
/// symmetry leaves the floor where it is. This is exact over the integers and
/// does not route through the threshold agreement of the crate docs; that the
/// verdict search respects it is `computed` in `tests/symmetry.rs`.
#[must_use]
pub const fn transported_floor(komi_floor: i64, swapped: bool) -> i64 {
    if swapped { -komi_floor - 1 } else { komi_floor }
}

/// Every root of a sweep, searched or transported.
fn solve_roots(table: &RuleTable, opts: Options) -> Vec<Root> {
    assert!(opts.threads > 0, "a sweep needs at least one thread");
    let dims = table.dims();
    let count =
        usize::try_from(u64::from(code_space(dims)) * 2).expect("the root count fits a usize");
    let maps = (opts.symmetry || opts.mirrored_moves)
        .then(|| Symmetries::new(dims).expect("the board has a transition table, so it has maps"));
    let solvers = || {
        let mut psk = Solver::new(table, Repetition::Psk).with_budget(opts.budget);
        let mut ssk = Solver::new(table, Repetition::Ssk).with_budget(opts.budget);
        if opts.mirrored_moves {
            let sym = maps.as_ref().expect("built when mirrored moves are on");
            psk = psk.with_mirrored_moves(sym);
            ssk = ssk.with_mirrored_moves(sym);
        }
        (psk, ssk)
    };
    if !opts.symmetry {
        return solve_indexed(count, opts.threads, solvers, |(psk, ssk), index| {
            solve_at(psk, ssk, dims, opts.min_stones, index)
        });
    }

    let sym = maps.as_ref().expect("built when symmetry is on");
    let orbits = representatives(sym);
    let reps: Vec<usize> = (0..count).filter(|&i| orbits[i].0 == i).collect();
    let searched = solve_indexed(reps.len(), opts.threads, solvers, |(psk, ssk), k| {
        solve_at(psk, ssk, dims, opts.min_stones, reps[k])
    });
    let mut at: Vec<Option<Root>> = vec![None; count];
    for (&index, root) in reps.iter().zip(searched) {
        at[index] = Some(root);
    }
    (0..count)
        .map(|index| {
            let (rep, swapped) = orbits[index];
            let from = at[rep].expect("every representative was searched");
            if rep == index {
                from
            } else {
                transport(from, swapped, dims, index)
            }
        })
        .collect()
}

/// For every place in the sweep order, the place of its orbit's
/// representative and whether the map from the root to it exchanges the
/// colors.
///
/// The representative is the least place in the orbit, which is the least
/// root in [`rank_of`] order because the orbit's roots share a stone count.
/// When the least place is reached by a map with the swap and by one without,
/// the one without is taken. If the transport is sound the two give the same
/// value — the orbit's values are then their own negations, so zero — and if
/// it is not, `tests/symmetry.rs` fails on that orbit.
fn representatives(sym: &Symmetries) -> Vec<(usize, bool)> {
    let dims = sym.dims();
    let codes = code_space(dims);
    let mut out = Vec::with_capacity(codes as usize * 2);
    for raw in 0..codes {
        let code = PosCode(raw);
        let swapped = sym.swap(code);
        for to_move in [Color::Black, Color::White] {
            let mut best = (place(code, to_move), false);
            for g in 0..sym.order() {
                let plain = (place(sym.code(g, code), to_move), false);
                let exchanged = (place(sym.code(g, swapped), to_move.other()), true);
                best = best.min(plain).min(exchanged);
            }
            out.push(best);
        }
    }
    out
}

/// The place of a root in the sweep order, the inverse of [`root_at`].
fn place(code: PosCode, to_move: Color) -> usize {
    let base = code.0 as usize * 2;
    match to_move {
        Color::Black => base,
        Color::White => base + 1,
    }
}

/// A root's result from its representative's: the stones and liberties of the
/// root itself, the representative's two searches with their values
/// transported.
fn transport(from: Root, swapped: bool, dims: Dims, index: usize) -> Root {
    let (code, _) = root_at(index);
    let board = decode(dims, code);
    let own_stones = stone_count(&board);
    let own_liberties = is_legal(&board);
    match from {
        Root::Skipped => Root::Skipped,
        Root::Searched {
            stones,
            liberties,
            psk,
            ssk,
            ..
        } => {
            // The maps move and recolor stones and add or remove none, and
            // `superko-rules`'s tests/symmetry.rs checks that they keep a
            // position's liberties; a disagreement here is a wrong map.
            assert_eq!(stones, own_stones, "an orbit changed the stone count");
            assert_eq!(liberties, own_liberties, "an orbit changed legality");
            let carry = |s: Solution| Solution {
                value: s.value.map(|v| transported_value(v, swapped)),
                ..s
            };
            Root::Searched {
                stones: own_stones,
                liberties: own_liberties,
                psk: carry(psk),
                ssk: carry(ssk),
                transported: true,
            }
        }
    }
}

/// Stones on a board, both colors.
fn stone_count(board: &superko_rules::reference::Position) -> u32 {
    u32::try_from(board.cells().iter().filter(|cell| cell.is_some()).count())
        .expect("stone count fits a u32")
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
        /// Whether the two solutions are a representative's, transported.
        transported: bool,
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
    let stones = stone_count(&board);
    if stones < min_stones {
        return Root::Skipped;
    }
    Root::Searched {
        stones,
        liberties: is_legal(&board),
        psk: psk.solve_root(code, to_move),
        ssk: ssk.solve_root(code, to_move),
        transported: false,
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
fn fold(dims: Dims, opts: Options, roots: &[Root]) -> Sweep {
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
        min_stones: opts.min_stones,
        symmetry: opts.symmetry,
        searched: 0,
        transported: 0,
        mirrored_moves: opts.mirrored_moves,
        mirrored_skips: 0,
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
            transported,
        } = *root
        else {
            out.skipped += 1;
            continue;
        };
        if transported {
            out.transported += 1;
        } else {
            out.searched += 1;
            out.nodes = out
                .nodes
                .checked_add(a.nodes)
                .and_then(|n| n.checked_add(b.nodes))
                .expect("a node count overflowed");
            out.mirrored_skips = out
                .mirrored_skips
                .checked_add(a.mirrored_skips)
                .and_then(|n| n.checked_add(b.mirrored_skips))
                .expect("a skip count overflowed");
        }
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
            transported,
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

    /// Each mode is the one its two halves name, and its spelling is its own.
    #[test]
    fn a_symmetry_mode_is_its_two_halves() {
        for mode in SymmetryMode::ALL {
            assert_eq!(SymmetryMode::from_halves(mode.roots(), mode.moves()), mode);
        }
        let names: Vec<&str> = SymmetryMode::ALL.iter().map(|m| m.name()).collect();
        assert_eq!(names, ["off", "roots", "moves", "on"]);
        assert_eq!(SymmetryMode::default(), SymmetryMode::Off);
        assert!(!SymmetryMode::Roots.moves() && SymmetryMode::Roots.roots());
        assert!(SymmetryMode::Moves.moves() && !SymmetryMode::Moves.roots());
    }

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
