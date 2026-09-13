//! `superko bench` — the fixed suite the solver's performance changes are
//! measured against (`docs/plans/solver-plan.md` §5.5).
//!
//! # A measurement, not a body
//!
//! Every other subcommand prints a results body that `tools/verify-results.sh`
//! can diff. This one does not: its lines carry wall times, which differ
//! between two runs of the same command, and its purpose is to compare one
//! build of the solver against another on one machine. Its output goes under
//! `data/bench/`, never to `results/`, and a notebook entry quotes it.
//!
//! A value this suite prints is the same `computed` quantity `superko solve`
//! prints for that root, under the same divergences, and nothing here is a
//! claim about Go. A value to be cited is regenerated with `superko solve` or
//! `superko separate` and promoted with a witness header in the usual way.
//!
//! # The suite
//!
//! Every case is under `Suicide::Forbid` and carries its own node budget,
//! within the caps of the plan's §6: at most 10⁸ nodes for a single search and
//! 10⁶ per search of a sweep.
//!
//! | case          | rules      | budget          |
//! |---------------|------------|-----------------|
//! | `empty-1x5`   | PSK, SSK   | 10⁸             |
//! | `empty-1x6`   | PSK, SSK   | 10⁸             |
//! | `empty-1x7`   | PSK, SSK   | 10⁸             |
//! | `empty-2x3`   | PSK, SSK   | 10⁸             |
//! | `sweep-2x3`   | both       | 10⁶ per search  |
//!
//! An `empty-` case solves the empty board with Black to move, on one thread.
//! `sweep-2x3` is `superko_solve::separate::sweep_with` over every root of
//! 2×3 at the thread count `--threads` names (one by default), without the
//! witness verdicts `superko separate` adds when a root separates. The thread
//! count moves only its `seconds`: every other field of the line is a field of
//! the `Sweep`, which does not depend on it (`superko_solve::separate`'s
//! module docs, and its `tests/threads.rs`).
//!
//! `--symmetry` takes one of four values, so that the two halves of the
//! symmetry feature can be measured apart:
//!
//! | value   | canonical roots (sweep case) | mirrored moves (every case) |
//! |---------|------------------------------|-----------------------------|
//! | `off`   | no                           | no                          |
//! | `roots` | yes                          | no                          |
//! | `moves` | no                           | yes                         |
//! | `on`    | yes                          | yes                         |
//!
//! **Canonical roots** reach the sweep case only: it searches one root of each
//! orbit under the board's symmetries and the color swap and transports the
//! rest (`superko_solve::separate`'s module docs). Under the case's budget they
//! can move `resolved`, `unresolved` and the ssk-only counts, since a
//! transported root is resolved exactly when its representative is.
//! **Mirrored moves** reach every case: each search skips a play at a state a
//! board symmetry fixes when the symmetry maps an earlier play onto it
//! (`superko_solve::search`'s module docs). They move `nodes`, `seconds`,
//! `max-depth` and the ssk-only counts, and under a budget whether a search
//! resolves; `mirrored-skips` counts the plays skipped. `superko separate
//! --symmetry on` turns both halves on, as `on` does here.
//!
//! `superko bench` built before mirrored moves existed printed `symmetry=on`
//! for what this build calls `roots`: a `flags` line is read against the build
//! that printed it.
//!
//! # The line format
//!
//! The first line is `flags` followed by the settings in force as `key=value`
//! pairs. Then one line per case, of space-separated `key=value` fields in a
//! fixed order:
//!
//! - `case`, then `rule` — `psk` or `ssk`, or `psk,ssk` for the sweep;
//! - for a single search, `value`: the minimax area difference, or
//!   `unresolved` when the budget stopped the search — an outcome to expect on
//!   `empty-1x7`, which the plan's §1 records unresolved at 4 × 10⁷ nodes; for
//!   the sweep, `resolved`, `unresolved` and
//!   `separating` roots, as `Sweep` counts them;
//! - `nodes`, across both searches of every root for the sweep;
//! - `seconds`: wall time to three decimals, of the same span in both kinds of
//!   case — everything after the transition table is built. For an `empty-`
//!   case that is the construction of its `Solver` (the archive allocation)
//!   and the search; for the sweep it is the whole of `sweep_with`: starting
//!   and joining its threads when there is more than one, each thread's two
//!   `Solver`s, the per-root decode and legality check, every search, and
//!   adding up the per-root results.
//!   Building the table is not timed, since no solver feature changes it.
//!   With either half of symmetry on, the board's symmetry maps are built
//!   inside the span: by `sweep_with` for the sweep, with the orbit
//!   representatives when canonical roots are on, and just before the
//!   `Solver` for an `empty-` case with mirrored moves;
//! - `budget`;
//! - the counters that apply: `max-depth` and `ssk-only` for a single search,
//!   `roots`, `ssk-only-plays` and `resolved-with-ssk-only` for the sweep,
//!   then `searched` and `transported`: the roots whose searches ran and the
//!   roots whose values were transported, `transported=0` without canonical
//!   roots; and last, on both kinds of line, `mirrored-skips`: the plays
//!   skipped as mirror images, over every search of the case, zero without
//!   mirrored moves.
//!
//! A feature that adds a flag records it on the `flags` line through
//! [`Settings::header`], and a feature that adds a counter appends it after
//! the fields above, so a line from an earlier build stays a prefix-compatible
//! reading of a later one.

use std::fmt;
use std::time::Instant;

use superko_rules::code::PosCode;
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::Color;
use superko_rules::symmetry::Symmetries;
use superko_rules::table::RuleTable;
use superko_solve::search::{MoveOrder, Solver};
use superko_solve::separate as sep;

/// The largest budget a single search in the suite may carry.
pub const SOLVE_BUDGET_CAP: u64 = 100_000_000;

/// The largest budget a search of a sweep in the suite may carry.
pub const SWEEP_BUDGET_CAP: u64 = 1_000_000;

/// Which halves of the symmetry feature a bench run has on (the module docs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Symmetry {
    /// Neither half.
    #[default]
    Off,
    /// Canonical roots in the sweep case only.
    Roots,
    /// Mirrored moves in every search only.
    Moves,
    /// Both halves, as `superko separate --symmetry on`.
    On,
}

impl Symmetry {
    /// Every value, in the order the usage text names them.
    pub const ALL: [Self; 4] = [Self::Off, Self::Roots, Self::Moves, Self::On];

    /// The spelling `--symmetry` takes and the `flags` line prints.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Roots => "roots",
            Self::Moves => "moves",
            Self::On => "on",
        }
    }

    /// Whether the sweep case searches orbit representatives only.
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

/// The settings a bench run is under.
///
/// The suite runs the default solver. Each flag a feature adds is a field
/// here, read from the command line in `main.rs` and named by
/// [`Settings::header`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// The threads the sweep case spreads its roots over, at least one. The
    /// `empty-` cases are one search each and ignore it.
    pub threads: usize,
    /// Which halves of the symmetry feature are on.
    pub symmetry: Symmetry,
}

impl Default for Settings {
    /// One thread, no symmetry.
    fn default() -> Self {
        Self {
            threads: 1,
            symmetry: Symmetry::Off,
        }
    }
}

impl Settings {
    /// The first line of a bench run: the settings in force.
    ///
    /// The move order is named although no flag sets it, because it is what a
    /// later ordering feature changes and a baseline that did not say which
    /// order it measured would not be comparable. A line from before the
    /// thread count existed, `flags order=heuristic`, ran on one thread, a
    /// line from before symmetry existed ran without it, and a line from before
    /// mirrored moves existed that says `symmetry=on` ran canonical roots only.
    #[must_use]
    pub fn header(&self) -> String {
        format!(
            "flags order={} threads={} symmetry={}",
            order_name(MoveOrder::default()),
            self.threads,
            self.symmetry.name()
        )
    }
}

/// The name the `flags` line gives a move order.
///
/// `Solver::new` starts every solver the suite builds in
/// `MoveOrder::Heuristic`, and `the_header_names_the_default_order` holds that
/// this is `MoveOrder::default()`. The match is exhaustive, so a new order
/// cannot be printed under an old order's name.
const fn order_name(order: MoveOrder) -> &'static str {
    match order {
        MoveOrder::Heuristic => "heuristic",
        MoveOrder::Static => "static",
    }
}

/// One case of the suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Case {
    /// The empty board with Black to move, under one repetition rule.
    Empty {
        /// The board.
        dims: Dims,
        /// The repetition rule.
        rep: Repetition,
        /// The node budget of the search.
        budget: u64,
    },
    /// The separation sweep of a board.
    Sweep {
        /// The board.
        dims: Dims,
        /// The node budget of each search of the sweep.
        budget: u64,
    },
}

impl Case {
    /// The case's name, as the `case=` field prints it.
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Empty { dims, .. } => format!("empty-{dims}"),
            Self::Sweep { dims, .. } => format!("sweep-{dims}"),
        }
    }

    /// The board the case is on.
    #[must_use]
    pub const fn dims(&self) -> Dims {
        match self {
            Self::Empty { dims, .. } | Self::Sweep { dims, .. } => *dims,
        }
    }
}

const fn empty(rows: usize, cols: usize, rep: Repetition) -> Case {
    Case::Empty {
        dims: Dims::new(rows, cols),
        rep,
        budget: SOLVE_BUDGET_CAP,
    }
}

/// The fixed suite, in the order it runs and prints.
pub const SUITE: [Case; 9] = [
    empty(1, 5, Repetition::Psk),
    empty(1, 5, Repetition::Ssk),
    empty(1, 6, Repetition::Psk),
    empty(1, 6, Repetition::Ssk),
    empty(1, 7, Repetition::Psk),
    empty(1, 7, Repetition::Ssk),
    empty(2, 3, Repetition::Psk),
    empty(2, 3, Repetition::Ssk),
    Case::Sweep {
        dims: Dims::new(2, 3),
        budget: SWEEP_BUDGET_CAP,
    },
];

/// One output line: `key=value` fields in the order they were pushed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Line {
    fields: Vec<(&'static str, String)>,
}

impl Line {
    fn push(&mut self, key: &'static str, value: impl fmt::Display) {
        self.fields.push((key, value.to_string()));
    }
}

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, (key, value)) in self.fields.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            write!(f, "{key}={value}")?;
        }
        Ok(())
    }
}

/// Run one case and describe it.
///
/// # Errors
///
/// Refuses a board the transition table refuses, which no case of [`SUITE`]
/// is.
pub fn run_case(case: &Case, settings: &Settings) -> Result<Line, String> {
    let table = RuleTable::build(case.dims(), Suicide::Forbid).map_err(|e| e.to_string())?;
    let mut line = Line::default();
    line.push("case", case.name());
    match *case {
        Case::Empty { rep, budget, .. } => {
            let started = Instant::now();
            let maps = settings.symmetry.moves().then(|| {
                Symmetries::new(case.dims()).expect("a board with a table has symmetry maps")
            });
            let mut solver = Solver::new(&table, rep).with_budget(Some(budget));
            if let Some(sym) = &maps {
                solver = solver.with_mirrored_moves(sym);
            }
            let solution = solver.solve_root(PosCode(0), Color::Black);
            let seconds = started.elapsed().as_secs_f64();
            line.push("rule", rep);
            line.push(
                "value",
                solution
                    .value
                    .map_or_else(|| "unresolved".to_string(), |v| v.to_string()),
            );
            line.push("nodes", solution.nodes);
            line.push("seconds", format!("{seconds:.3}"));
            line.push("budget", budget);
            line.push("max-depth", solution.max_depth);
            line.push("ssk-only", solution.ssk_only);
            line.push("mirrored-skips", solution.mirrored_skips);
        }
        Case::Sweep { budget, .. } => {
            let started = Instant::now();
            let opts = sep::Options {
                budget: Some(budget),
                min_stones: 0,
                threads: settings.threads,
                symmetry: settings.symmetry.roots(),
                mirrored_moves: settings.symmetry.moves(),
            };
            let sweep = sep::sweep_with(&table, opts);
            let seconds = started.elapsed().as_secs_f64();
            line.push("rule", "psk,ssk");
            line.push("resolved", sweep.resolved);
            line.push("unresolved", sweep.unresolved);
            line.push("separating", sweep.separating);
            line.push("nodes", sweep.nodes);
            line.push("seconds", format!("{seconds:.3}"));
            line.push("budget", budget);
            line.push("roots", sweep.roots);
            line.push("ssk-only-plays", sweep.ssk_only_plays);
            line.push("resolved-with-ssk-only", sweep.resolved_with_ssk_only);
            line.push("searched", sweep.searched);
            line.push("transported", sweep.transported);
            line.push("mirrored-skips", sweep.mirrored_skips);
        }
    }
    Ok(line)
}

/// Run the whole suite, handing each line to `emit` as soon as it is known,
/// so that a long case does not hold back the lines before it.
///
/// # Errors
///
/// Stops at the first case [`run_case`] refuses.
pub fn run(settings: &Settings, mut emit: impl FnMut(&str)) -> Result<(), String> {
    emit(&settings.header());
    for case in &SUITE {
        let line = run_case(case, settings)?;
        emit(&line.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line's fields without `seconds`, which no two runs share; the
    /// position of `seconds` among the keys is checked separately.
    fn fields_but_seconds(line: &Line) -> Vec<(&'static str, String)> {
        line.fields
            .iter()
            .filter(|(key, _)| *key != "seconds")
            .cloned()
            .collect()
    }

    fn keys(line: &Line) -> Vec<&'static str> {
        line.fields.iter().map(|(key, _)| *key).collect()
    }

    fn owned(pairs: &[(&'static str, &str)]) -> Vec<(&'static str, String)> {
        pairs.iter().map(|&(k, v)| (k, v.to_string())).collect()
    }

    #[test]
    fn the_header_names_the_default_order() {
        assert_eq!(MoveOrder::default(), MoveOrder::Heuristic);
        assert_eq!(
            Settings::default().header(),
            "flags order=heuristic threads=1 symmetry=off"
        );
        for symmetry in Symmetry::ALL {
            assert_eq!(
                Settings {
                    threads: 14,
                    symmetry
                }
                .header(),
                format!(
                    "flags order=heuristic threads=14 symmetry={}",
                    symmetry.name()
                )
            );
        }
        let halves: Vec<(bool, bool)> = Symmetry::ALL
            .iter()
            .map(|s| (s.roots(), s.moves()))
            .collect();
        assert_eq!(
            halves,
            [(false, false), (true, false), (false, true), (true, true)]
        );
    }

    /// The suite pinned case by case, budgets included, and the budgets
    /// checked against the plan's §6 caps written out here rather than read
    /// from the constants the suite is built from.
    #[test]
    fn the_suite_is_pinned_and_within_the_caps() {
        let described: Vec<(String, String, u64)> = SUITE
            .iter()
            .map(|case| match *case {
                Case::Empty { rep, budget, .. } => {
                    assert!(budget <= 100_000_000, "{}", case.name());
                    (case.name(), rep.to_string(), budget)
                }
                Case::Sweep { budget, .. } => {
                    assert!(budget <= 1_000_000, "{}", case.name());
                    (case.name(), "psk,ssk".to_string(), budget)
                }
            })
            .collect();
        let expected: Vec<(String, String, u64)> = [
            ("empty-1x5", "psk", 100_000_000),
            ("empty-1x5", "ssk", 100_000_000),
            ("empty-1x6", "psk", 100_000_000),
            ("empty-1x6", "ssk", 100_000_000),
            ("empty-1x7", "psk", 100_000_000),
            ("empty-1x7", "ssk", 100_000_000),
            ("empty-2x3", "psk", 100_000_000),
            ("empty-2x3", "ssk", 100_000_000),
            ("sweep-2x3", "psk,ssk", 1_000_000),
        ]
        .iter()
        .map(|&(n, r, b)| (n.to_string(), r.to_string(), b))
        .collect();
        assert_eq!(described, expected);
    }

    /// A budget too small for the empty 1×5 board reaches the solver: the
    /// search stops one node past it and the line says unresolved. Every
    /// field but `seconds` is pinned.
    #[test]
    fn an_empty_case_runs_under_its_budget() {
        let case = Case::Empty {
            dims: Dims::new(1, 5),
            rep: Repetition::Psk,
            budget: 100,
        };
        let line = run_case(&case, &Settings::default()).unwrap();
        assert_eq!(
            keys(&line),
            [
                "case",
                "rule",
                "value",
                "nodes",
                "seconds",
                "budget",
                "max-depth",
                "ssk-only",
                "mirrored-skips"
            ]
        );
        assert_eq!(
            fields_but_seconds(&line),
            owned(&[
                ("case", "empty-1x5"),
                ("rule", "psk"),
                ("value", "unresolved"),
                ("nodes", "101"),
                ("budget", "100"),
                ("max-depth", "10"),
                ("ssk-only", "0"),
                ("mirrored-skips", "0"),
            ])
        );
    }

    /// At a budget it fits in, the same case reports the value and the node
    /// count the plan's §1 records for the empty 1×5 board under PSK.
    #[test]
    fn an_empty_case_resolves_within_a_sufficient_budget() {
        let case = Case::Empty {
            dims: Dims::new(1, 5),
            rep: Repetition::Psk,
            budget: 100_000_000,
        };
        let line = run_case(&case, &Settings::default()).unwrap();
        assert_eq!(
            fields_but_seconds(&line),
            owned(&[
                ("case", "empty-1x5"),
                ("rule", "psk"),
                ("value", "0"),
                ("nodes", "2115"),
                ("budget", "100000000"),
                ("max-depth", "17"),
                ("ssk-only", "0"),
                ("mirrored-skips", "0"),
            ])
        );
    }

    /// Mirrored moves reach an `empty-` case, and canonical roots do not: with
    /// `moves` or `on` the empty 1×5 board keeps its value and skips plays, and
    /// with `roots` its line is the plain line.
    #[test]
    fn an_empty_case_under_mirrored_moves_skips_plays() {
        let case = Case::Empty {
            dims: Dims::new(1, 5),
            rep: Repetition::Psk,
            budget: 100_000_000,
        };
        let at = |symmetry| {
            fields_but_seconds(
                &run_case(
                    &case,
                    &Settings {
                        threads: 1,
                        symmetry,
                    },
                )
                .unwrap(),
            )
        };
        let plain = at(Symmetry::Off);
        assert_eq!(at(Symmetry::Roots), plain);
        let moves = at(Symmetry::Moves);
        assert_eq!(at(Symmetry::On), moves);
        let field = |line: &[(&'static str, String)], key: &str| {
            line.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| v.clone())
                .unwrap()
        };
        assert_eq!(field(&moves, "value"), "0");
        assert_ne!(field(&moves, "mirrored-skips"), "0");
        // A skip is a subtree not searched; the node count is pinned in
        // `superko_solve`'s tests/mirrored.rs, not here.
        assert_ne!(field(&moves, "nodes"), field(&plain, "nodes"));
    }

    /// A sweep case passes its budget to every search and reports the
    /// `Sweep`'s own counts in the fields that name them: the line is compared
    /// with a direct `sweep_on_above` call at the same budget, and pinned, with
    /// roots left unresolved so that swapping `resolved` and `unresolved`, or
    /// dropping the budget, changes it.
    #[test]
    fn a_sweep_case_reports_the_sweep_it_ran() {
        let dims = Dims::new(2, 2);
        let budget = 50;
        let case = Case::Sweep { dims, budget };
        let line = run_case(&case, &Settings::default()).unwrap();
        assert_eq!(
            keys(&line),
            [
                "case",
                "rule",
                "resolved",
                "unresolved",
                "separating",
                "nodes",
                "seconds",
                "budget",
                "roots",
                "ssk-only-plays",
                "resolved-with-ssk-only",
                "searched",
                "transported",
                "mirrored-skips",
            ]
        );
        let pinned = owned(&[
            ("case", "sweep-2x2"),
            ("rule", "psk,ssk"),
            ("resolved", "64"),
            ("unresolved", "98"),
            ("separating", "0"),
            ("nodes", "12108"),
            ("budget", "50"),
            ("roots", "162"),
            ("ssk-only-plays", "52"),
            ("resolved-with-ssk-only", "0"),
            ("searched", "162"),
            ("transported", "0"),
            ("mirrored-skips", "0"),
        ]);
        assert_eq!(fields_but_seconds(&line), pinned);
        // The thread count reaches the sweep and moves nothing but `seconds`.
        for threads in [2, 14] {
            let spread = run_case(
                &case,
                &Settings {
                    threads,
                    symmetry: Symmetry::Off,
                },
            )
            .unwrap();
            assert_eq!(fields_but_seconds(&spread), pinned, "{threads} threads");
        }

        let table = RuleTable::build(dims, Suicide::Forbid).unwrap();
        let sweep = sep::sweep_on_above(&table, Some(budget), 0);
        assert_eq!(sweep.skipped, 0);
        assert_eq!(sweep.resolved + sweep.unresolved, sweep.roots);
        let n = |v: u64| v.to_string();
        assert_eq!(
            pinned,
            vec![
                ("case", "sweep-2x2".to_string()),
                ("rule", "psk,ssk".to_string()),
                ("resolved", n(sweep.resolved)),
                ("unresolved", n(sweep.unresolved)),
                ("separating", n(sweep.separating)),
                ("nodes", n(sweep.nodes)),
                ("budget", n(budget)),
                ("roots", n(sweep.roots)),
                ("ssk-only-plays", n(sweep.ssk_only_plays)),
                ("resolved-with-ssk-only", n(sweep.resolved_with_ssk_only)),
                ("searched", n(sweep.searched)),
                ("transported", n(sweep.transported)),
                ("mirrored-skips", n(sweep.mirrored_skips)),
            ]
        );
    }

    /// Each value of `symmetry` reaches the sweep as the halves it names: the
    /// line reports the `Sweep` with those options' own counts, canonical roots
    /// search fewer roots than there are, mirrored moves skip plays, and the
    /// thread count still moves nothing but `seconds`.
    #[test]
    fn a_sweep_case_under_symmetry_reports_the_symmetric_sweep() {
        for symmetry in [Symmetry::Roots, Symmetry::Moves, Symmetry::On] {
            a_sweep_case_reports_the_sweep_under(symmetry);
        }
    }

    fn a_sweep_case_reports_the_sweep_under(symmetry: Symmetry) {
        let dims = Dims::new(2, 2);
        let budget = 50;
        let case = Case::Sweep { dims, budget };
        let on = Settings {
            threads: 1,
            symmetry,
        };
        let line = run_case(&case, &on).unwrap();
        let table = RuleTable::build(dims, Suicide::Forbid).unwrap();
        let sweep = sep::sweep_with(
            &table,
            sep::Options {
                budget: Some(budget),
                min_stones: 0,
                threads: 1,
                symmetry: symmetry.roots(),
                mirrored_moves: symmetry.moves(),
            },
        );
        assert_eq!(sweep.transported > 0, symmetry.roots(), "{symmetry:?}");
        assert_eq!(sweep.mirrored_skips > 0, symmetry.moves(), "{symmetry:?}");
        assert_eq!(sweep.searched + sweep.transported, sweep.roots);
        let n = |v: u64| v.to_string();
        let expected = vec![
            ("case", "sweep-2x2".to_string()),
            ("rule", "psk,ssk".to_string()),
            ("resolved", n(sweep.resolved)),
            ("unresolved", n(sweep.unresolved)),
            ("separating", n(sweep.separating)),
            ("nodes", n(sweep.nodes)),
            ("budget", n(budget)),
            ("roots", n(sweep.roots)),
            ("ssk-only-plays", n(sweep.ssk_only_plays)),
            ("resolved-with-ssk-only", n(sweep.resolved_with_ssk_only)),
            ("searched", n(sweep.searched)),
            ("transported", n(sweep.transported)),
            ("mirrored-skips", n(sweep.mirrored_skips)),
        ];
        assert_eq!(fields_but_seconds(&line), expected, "{symmetry:?}");
        let spread = run_case(
            &case,
            &Settings {
                threads: 14,
                symmetry,
            },
        )
        .unwrap();
        assert_eq!(fields_but_seconds(&spread), expected, "{symmetry:?}");
    }
}
