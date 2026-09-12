//! The two engines, the two recursions and the two orders, held equal.
//!
//! Four separate agreements, because each catches a different way the fast
//! path could be wrong:
//!
//! 1. **Engines.** [`superko_solve::naive`] clones a state per node, runs the
//!    rule predicates themselves and prunes nothing. The fast path reads a
//!    memo of those predicates, makes and unmakes a dense archive, and cuts.
//!    They share no control flow. Bounded to `m · n <= 3`: the naive engine
//!    visits the whole tree, which on four points is billions of nodes.
//!
//! 2. **Recursions.** The score search and the verdict search are written
//!    separately and neither is computed from the other, so holding
//!    `black wins at komi floor k` equal to `value > k` at every komi floor
//!    checks the threshold agreement the crate docs mark `computed` rather
//!    than assuming it.
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
