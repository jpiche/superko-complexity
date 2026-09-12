//! The table is a memo of the transliteration, and this re-derives it.
//!
//! `RuleTable::build` calls the reference functions; that its output is what
//! those functions say is `computed` for the boards checked here and is not
//! proved. Nothing else in the workspace may read a table entry and call it a
//! rule.

use superko_rules::code::{PosCode, code_space, decode, encode, render};
use superko_rules::config::{Dims, Suicide};
use superko_rules::reference::{Color, area, playable_at, resolve_under};
use superko_rules::table::{MAX_TABLE_POINTS, RuleTable};

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

#[test]
fn table_is_its_generator() {
    for (rows, cols) in boards_to_six() {
        let dims = Dims::new(rows, cols);
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let table = RuleTable::build(dims, suicide).expect("within the point budget");
            assert_eq!(table.dims(), dims);
            assert_eq!(table.suicide(), suicide);
            assert_eq!(table.code_count(), code_space(dims) as usize);

            for raw in 0..code_space(dims) {
                let code = PosCode(raw);
                let b = decode(dims, code);
                for c in [Color::Black, Color::White] {
                    assert_eq!(
                        table.area(code, c),
                        u32::try_from(area(&b, c)).expect("area fits"),
                        "area of {c} on {} ({dims})",
                        render(&b)
                    );
                    for p in dims.points() {
                        assert_eq!(
                            table.succ(code, c, p),
                            encode(&resolve_under(&b, c, p, suicide)),
                            "successor of {c} at {p} on {} ({dims}, {suicide})",
                            render(&b)
                        );
                        assert_eq!(
                            table.playable(code, c, p),
                            playable_at(&b, c, p, suicide),
                            "playability of {c} at {p} on {} ({dims}, {suicide})",
                            render(&b)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn table_refuses_a_board_past_the_budget() {
    let dims = Dims::new(4, 4);
    let refusal = RuleTable::build(dims, Suicide::Forbid).expect_err("16 points is past the cap");
    assert_eq!(refusal.dims, dims);
    assert_eq!(refusal.limit, MAX_TABLE_POINTS);
    assert!(refusal.to_string().contains("at most 12"));
}

#[test]
fn table_covers_occupied_points_because_resolve_is_total() {
    let dims = Dims::new(2, 2);
    let table = RuleTable::build(dims, Suicide::Forbid).expect("within the point budget");
    let b = decode(dims, PosCode(1));
    let p = dims.point_at(0);
    assert_eq!(b.get(p), Some(Color::Black));
    assert!(!table.playable(PosCode(1), Color::White, p));
    assert_eq!(
        table.succ(PosCode(1), Color::White, p),
        encode(&resolve_under(&b, Color::White, p, Suicide::Forbid))
    );
}
