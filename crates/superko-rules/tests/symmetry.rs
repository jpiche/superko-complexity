//! The symmetry maps are what they say, and the transition table commutes
//! with them.
//!
//! Two kinds of check. The first holds the maps of `superko_rules::symmetry`
//! to their definition: each board symmetry and the color swap is a bijection
//! on codes that moves stones without adding, removing or recoloring any (the
//! swap recolors every one), keeps a position legal exactly when it was, and
//! the board symmetries form a group of the order `docs/plans/solver-plan.md`
//! §2.2 as corrected names — 1, 2, 2, 4, 8, 8 on 1×1, 1×4, 4×1, 2×3, 2×2, 3×3.
//!
//! The second is the invariance the `board-symmetry` and `color-swap`
//! divergences rest on: for every code `c`, color `x`, point `p` and board
//! symmetry `s`,
//!
//! - `succ(s(c), x, s(p)) = s(succ(c, x, p))`,
//! - `playable(s(c), x, s(p)) = playable(c, x, p)`,
//! - `area(s(c), x) = area(c, x)`,
//!
//! and for the color swap,
//!
//! - `succ(swap(c), x.other, p) = swap(succ(c, x, p))`,
//! - `playable(swap(c), x.other, p) = playable(c, x, p)`,
//! - `area(swap(c), x.other) = area(c, x)`,
//!
//! checked exhaustively on every board of at most six points in both
//! orientations under both suicide conventions, and on 3×3 and 3×4 in an
//! ignored test for release. The table is a memo of the reference functions
//! (`tests/table.rs`), so this is a check of the reference functions at those
//! arguments. It is `computed` on the boards named and says nothing about any
//! other board.

use superko_rules::code::{PosCode, code_space, decode};
use superko_rules::config::{Dims, Suicide};
use superko_rules::reference::{Color, Position, has_liberty};
use superko_rules::symmetry::Symmetries;
use superko_rules::table::RuleTable;

/// Every board of at most six points, both orientations of each.
fn boards_to_six() -> Vec<Dims> {
    let mut out = Vec::new();
    for rows in 1..=6 {
        for cols in 1..=6 {
            if rows * cols <= 6 {
                out.push(Dims::new(rows, cols));
            }
        }
    }
    out
}

/// Whether no chain of the position lacks a liberty — the predicate
/// `superko_graph::census::is_legal` states, written here over the reference
/// function because this crate cannot depend on that one.
fn liberties(b: &Position) -> bool {
    b.dims()
        .points()
        .all(|p| b.get(p).is_none() || has_liberty(b, p))
}

#[test]
fn the_group_has_the_order_of_its_board() {
    for (rows, cols, order) in [
        (1, 1, 1),
        (1, 4, 2),
        (4, 1, 2),
        (2, 3, 4),
        (3, 2, 4),
        (2, 2, 8),
        (3, 3, 8),
        (1, 6, 2),
        (6, 1, 2),
    ] {
        let dims = Dims::new(rows, cols);
        let sym = Symmetries::new(dims).expect("a small board");
        assert_eq!(sym.order(), order, "{dims}");
        assert_eq!(sym.transform(0).name(), "identity", "{dims}");
        for g in 0..sym.order() {
            let t = sym.transform(g);
            assert!(
                !t.transpose || rows == cols,
                "{dims}: {} transposes a board that is not square",
                t.name()
            );
            for p in dims.points() {
                assert_eq!(sym.point(g, p), t.apply(dims, p), "{dims} {}", t.name());
            }
        }
    }
}

#[test]
fn a_board_past_the_table_budget_is_refused() {
    assert!(Symmetries::new(Dims::new(4, 4)).is_err());
}

/// Distinct elements are distinct maps of the grid, and the point maps are
/// permutations.
#[test]
fn the_elements_are_distinct_permutations_of_the_points() {
    for dims in boards_to_six().into_iter().chain([Dims::new(3, 3)]) {
        let sym = Symmetries::new(dims).expect("a small board");
        let maps: Vec<Vec<usize>> = (0..sym.order())
            .map(|g| dims.points().map(|p| dims.index(sym.point(g, p))).collect())
            .collect();
        for (g, map) in maps.iter().enumerate() {
            let mut sorted = map.clone();
            sorted.sort_unstable();
            assert_eq!(
                sorted,
                (0..dims.point_count()).collect::<Vec<_>>(),
                "{dims} element {g}"
            );
            for (h, other) in maps.iter().enumerate().skip(g + 1) {
                assert_ne!(map, other, "{dims}: elements {g} and {h} are one map");
            }
        }
    }
}

/// The flat tables are what the per-entry accessors say: `code_table` at
/// `g · code_count + c` is `code(g, c)`, and `point_table` at
/// `g · point_count + i` is the index of `point(g, i)`.
#[test]
fn the_flat_tables_agree_with_the_accessors() {
    for dims in boards_to_six().into_iter().chain([Dims::new(3, 3)]) {
        let sym = Symmetries::new(dims).expect("a small board");
        let codes = code_space(dims) as usize;
        let points = dims.point_count();
        assert_eq!(sym.code_count(), codes, "{dims}");
        assert_eq!(sym.code_table().len(), sym.order() * codes, "{dims}");
        assert_eq!(sym.point_table().len(), sym.order() * points, "{dims}");
        for raw in 0..code_space(dims) {
            let code = PosCode(raw);
            for g in 0..sym.order() {
                assert_eq!(
                    sym.code_table()[g * codes + raw as usize],
                    sym.code(g, code).0,
                    "{dims} element {g} code {code}"
                );
            }
        }
        for g in 0..sym.order() {
            for p in dims.points() {
                assert_eq!(
                    sym.point_table()[g * points + dims.index(p)],
                    dims.index(sym.point(g, p)),
                    "{dims} element {g}"
                );
            }
        }
    }
}

/// Every element's code map and the swap are bijections that keep the stone
/// count — per color for a board symmetry, exchanged for the swap — keep a
/// position's liberties, and move each stone where the point map sends it.
#[test]
fn the_code_maps_are_bijections_that_move_stones_and_keep_legality() {
    for dims in boards_to_six().into_iter().chain([Dims::new(3, 3)]) {
        let sym = Symmetries::new(dims).expect("a small board");
        let codes = code_space(dims) as usize;
        for g in 0..sym.order() {
            let mut hit = vec![false; codes];
            for raw in 0..code_space(dims) {
                let code = PosCode(raw);
                let image = sym.code(g, code);
                assert!(
                    !std::mem::replace(&mut hit[image.0 as usize], true),
                    "{dims} element {g}: two codes map to {image}"
                );
                let b = decode(dims, code);
                let sb = decode(dims, image);
                for p in dims.points() {
                    assert_eq!(sb.get(sym.point(g, p)), b.get(p), "{dims} element {g}");
                }
                for c in [Color::Black, Color::White] {
                    assert_eq!(sb.stone_count(c), b.stone_count(c));
                }
                assert_eq!(liberties(&sb), liberties(&b), "{dims} element {g}");
            }
        }
        let mut hit = vec![false; codes];
        for raw in 0..code_space(dims) {
            let code = PosCode(raw);
            let image = sym.swap(code);
            assert!(
                !std::mem::replace(&mut hit[image.0 as usize], true),
                "{dims} swap: two codes map to {image}"
            );
            let b = decode(dims, code);
            let sb = decode(dims, image);
            for p in dims.points() {
                assert_eq!(sb.get(p), b.get(p).map(Color::other), "{dims} swap");
            }
            assert_eq!(sb.stone_count(Color::Black), b.stone_count(Color::White));
            assert_eq!(sb.stone_count(Color::White), b.stone_count(Color::Black));
            assert_eq!(liberties(&sb), liberties(&b), "{dims} swap");
        }
    }
}

/// Composition is closed and agrees with composing the code maps, inverses
/// undo, the identity is element 0, the swap is an involution, and the swap
/// commutes with every board symmetry.
#[test]
fn the_maps_form_a_group_and_the_swap_commutes_with_it() {
    for dims in boards_to_six().into_iter().chain([Dims::new(3, 3)]) {
        let sym = Symmetries::new(dims).expect("a small board");
        let order = sym.order();
        for raw in 0..code_space(dims) {
            let code = PosCode(raw);
            assert_eq!(
                sym.code(0, code),
                code,
                "{dims}: element 0 is not the identity"
            );
            assert_eq!(
                sym.swap(sym.swap(code)),
                code,
                "{dims}: swap is not an involution"
            );
            for a in 0..order {
                let inv = sym.inverse(a);
                assert_eq!(sym.code(inv, sym.code(a, code)), code, "{dims} {a}");
                assert_eq!(sym.code(a, sym.code(inv, code)), code, "{dims} {a}");
                assert_eq!(
                    sym.code(a, sym.swap(code)),
                    sym.swap(sym.code(a, code)),
                    "{dims}: element {a} does not commute with the swap"
                );
                for b in 0..order {
                    let ab = sym.compose(a, b);
                    assert!(ab < order);
                    assert_eq!(
                        sym.code(ab, code),
                        sym.code(a, sym.code(b, code)),
                        "{dims}: compose({a}, {b}) is not b then a"
                    );
                }
            }
        }
        for a in 0..order {
            assert_eq!(sym.compose(a, sym.inverse(a)), 0, "{dims} {a}");
            assert_eq!(sym.compose(sym.inverse(a), a), 0, "{dims} {a}");
            for p in dims.points() {
                for b in 0..order {
                    assert_eq!(
                        sym.point(sym.compose(a, b), p),
                        sym.point(a, sym.point(b, p)),
                        "{dims}"
                    );
                }
            }
        }
    }
}

/// The invariance itself, at every code, color, point and map of one board
/// under one suicide convention. Returns the number of `(code, color, point,
/// map)` checks made, the swap counted as one map, so a caller can pin it.
fn table_commutes(dims: Dims, suicide: Suicide) -> u64 {
    let table = RuleTable::build(dims, suicide).expect("a board within the table budget");
    let sym = Symmetries::new(dims).expect("a board within the table budget");
    let mut checks = 0u64;
    for raw in 0..code_space(dims) {
        let code = PosCode(raw);
        for x in [Color::Black, Color::White] {
            let sc = sym.swap(code);
            assert_eq!(
                table.area(sc, x.other()),
                table.area(code, x),
                "{dims} {suicide}: area of {x} at {code} under the swap"
            );
            for g in 0..sym.order() {
                assert_eq!(
                    table.area(sym.code(g, code), x),
                    table.area(code, x),
                    "{dims} {suicide}: area of {x} at {code} under element {g}"
                );
            }
            for p in dims.points() {
                let succ = table.succ(code, x, p);
                let playable = table.playable(code, x, p);
                assert_eq!(
                    table.succ(sc, x.other(), p),
                    sym.swap(succ),
                    "{dims} {suicide}: successor of {x} at {p} on {code} under the swap"
                );
                assert_eq!(
                    table.playable(sc, x.other(), p),
                    playable,
                    "{dims} {suicide}: playability of {x} at {p} on {code} under the swap"
                );
                checks += 1;
                for g in 0..sym.order() {
                    let gc = sym.code(g, code);
                    let gp = sym.point(g, p);
                    assert_eq!(
                        table.succ(gc, x, gp),
                        sym.code(g, succ),
                        "{dims} {suicide}: successor of {x} at {p} on {code} under element {g}"
                    );
                    assert_eq!(
                        table.playable(gc, x, gp),
                        playable,
                        "{dims} {suicide}: playability of {x} at {p} on {code} under element {g}"
                    );
                    checks += 1;
                }
            }
        }
    }
    checks
}

/// Every board of at most six points, both orientations, both conventions.
/// The count of checks is pinned per board: `3^(m·n)` codes × 2 colors ×
/// `m·n` points × (group order + 1), so a map list that came back short
/// fails rather than checking less.
#[test]
fn the_table_commutes_with_every_map_on_boards_of_six_points() {
    for dims in boards_to_six() {
        let sym = Symmetries::new(dims).expect("a small board");
        let expected =
            u64::from(code_space(dims)) * 2 * dims.point_count() as u64 * (sym.order() as u64 + 1);
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            assert_eq!(table_commutes(dims, suicide), expected, "{dims} {suicide}");
        }
    }
}

/// 3×3 (eight symmetries, the quarter turns among them) and 3×4 (four), both
/// conventions. Too slow for a debug build.
///
/// `cargo test --release -p superko-rules --test symmetry -- --ignored the_table_commutes_with_every_map_on_3x3_and_3x4`
#[test]
#[ignore = "release only: builds the 3x4 transition table"]
fn the_table_commutes_with_every_map_on_3x3_and_3x4() {
    for (rows, cols, order) in [(3, 3, 8u64), (3, 4, 4)] {
        let dims = Dims::new(rows, cols);
        let expected = u64::from(code_space(dims)) * 2 * dims.point_count() as u64 * (order + 1);
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            assert_eq!(table_commutes(dims, suicide), expected, "{dims} {suicide}");
        }
    }
}
