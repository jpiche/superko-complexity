//! The situation graph's component structure, and the bound it puts on what
//! the forward-cone prune of C-42 can remove.
//!
//! These are `computed` figures. What they establish is that a proved theorem
//! is not a compression — not anything about the theorem's truth.

use superko_graph::scc::{SccCensus, situation_graph};
use superko_rules::config::{Dims, Suicide};

fn legal(rows: usize, cols: usize) -> SccCensus {
    situation_graph(Dims::new(rows, cols), Suicide::Forbid, true)
}

/// The headline: over the positions play can reach, the graph is the empty
/// board's pass cycle and one giant component, two deep, on every board from
/// 1×3 up. So the cone of any non-empty situation is everything, and the
/// prune of `Compress.winsFor_seen_inter_cone` removes at most two entries
/// from an archive that may hold `2 · 3^(m·n)`.
#[test]
fn two_components_from_1x3_up() {
    for (rows, cols) in [
        (1, 3),
        (1, 4),
        (1, 5),
        (1, 6),
        (1, 7),
        (2, 2),
        (2, 3),
        (2, 4),
        (3, 3),
    ] {
        let c = legal(rows, cols);
        assert_eq!(
            c.components, 2,
            "{rows}x{cols}: expected the empty cycle and one giant component"
        );
        assert_eq!(c.empty_component, 2, "{rows}x{cols}: empty component size");
        assert_eq!(c.depth, 2, "{rows}x{cols}: condensation depth");
        assert_eq!(
            c.outside_largest, 2,
            "{rows}x{cols}: the cone prune can remove at most this many"
        );
        assert_eq!(c.trivial, 0, "{rows}x{cols}: no component of one vertex");
    }
}

/// The two exceptions, kept because they are real. On 1×1 no stone can ever be
/// played — a lone stone has no liberty — so the whole graph is the empty
/// board's pass cycle. On 1×2 the giant component splits: from `X.` only White
/// can move, and only to `.O`, from which only Black can move, and only back;
/// the mirrored pair `.X` / `O.` is a second 2-cycle that the first never
/// reaches.
#[test]
fn the_two_small_exceptions() {
    let c = legal(1, 1);
    assert_eq!((c.situations, c.components, c.largest), (2, 1, 2));

    let c = legal(1, 2);
    assert_eq!(c.situations, 10);
    assert_eq!(c.components, 3);
    assert_eq!(c.largest, 4);
    assert_eq!(c.empty_component, 2);
    assert_eq!(c.depth, 2);
}

/// Vertex counts are `2 · L(m,n)` with `L` the legal-position count the census
/// module reproduces against Tromp–Farnebäck (C-23).
#[test]
fn vertices_are_twice_the_legal_positions() {
    for (rows, cols, legal_positions) in [
        (1, 1, 1),
        (1, 2, 5),
        (1, 3, 15),
        (1, 4, 41),
        (1, 5, 113),
        (1, 6, 313),
        (1, 7, 867),
        (2, 2, 57),
        (2, 3, 489),
        (3, 3, 12675),
    ] {
        assert_eq!(
            legal(rows, cols).situations,
            2 * legal_positions,
            "{rows}x{cols}"
        );
    }
}

/// Every situation has the pass edge, so the graph has no sink and the edge
/// count is at least the vertex count.
#[test]
fn the_pass_edge_is_unconditional() {
    for (rows, cols) in [(1, 3), (2, 2), (2, 3), (3, 3)] {
        let c = legal(rows, cols);
        assert!(c.edges >= c.situations, "{rows}x{cols}");
    }
}

/// Over every coloring — `Defs.lean`'s `Position m n`, illegal ones included —
/// the picture below the illegal positions is unchanged: they are sources that
/// no play reaches, so they arrive as extra components and the largest stays
/// the giant one.
#[test]
fn illegal_colorings_are_sources() {
    for (rows, cols) in [(1, 3), (1, 4), (2, 2), (2, 3)] {
        let all = situation_graph(Dims::new(rows, cols), Suicide::Forbid, false);
        let only = legal(rows, cols);
        assert_eq!(
            all.largest, only.largest,
            "{rows}x{cols}: the giant component is the same"
        );
        assert!(
            all.components > only.components,
            "{rows}x{cols}: illegal colorings add components"
        );
        assert_eq!(
            all.situations - all.largest,
            all.outside_largest,
            "{rows}x{cols}"
        );
    }
}

/// `Compress.sitStep_empty_board` says a play never produces the empty board,
/// so the empty situation has in-degree zero under play edges and its component
/// is exactly the pass 2-cycle. That is a theorem for every board; this checks
/// the mirror agrees where it can count.
#[test]
fn the_empty_board_is_its_own_component() {
    for (rows, cols) in [(1, 1), (1, 2), (1, 3), (1, 4), (2, 2), (2, 3), (3, 3)] {
        let c = legal(rows, cols);
        assert_eq!(
            c.empty_component, 2,
            "{rows}x{cols}: the empty board's component is its pass cycle"
        );
    }
}

/// The suicide convention *does* change the structure, and the reason is the
/// theorem: `Compress.sitStep_empty_board` rests on `resolve_self`, that the
/// played stone survives, which holds because `Defs.lean`'s `resolve` clears
/// `c.other` only. Under `RemoveOwn` — Tromp–Farnebäck's convention, with no
/// counterpart in `Defs.lean` — a self-capture can take the played stone back
/// off, the empty board acquires an incoming edge, and the graph collapses to a
/// single component on every board censused. There the cone prunes nothing at
/// all rather than at most two.
#[test]
fn permitting_suicide_collapses_the_graph_to_one_component() {
    for (rows, cols) in [(1, 2), (1, 3), (1, 4), (1, 5), (2, 2), (2, 3), (3, 3)] {
        let dims = Dims::new(rows, cols);
        let remove_own = situation_graph(dims, Suicide::RemoveOwn, true);
        assert_eq!(remove_own.components, 1, "{rows}x{cols}");
        assert_eq!(remove_own.outside_largest, 0, "{rows}x{cols}");
        assert_eq!(remove_own.depth, 1, "{rows}x{cols}");
        assert_eq!(
            remove_own.empty_component, remove_own.situations,
            "{rows}x{cols}: the empty board joins the giant component"
        );

        // Same vertex set, strictly more edges: permitting suicide only adds
        // moves.
        let forbid = situation_graph(dims, Suicide::Forbid, true);
        assert_eq!(forbid.situations, remove_own.situations, "{rows}x{cols}");
        assert!(forbid.edges < remove_own.edges, "{rows}x{cols}");
    }
}
