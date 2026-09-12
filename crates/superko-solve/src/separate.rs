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
//! that the claim a reader is asked to check never routes through the
//! threshold agreement between scores and winners (see the crate docs).

use core::fmt;

use superko_graph::census::is_legal;
use superko_rules::code::{PosCode, code_space, decode, render};
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::Color;
use superko_rules::table::{RuleTable, TooLarge};

use crate::search::Solver;

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
    /// Roots examined: `2 · 3^(m·n)`, every coloring with each color to move.
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
    /// Plays the situational-superko searches took that positional superko
    /// would have refused. **A sweep reporting no separation with this at zero
    /// reports nothing**: the two rules never met in the trees it searched.
    pub ssk_only_plays: u64,
    /// Roots whose situational-superko search met at least one such play.
    pub roots_with_ssk_only: u64,
    /// **Resolved** roots whose situational-superko search met at least one
    /// such play. This is the number a null result stands on, and it is not
    /// the one above: on a budgeted sweep the roots with repetition cycles in
    /// them are exactly the expensive ones, so a sweep can meet thousands of
    /// such plays and resolve none of the roots that met them.
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
        if self.skipped > 0 {
            return false;
        }
        match (self.minimal, self.unresolved_least) {
            (_, None) => true,
            (None, Some(_)) => false,
            (Some(best), Some((code, to_move, stones))) => {
                best.rank() < rank_of(stones, code, to_move)
            }
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
            format!("empty-psk={}", opt(self.empty_psk)),
            format!("empty-ssk={}", opt(self.empty_ssk)),
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

/// A score that a budget may have left unresolved.
fn opt(v: Option<i32>) -> String {
    v.map_or_else(|| "unresolved".to_string(), |n| n.to_string())
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
/// # Panics
///
/// Panics when the board is too large for a position code.
#[must_use]
pub fn sweep_on_above(table: &RuleTable, budget: Option<u64>, min_stones: u32) -> Sweep {
    let dims = table.dims();
    let mut psk = Solver::new(table, Repetition::Psk).with_budget(budget);
    let mut ssk = Solver::new(table, Repetition::Ssk).with_budget(budget);
    let empty = PosCode(0);

    let mut out = Sweep {
        dims,
        roots: u64::from(code_space(dims)) * 2,
        resolved: 0,
        unresolved: 0,
        unresolved_least: None,
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

    for raw in 0..code_space(dims) {
        let code = PosCode(raw);
        let board = decode(dims, code);
        let liberties = is_legal(&board);
        let stones = u32::try_from(board.cells().iter().filter(|cell| cell.is_some()).count())
            .expect("stone count fits a u32");
        if stones < min_stones {
            out.skipped += 2;
            continue;
        }
        for to_move in [Color::Black, Color::White] {
            let a = psk.solve_root(code, to_move);
            let b = ssk.solve_root(code, to_move);
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
