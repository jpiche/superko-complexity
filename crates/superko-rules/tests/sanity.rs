//! `lean/SuperkoComplexity/Sanity.lean`, transliterated.
//!
//! One test per Lean theorem, named after it, on the same fixture. The Lean
//! checks are proved by `decide`, so the kernel verifies them; these are the
//! same facts asserted by a second route, which is what makes them evidence
//! that this crate and `Defs.lean` agree on the fixtures. They say nothing
//! about any board the fixtures do not cover.

use superko_rules::code::parse;
use superko_rules::config::{Dims, Point, Suicide};
use superko_rules::reference::{
    Color, Position, adj, area, chain, has_liberty, neighbors, playable_at, resolve,
};

fn board(rows: usize, cols: usize, text: &str) -> Position {
    parse(Dims::new(rows, cols), text).expect("fixture parses")
}

const fn pt(row: usize, col: usize) -> Point {
    Point::new(row, col)
}

// --- Board geometry ---------------------------------------------------------

#[test]
fn adj_orthogonal() {
    assert!(adj(pt(0, 0), pt(0, 1)));
}

#[test]
fn adj_not_diagonal() {
    assert!(!adj(pt(0, 0), pt(1, 1)));
}

#[test]
fn adj_irreflexive() {
    assert!(!adj(pt(0, 0), pt(0, 0)));
}

#[test]
fn adj_symmetric() {
    assert!(adj(pt(0, 1), pt(0, 0)));
}

#[test]
fn nbr_count_center() {
    let dims = Dims::new(3, 3);
    let count = dims.points().filter(|&q| adj(pt(1, 1), q)).count();
    assert_eq!(count, 4);
}

#[test]
fn nbr_count_corner() {
    let dims = Dims::new(3, 3);
    let count = dims.points().filter(|&q| adj(pt(0, 0), q)).count();
    assert_eq!(count, 2);
}

/// Not a `Sanity.lean` check: the enumeration `neighbors` performs is the
/// quantification over all points that Lean's `Adj` is used under, on every
/// board with `m * n <= 6`.
#[test]
fn adj_agrees_with_neighbors() {
    for (rows, cols) in boards_to_six() {
        let dims = Dims::new(rows, cols);
        for p in dims.points() {
            let quantified: Vec<Point> = dims.points().filter(|&q| adj(p, q)).collect();
            assert_eq!(neighbors(dims, p), quantified, "at {p} of {dims}");
        }
    }
}

fn boards_to_six() -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for rows in 1..=6 {
        for cols in 1..=6 {
            if rows * cols <= 6 {
                out.push((rows, cols));
            }
        }
    }
    out
}

// --- Chains -----------------------------------------------------------------

#[test]
fn chain_joins_adjacent() {
    assert!(chain(&board(2, 2, "XX/.."), pt(0, 0)).contains(&pt(0, 1)));
}

#[test]
fn chain_skips_diagonal() {
    assert!(!chain(&board(2, 2, "X./.X"), pt(0, 0)).contains(&pt(1, 1)));
}

#[test]
fn chain_respects_color() {
    assert!(!chain(&board(2, 2, "XO/.."), pt(0, 0)).contains(&pt(0, 1)));
}

// --- Liberties --------------------------------------------------------------

#[test]
fn no_liberty_when_surrounded() {
    assert!(!has_liberty(&board(2, 2, "XO/O."), pt(0, 0)));
}

#[test]
fn liberty_from_one_empty_neighbor() {
    assert!(has_liberty(&board(2, 2, "XO/.."), pt(0, 0)));
}

#[test]
fn liberty_is_shared_across_chain() {
    assert!(has_liberty(&board(3, 3, "XXO/O../..."), pt(0, 0)));
}

// --- Playing a move ---------------------------------------------------------

#[test]
fn play_captures() {
    let after = resolve(&board(3, 3, "OX./.../..."), Color::Black, pt(1, 0));
    assert_eq!(after.get(pt(0, 0)), None);
}

#[test]
fn suicide_is_illegal() {
    let b = board(3, 3, ".O./O../...");
    assert!(!playable_at(&b, Color::Black, pt(0, 0), Suicide::Forbid));
}

#[test]
fn same_point_legal_for_owner() {
    let b = board(3, 3, ".O./O../...");
    assert!(playable_at(&b, Color::White, pt(0, 0), Suicide::Forbid));
}

#[test]
fn capture_precedes_suicide_test() {
    let b = board(3, 3, ".OX/OX./...");
    assert!(playable_at(&b, Color::Black, pt(0, 0), Suicide::Forbid));
}

#[test]
fn occupied_not_playable() {
    let b = board(2, 2, "X./..");
    assert!(!playable_at(&b, Color::Black, pt(0, 0), Suicide::Forbid));
}

// --- Area scoring -----------------------------------------------------------

#[test]
fn area_empty_board() {
    let b = board(2, 2, "../..");
    assert_eq!(area(&b, Color::Black), 0);
    assert_eq!(area(&b, Color::White), 0);
}

#[test]
fn area_lone_stone() {
    let b = board(2, 2, "X./..");
    assert_eq!(area(&b, Color::Black), 4);
    assert_eq!(area(&b, Color::White), 0);
}

#[test]
fn area_dame_counts_for_neither() {
    let b = board(2, 2, "X./.O");
    assert_eq!(area(&b, Color::Black), 1);
    assert_eq!(area(&b, Color::White), 1);
}

#[test]
fn area_eye_and_partition() {
    let b = board(3, 3, ".XO/XOO/OOO");
    assert_eq!(area(&b, Color::Black), 3);
    assert_eq!(area(&b, Color::White), 6);
}
