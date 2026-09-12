//! The legal-position counts and the position-graph census.
//!
//! Neither quantity has any history in it, so both check chains, liberties and
//! capture without the repetition rule in the way. They are the cheapest
//! independent quantities this workspace can be held to.
//!
//! `L(m, n)` is Tromp–Farnebäck's legal-position table (C-23, `cited`). The
//! position-graph figures for 1×2 and 1×3 are the paper's Figures 1 and 2,
//! drawn under its suicide-permitting rules with self-loops excluded (`cited`);
//! the remaining figures, and every figure under `Defs.lean`'s
//! suicide-forbidding rule, are the scratch enumeration of 2026-09-11 recorded
//! in `docs/plans/rules-mirror-plan.md` (`computed`, and reproduced here by a
//! second route).

use superko_graph::census::{GraphCensus, legal_positions, position_graph};
use superko_rules::config::{Dims, Suicide};

/// `L(1, 1)` through `L(1, 8)`, then the two- and three-row boards.
#[test]
fn legal_position_counts() {
    let line = [1u64, 5, 15, 41, 113, 313, 867, 2401];
    for (i, expected) in line.into_iter().enumerate() {
        let cols = i + 1;
        assert_eq!(
            legal_positions(Dims::new(1, cols)),
            expected,
            "L(1, {cols})"
        );
    }
    assert_eq!(legal_positions(Dims::new(2, 2)), 57, "L(2, 2)");
    assert_eq!(legal_positions(Dims::new(2, 3)), 489, "L(2, 3)");
    assert_eq!(legal_positions(Dims::new(3, 3)), 12675, "L(3, 3)");
}

/// A board and its transpose have the same legal positions, since neither
/// chains nor liberties see the orientation. This is not an independent check
/// of the count — it is a check that `Dims` is not confusing rows with columns,
/// which a square board could never reveal.
#[test]
fn legal_positions_do_not_depend_on_orientation() {
    for (rows, cols) in [(1, 4), (2, 3), (1, 6)] {
        assert_eq!(
            legal_positions(Dims::new(rows, cols)),
            legal_positions(Dims::new(cols, rows)),
            "{rows}x{cols} and its transpose"
        );
    }
}

fn census(rows: usize, cols: usize, suicide: Suicide) -> GraphCensus {
    position_graph(Dims::new(rows, cols), suicide)
}

/// Under `Defs.lean`'s rule, where suicide is illegal, no play on the boards
/// below returns the position it was made from — the self-loop count is zero
/// on all five (`computed`).
///
/// The reason to expect that in general is that the played stone survives and
/// the board has one more stone or fewer opposing ones, so the position
/// changes. That argument is `folklore` here: it is not proved, in Lean or
/// anywhere else in this project, and the assertion below is a check on five
/// boards and not a proof of it.
#[test]
fn position_graph_under_defs_lean() {
    let expected = [
        ((1, 2), 5u64, 8u64),
        ((1, 3), 15, 36),
        ((1, 4), 41, 128),
        ((2, 2), 57, 184),
        ((2, 3), 489, 2220),
    ];
    for ((rows, cols), nodes, edges) in expected {
        let c = census(rows, cols, Suicide::Forbid);
        assert_eq!(c.nodes, nodes, "{rows}x{cols} nodes");
        assert_eq!(c.edges, edges, "{rows}x{cols} edges");
        assert_eq!(c.self_loops, 0, "{rows}x{cols} self-loops");
    }
}

/// Under the paper's suicide-permitting rule, with self-loops counted apart.
#[test]
fn position_graph_under_remove_own() {
    let expected = [
        ((1, 2), 5u64, 12u64, 0u64),
        ((1, 3), 15, 42, 4),
        ((1, 4), 41, 144, 16),
        ((2, 2), 57, 192, 8),
        ((2, 3), 489, 2312, 116),
    ];
    for ((rows, cols), nodes, edges, self_loops) in expected {
        let c = census(rows, cols, Suicide::RemoveOwn);
        assert_eq!(c.nodes, nodes, "{rows}x{cols} nodes");
        assert_eq!(c.edges, edges, "{rows}x{cols} edges");
        assert_eq!(c.self_loops, self_loops, "{rows}x{cols} self-loops");
    }
}

/// The node count of the position graph is the legal-position count, by two
/// routes that share only `is_legal`.
#[test]
fn nodes_are_the_legal_positions() {
    for (rows, cols) in [(1, 3), (1, 4), (2, 2), (2, 3)] {
        let dims = Dims::new(rows, cols);
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            assert_eq!(position_graph(dims, suicide).nodes, legal_positions(dims));
        }
    }
}

/// Permitting suicide adds edges and removes none, on the five boards below
/// (`computed`).
///
/// The reason to expect that in general: every play the stricter convention
/// permits resolves the same way under the looser one, because the second
/// `clear` finds nothing to remove when the played chain has a liberty. That
/// argument is `folklore` — it is not proved anywhere in this project, and the
/// assertion below is a check on five boards.
#[test]
fn remove_own_only_adds_edges() {
    for (rows, cols) in [(1, 2), (1, 3), (1, 4), (2, 2), (2, 3)] {
        let forbid = census(rows, cols, Suicide::Forbid);
        let remove = census(rows, cols, Suicide::RemoveOwn);
        assert!(
            remove.edges >= forbid.edges,
            "{rows}x{cols}: permitting suicide lost an edge"
        );
        assert!(remove.self_loops >= forbid.self_loops);
    }
}
