//! The two engines, the two recursions and the two orders, held equal.
//!
//! Four separate agreements, because each catches a different way the fast
//! path could be wrong:
//!
//! 1. **Engines.** [`superko_solve::naive`] clones a state per node, runs the
//!    rule predicates themselves and prunes nothing. The fast path reads a
//!    memo of those predicates, makes and unmakes a dense archive, and cuts.
//!    They share no control flow. Bounded to `m · n <= 3`: the naive engine
//!    visits the whole tree, which on four points is billions of nodes. The
//!    fast path with mirrored moves on (`superko_solve::search`'s module docs)
//!    is held to the naive engine on the same boards, 1×3 and 3×1 in an
//!    ignored release test.
//!
//! 2. **Recursions.** The score search and the verdict search are written
//!    separately and neither is computed from the other, so holding
//!    `black wins at komi floor k` equal to `value > k` at every komi floor
//!    from `-(m·n) - 1` to `m·n + 1` checks the threshold agreement the crate
//!    docs mark `computed` rather than assuming it. Floors outside that range
//!    are not tested. On every board of at most five points, suicide
//!    forbidden, the winners under the two superko rules are also held equal
//!    at every such floor, in a test ignored in debug.
//!
//! 3. **Determinacy.** Exactly one color wins from every state (C-28,
//!    `proved`). The verdict searches are held to it.
//!
//! 4. **Move order.** Alpha-beta returns the minimax value whatever order it
//!    visits moves in; its cutoffs depend on the order entirely. A reversed
//!    order agreeing on the value is therefore evidence about the cutoffs, and
//!    it is available on boards the naive engine cannot reach.
//!
//! What all of this establishes is `computed`, on the boards named, and
//! nothing beyond them.

use superko_rules::code::{PosCode, code_space, decode};
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::Color;
use superko_rules::symmetry::Symmetries;
use superko_rules::table::RuleTable;
use superko_solve::naive;
use superko_solve::search::{MoveOrder, Solver};

/// Every board with at most three points, in both orientations.
const SMALL: [(usize, usize); 5] = [(1, 1), (1, 2), (2, 1), (1, 3), (3, 1)];

/// The boards the fast engine alone covers here.
const FAST: [(usize, usize); 5] = [(1, 1), (1, 2), (1, 3), (1, 4), (2, 2)];

/// The boards the move-order comparison covers. 2×2 is out: the unordered
/// reversed search visits the pass last, and the pass is what gives alpha-beta
/// its cheapest bound, so that search on 2×2 is slower than the whole rest of
/// this file by orders of magnitude. 1×4 is in, and is a board the naive
/// arbiter cannot reach, which is the point of the comparison.
const REVERSED: [(usize, usize); 4] = [(1, 1), (1, 2), (1, 3), (1, 4)];

/// The komi floors worth asking about: one below the lowest score and one
/// above the highest, so both ends of the range are exercised.
fn floors(dims: Dims) -> impl Iterator<Item = i64> {
    let span = i64::try_from(dims.point_count()).expect("point count fits an i64");
    (-span - 1)..=(span + 1)
}

#[test]
fn the_engines_agree_on_every_value_of_every_board_of_three_points() {
    for (rows, cols) in SMALL {
        let dims = Dims::new(rows, cols);
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let table = RuleTable::build(dims, suicide).expect("a small board");
            for rep in [Repetition::Psk, Repetition::Ssk] {
                let mut solver = Solver::new(&table, rep);
                for raw in 0..code_space(dims) {
                    let code = PosCode(raw);
                    let board = decode(dims, code);
                    for to_move in [Color::Black, Color::White] {
                        let fast = solver
                            .solve_root(code, to_move)
                            .value
                            .expect("no budget was set");
                        let slow = naive::value_from(&board, to_move, rep, suicide);
                        assert_eq!(
                            fast, slow,
                            "{dims} {rep} {suicide} root {code} {to_move} to move: \
                             fast {fast} naive {slow}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn the_engines_agree_on_every_verdict_of_every_board_of_three_points() {
    for (rows, cols) in SMALL {
        let dims = Dims::new(rows, cols);
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let table = RuleTable::build(dims, suicide).expect("a small board");
            for rep in [Repetition::Psk, Repetition::Ssk] {
                let mut solver = Solver::new(&table, rep);
                for raw in 0..code_space(dims) {
                    let code = PosCode(raw);
                    let board = decode(dims, code);
                    for to_move in [Color::Black, Color::White] {
                        for floor in floors(dims) {
                            for c in [Color::Black, Color::White] {
                                let fast = solver
                                    .decide_root(code, to_move, floor, c)
                                    .wins
                                    .expect("no budget was set");
                                let slow =
                                    naive::wins_from(&board, to_move, rep, suicide, floor, c);
                                assert_eq!(
                                    fast, slow,
                                    "{dims} {rep} {suicide} root {code} {to_move} to move, \
                                     komi floor {floor}, {c}: fast {fast} naive {slow}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The fast engine with mirrored moves against the naive engine on the named
/// boards: every root's value, and both colors' verdicts at every komi floor
/// of [`floors`], `-(m·n) - 1` to `m·n + 1`, under both rules and both suicide
/// conventions. Returns the plays the value
/// searches and the verdict searches skipped.
fn mirrored_engines_agree(boards: &[(usize, usize)]) -> (u64, u64) {
    let (mut value_skips, mut verdict_skips) = (0u64, 0u64);
    for &(rows, cols) in boards {
        let dims = Dims::new(rows, cols);
        let sym = Symmetries::new(dims).expect("a small board");
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let table = RuleTable::build(dims, suicide).expect("a small board");
            for rep in [Repetition::Psk, Repetition::Ssk] {
                let mut solver = Solver::new(&table, rep).with_mirrored_moves(&sym);
                for raw in 0..code_space(dims) {
                    let code = PosCode(raw);
                    let board = decode(dims, code);
                    for to_move in [Color::Black, Color::White] {
                        let sol = solver.solve_root(code, to_move);
                        value_skips += sol.mirrored_skips;
                        let fast = sol.value.expect("no budget was set");
                        let slow = naive::value_from(&board, to_move, rep, suicide);
                        assert_eq!(
                            fast, slow,
                            "{dims} {rep} {suicide} root {code} {to_move} to move: \
                             fast with mirrored moves {fast} naive {slow}"
                        );
                        for floor in floors(dims) {
                            for c in [Color::Black, Color::White] {
                                let d = solver.decide_root(code, to_move, floor, c);
                                verdict_skips += d.mirrored_skips;
                                let fast = d.wins.expect("no budget was set");
                                let slow =
                                    naive::wins_from(&board, to_move, rep, suicide, floor, c);
                                assert_eq!(
                                    fast, slow,
                                    "{dims} {rep} {suicide} root {code} {to_move} to move, \
                                     komi floor {floor}, {c}: fast with mirrored moves {fast} \
                                     naive {slow}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    (value_skips, verdict_skips)
}

/// 1×1, 1×2 and 2×1: every value and verdict of the fast engine with mirrored
/// moves is the naive engine's. The plays skipped are pinned, so the check
/// reaches the skip.
#[test]
fn the_engines_agree_with_mirrored_moves_on_two_points() {
    assert_eq!(
        mirrored_engines_agree(&[(1, 1), (1, 2), (2, 1)]),
        MIRRORED_TWO_SKIPS
    );
}

/// Plays skipped by the value searches and the verdict searches of
/// [`the_engines_agree_with_mirrored_moves_on_two_points`].
const MIRRORED_TWO_SKIPS: (u64, u64) = (32, 224);

/// 1×3 and 3×1: every value and verdict of the fast engine with mirrored moves
/// is the naive engine's, the plays skipped pinned. The naive engine on three
/// points is what `the_engines_agree_on_every_value_of_every_board_of_three_points`
/// spends most of its debug time on, so this second pass over it is release
/// only.
///
/// `cargo test --release -p superko-solve --test agreement -- --ignored the_engines_agree_with_mirrored_moves_on_three_points`
#[test]
#[ignore = "release only: the naive engine on three points"]
fn the_engines_agree_with_mirrored_moves_on_three_points() {
    assert_eq!(
        mirrored_engines_agree(&[(1, 3), (3, 1)]),
        MIRRORED_THREE_SKIPS
    );
}

/// Plays skipped by the value searches and the verdict searches of
/// [`the_engines_agree_with_mirrored_moves_on_three_points`].
const MIRRORED_THREE_SKIPS: (u64, u64) = (188, 780);

/// Exactly one color wins, and Black wins exactly when the value exceeds the
/// komi floor. The first is determinacy (C-28, `proved`) seen in the verdicts;
/// the second is the threshold agreement, which is `computed` and is what this
/// test is the evidence for.
#[test]
fn verdicts_are_determined_and_track_the_value() {
    for (rows, cols) in FAST {
        let dims = Dims::new(rows, cols);
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let table = RuleTable::build(dims, suicide).expect("a small board");
            for rep in [Repetition::Psk, Repetition::Ssk] {
                let mut solver = Solver::new(&table, rep);
                for raw in 0..code_space(dims) {
                    let code = PosCode(raw);
                    for to_move in [Color::Black, Color::White] {
                        let value = solver
                            .solve_root(code, to_move)
                            .value
                            .expect("no budget was set");
                        for floor in floors(dims) {
                            let black = solver
                                .decide_root(code, to_move, floor, Color::Black)
                                .wins
                                .expect("no budget was set");
                            let white = solver
                                .decide_root(code, to_move, floor, Color::White)
                                .wins
                                .expect("no budget was set");
                            assert_ne!(
                                black, white,
                                "{dims} {rep} {suicide} root {code} {to_move} to move at komi \
                                 floor {floor}: both colors or neither has a winning strategy"
                            );
                            assert_eq!(
                                black,
                                i64::from(value) > floor,
                                "{dims} {rep} {suicide} root {code} {to_move} to move: value \
                                 {value} and the verdict at komi floor {floor} disagree"
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Every board of at most five points, in both orientations.
const FIVE: [(usize, usize); 10] = [
    (1, 1),
    (1, 2),
    (2, 1),
    (1, 3),
    (3, 1),
    (1, 4),
    (4, 1),
    (2, 2),
    (1, 5),
    (5, 1),
];

/// Roots of [`FIVE`], `2 · 3^(m·n)` a board: 6 on 1×1, 18 on each two-point
/// line, 54 on each three-point line, 162 on each four-point line and on 2×2,
/// and 486 on each five-point line, so `6 + 2·18 + 2·54 + 3·162 + 2·486`.
const FIVE_ROOTS: u64 = 1608;

/// Verdict searches over [`FIVE`]: at each root, `2·m·n + 3` komi floors, both
/// colors and both rules, so `roots · (2·m·n + 3) · 4` a board: 120 on 1×1,
/// 504 on each two-point line, 1 944 on each three-point line, 7 128 on each
/// four-point line and on 2×2, and 25 272 on each five-point line, so
/// `120 + 2·504 + 2·1944 + 3·7128 + 2·25272`.
const FIVE_VERDICTS: u64 = 76_944;

/// Worker threads for [`the_winners_agree_under_both_rules_on_every_board_of_five_points`].
/// The checks are the same at any count; this only spreads the roots.
const FIVE_THREADS: usize = 8;

/// Under the rules of `Defs.lean` (suicide forbidden), at every root of every
/// board of at most five points in both orientations, with the plain solver
/// and no budget: at every komi floor of [`floors`], `-(m·n) - 1` to `m·n + 1`,
/// exactly one color wins under each rule, Black wins exactly when that rule's
/// value exceeds the floor, the PSK value equals the SSK value, and the PSK
/// winner equals the SSK winner, both colors' verdicts compared.
///
/// The last is the one C-53 needs on 1×5 and 5×1: there the threshold agreement
/// was not otherwise checked, so equal values reached equal winners only by
/// inference. Here the winners are computed by the verdict recursion under
/// each rule and compared directly. Floors outside the range are not tested.
/// The counts of roots and verdict searches are pinned from arithmetic, so the
/// test cannot pass on a loop that visits nothing.
///
/// Release only: 25 272 verdict searches on each five-point line.
///
/// `cargo test --release -p superko-solve --test agreement -- --ignored the_winners_agree_under_both_rules_on_every_board_of_five_points`
#[test]
#[ignore = "release only: every verdict of every root of 1x5 and 5x1"]
fn the_winners_agree_under_both_rules_on_every_board_of_five_points() {
    let (mut roots, mut verdicts) = (0u64, 0u64);
    for (rows, cols) in FIVE {
        let dims = Dims::new(rows, cols);
        let table = RuleTable::build(dims, Suicide::Forbid).expect("a small board");
        let board_roots = 2 * code_space(dims);
        let exponent = u32::try_from(rows * cols).expect("point count fits a u32");
        assert_eq!(
            u64::from(board_roots),
            2 * 3u64.pow(exponent),
            "{dims}: the root count is not 2 * 3^(m*n)"
        );
        let (board_seen, board_verdicts) = std::thread::scope(|s| {
            let workers: Vec<_> = (0..FIVE_THREADS)
                .map(|t| {
                    let table = &table;
                    s.spawn(move || winners_agree_on_roots(table, t, FIVE_THREADS))
                })
                .collect();
            workers
                .into_iter()
                .map(|w| w.join().unwrap_or_else(|e| std::panic::resume_unwind(e)))
                .fold((0u64, 0u64), |(a, b), (c, d)| (a + c, b + d))
        });
        let span = u64::try_from(rows * cols).expect("point count fits a u64");
        assert_eq!(board_seen, u64::from(board_roots), "{dims}: roots visited");
        assert_eq!(
            board_verdicts,
            u64::from(board_roots) * (2 * span + 3) * 4,
            "{dims}: verdict searches made"
        );
        roots += board_seen;
        verdicts += board_verdicts;
    }
    assert_eq!(
        (roots, verdicts),
        (FIVE_ROOTS, FIVE_VERDICTS),
        "roots visited and verdict searches made over every board of at most five points"
    );
}

/// The checks of [`the_winners_agree_under_both_rules_on_every_board_of_five_points`]
/// at the roots `stride · i + offset` of a board's sweep order, root `r` being
/// code `r / 2` with Black to move when `r` is even. Returns the roots checked
/// and the verdict searches made.
fn winners_agree_on_roots(table: &RuleTable, offset: usize, stride: usize) -> (u64, u64) {
    let dims = table.dims();
    let mut psk = Solver::new(table, Repetition::Psk);
    let mut ssk = Solver::new(table, Repetition::Ssk);
    let (mut roots, mut verdicts) = (0u64, 0u64);
    let count = 2 * usize::try_from(code_space(dims)).expect("code space fits a usize");
    for root in (offset..count).step_by(stride) {
        let code = PosCode(u32::try_from(root / 2).expect("a position code"));
        let to_move = if root % 2 == 0 {
            Color::Black
        } else {
            Color::White
        };
        let values = [&mut psk, &mut ssk].map(|solver| {
            solver
                .solve_root(code, to_move)
                .value
                .expect("no budget was set")
        });
        assert_eq!(
            values[0], values[1],
            "{dims} forbid root {code} {to_move} to move: PSK value {} and SSK value {} differ",
            values[0], values[1]
        );
        roots += 1;
        for floor in floors(dims) {
            // wins[rule][color]: rule 0 is PSK, color 0 is Black.
            let mut wins = [[false; 2]; 2];
            for (slot, (rep, solver)) in [(Repetition::Psk, &mut psk), (Repetition::Ssk, &mut ssk)]
                .into_iter()
                .enumerate()
            {
                for (ci, c) in [Color::Black, Color::White].into_iter().enumerate() {
                    wins[slot][ci] = solver
                        .decide_root(code, to_move, floor, c)
                        .wins
                        .expect("no budget was set");
                    verdicts += 1;
                }
                let [black, white] = wins[slot];
                assert_ne!(
                    black, white,
                    "{dims} {rep} forbid root {code} {to_move} to move at komi floor {floor}: \
                     both colors or neither has a winning strategy"
                );
                assert_eq!(
                    black,
                    i64::from(values[slot]) > floor,
                    "{dims} {rep} forbid root {code} {to_move} to move: value {} and the verdict \
                     at komi floor {floor} disagree",
                    values[slot]
                );
            }
            for (ci, c) in [Color::Black, Color::White].into_iter().enumerate() {
                assert_eq!(
                    wins[0][ci], wins[1][ci],
                    "{dims} forbid root {code} {to_move} to move at komi floor {floor}: whether \
                     {c} wins is {} under PSK and {} under SSK",
                    wins[0][ci], wins[1][ci]
                );
            }
        }
    }
    (roots, verdicts)
}

/// The value does not depend on the move order and the node count does. The
/// second half is asserted because an order that changed nothing would make
/// the first half vacuous.
///
/// The comparison is the default heuristic order — the pass in place, the
/// plays sorted by a one-ply area count — against `MoveOrder::Static` run
/// over the reversed move list, which is every point in reverse row-major
/// order and then the pass: the pass is first in one order and last in the other.
///
/// A root the second order cannot reach within its budget is skipped and
/// counted, not failed: the comparison is about the cutoffs, and a search that
/// did not finish makes no claim about them.
#[test]
fn the_value_survives_a_reversed_move_order() {
    /// Nodes the second order may spend on one root.
    const BUDGET: u64 = 2_000_000;
    /// Roots, of the 960 on these boards, the second order resolves within its
    /// budget.
    const COMPARED: u64 = 816;
    let mut differing_counts = 0u64;
    let mut compared = 0u64;
    let mut skipped = 0u64;
    for (rows, cols) in REVERSED {
        let dims = Dims::new(rows, cols);
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let table = RuleTable::build(dims, suicide).expect("a small board");
            for rep in [Repetition::Psk, Repetition::Ssk] {
                let mut forward = Solver::new(&table, rep);
                let mut backward = Solver::new(&table, rep)
                    .with_order(MoveOrder::Static)
                    .with_reversed_moves()
                    .with_budget(Some(BUDGET));
                for raw in 0..code_space(dims) {
                    let code = PosCode(raw);
                    for to_move in [Color::Black, Color::White] {
                        let a = forward.solve_root(code, to_move);
                        let b = backward.solve_root(code, to_move);
                        if b.value.is_none() {
                            skipped += 1;
                            continue;
                        }
                        assert_eq!(
                            a.value, b.value,
                            "{dims} {rep} {suicide} root {code} {to_move} to move: the value \
                             depends on the move order, so a cutoff is unsound"
                        );
                        compared += 1;
                        if a.nodes != b.nodes {
                            differing_counts += 1;
                        }
                    }
                }
            }
        }
    }
    assert!(
        differing_counts > 0,
        "no search changed its node count under the reversed order, so the order \
         agreement was asserted about a case that never arises"
    );
    // Pinned exactly, so that a solver change which quietly stopped reaching
    // these roots — the expensive, cycle-rich ones, likeliest to expose an
    // unsound cutoff — breaks the build instead of shrinking the test.
    assert_eq!(
        (compared, skipped),
        (COMPARED, 960 - COMPARED),
        "the two orders were compared at {compared} roots and {skipped} were skipped at a \
         budget of {BUDGET} nodes; this test records {COMPARED} compared"
    );
}

/// A budget that stops a search leaves no value behind, rather than a bound
/// dressed as one.
#[test]
fn an_exhausted_budget_reports_no_value() {
    let dims = Dims::new(1, 5);
    let table = RuleTable::build(dims, Suicide::Forbid).expect("a small board");
    let empty = PosCode(0);

    let mut capped = Solver::new(&table, Repetition::Psk).with_budget(Some(100));
    let stopped = capped.solve_root(empty, Color::Black);
    assert_eq!(stopped.value, None);
    assert!(stopped.nodes <= 101);

    // The same root without a budget resolves, so the cap is the reason the
    // search stopped and the board is not.
    let mut free = Solver::new(&table, Repetition::Psk);
    assert!(free.solve_root(empty, Color::Black).value.is_some());
}

/// A line board and the line board turned on its side give the same sweep,
/// field for field.
///
/// On a line the position code is transposition-invariant: `Dims::index` is
/// `row · cols + col`, which is the single coordinate whichever way the line
/// runs, so code `k` on 1×n names the transpose of the position code `k` names
/// on n×1, and `all_moves` walks the same index sequence. Adjacency is
/// symmetric in the two coordinates. Two sweeps must therefore agree on
/// everything, the chosen witness included.
///
/// **Off a line this is false**, which is why no rectangle is here. On 2×3 the
/// index is `2 · row + col` and on 3×2 it is `3 · row + col`, so transposing a
/// point permutes the code; the *counts* of separating roots still agree by
/// that bijection, but the move order does not correspond, so alpha-beta
/// explores differently and — under any node budget — resolves a different set
/// of roots. The smallest rectangle that would test the geometry is 2×3, and
/// its sweep is out of reach without a budget.
///
/// This is a check on the geometry rather than on the solver: a `neighbors`
/// that treated rows and columns differently would pass every other test in
/// this file and fail this one.
#[test]
fn a_line_sweep_agrees_with_the_sweep_of_the_line_turned_on_its_side() {
    /// Nodes one search of the sweep may spend. The two sweeps of a line and
    /// its transpose visit the same tree in the same order, so their node
    /// counts are equal and the same budget resolves the same roots in both:
    /// the comparison stays exact rather than becoming a comparison of what
    /// each happened to finish.
    const BUDGET: Option<u64> = Some(2_000_000);
    for n in 2..=5 {
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            // 1×5 with suicide removed leaves roots unresolved at this budget,
            // so the comparison could not cover them, and no claim rests on
            // that board; it is left out rather than compared vacuously.
            if n == 5 && matches!(suicide, Suicide::RemoveOwn) {
                continue;
            }
            let across = superko_solve::separate::sweep(Dims::new(1, n), suicide, BUDGET)
                .expect("a small board");
            let down = superko_solve::separate::sweep(Dims::new(n, 1), suicide, BUDGET)
                .expect("a small board");
            // An unresolved root would be unresolved in both sweeps and the two
            // would still agree, so the comparison would say nothing about it.
            assert_eq!(
                across.unresolved, 0,
                "the 1x{n} sweep under {suicide} left roots unresolved at the budget"
            );
            // Every field, the node counts included; only the board differs.
            let turned = superko_solve::separate::Sweep {
                dims: across.dims,
                ..down
            };
            assert_eq!(
                across, turned,
                "the 1x{n} sweep and the {n}x1 sweep disagree under {suicide}"
            );
        }
    }
}
