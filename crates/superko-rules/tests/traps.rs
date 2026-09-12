//! The traps: the errors a hand-written engine actually makes.
//!
//! Each of these is a place where a plausible implementation differs from
//! `Defs.lean` and where the difference is invisible in a total — a count that
//! comes out wrong with no symptom but the number. They are tests rather than
//! claims: passing means this crate does not make that error on that fixture.

use superko_rules::code::parse;
use superko_rules::config::{Dims, Point, Repetition, Suicide};
use superko_rules::reference::{
    Color, Move, Position, Rat, State, chain, clear, ended, has_liberty, legality, margin, permits,
    psk, resolve, resolve_remove_own, ssk, step, winner, winner_z,
};
use superko_rules::reference::{Refusal, Situation};

fn board(rows: usize, cols: usize, text: &str) -> Position {
    parse(Dims::new(rows, cols), text).expect("fixture parses")
}

const fn pt(row: usize, col: usize) -> Point {
    Point::new(row, col)
}

// --- Capture ----------------------------------------------------------------

/// Two neighbors of the played point belong to one dying chain. An engine that
/// walks the played point's neighbors and removes a chain per neighbor removes
/// this chain twice, and an engine that stops at the first dying chain removes
/// it once and misses nothing — but the two are indistinguishable here unless
/// the whole chain comes off exactly once.
#[test]
fn two_neighbors_of_the_played_point_in_one_dying_chain() {
    let b = board(3, 3, "OOX/O../X..");
    assert_eq!(
        chain(&b, pt(0, 0)).len(),
        3,
        "the three white stones are one chain"
    );
    let after = resolve(&b, Color::Black, pt(1, 1));
    assert_eq!(after.get(pt(0, 0)), None);
    assert_eq!(after.get(pt(0, 1)), None);
    assert_eq!(after.get(pt(1, 0)), None);
    assert_eq!(after.get(pt(1, 1)), Some(Color::Black));
    assert_eq!(after.stone_count(Color::White), 0);
}

/// Two separate chains die on one play.
#[test]
fn two_chains_die_at_once() {
    let b = board(3, 3, "XOX/O../X..");
    assert!(
        !chain(&b, pt(0, 1)).contains(&pt(1, 0)),
        "the two white stones are separate chains"
    );
    let after = resolve(&b, Color::Black, pt(1, 1));
    assert_eq!(after.get(pt(0, 1)), None);
    assert_eq!(after.get(pt(1, 0)), None);
    assert_eq!(after.stone_count(Color::White), 0);
}

/// `clear` reads the original board at every point, so the order the board is
/// swept in cannot change the answer. Checked here against three sweep orders
/// on every position of three small boards; that is `computed` for those
/// boards, and the reason it holds is that the removals do not see each other.
#[test]
fn clear_is_order_independent() {
    for (rows, cols) in [(2usize, 2usize), (2, 3), (1, 4)] {
        let dims = Dims::new(rows, cols);
        let row_major: Vec<Point> = dims.points().collect();
        let reversed: Vec<Point> = {
            let mut v = row_major.clone();
            v.reverse();
            v
        };
        let column_major: Vec<Point> = (0..cols)
            .flat_map(|col| (0..rows).map(move |row| pt(row, col)))
            .collect();
        for code in 0..superko_rules::code::code_space(dims) {
            let b = superko_rules::code::decode(dims, superko_rules::PosCode(code));
            for c in [Color::Black, Color::White] {
                let want = clear(&b, c);
                for order in [&row_major, &reversed, &column_major] {
                    assert_eq!(
                        want,
                        clear_in_order(&b, c, order),
                        "clearing {c} on {}",
                        superko_rules::code::render(&b)
                    );
                }
            }
        }
    }
}

/// `clear` is not the in-place sweep, and the difference is not hypothetical:
/// on the 2x2 board of four black stones, `Superko.clear` takes all four off,
/// while a sweep that removes as it goes leaves three standing — the first
/// removal hands the rest a liberty.
///
/// An engine written the second way scores this position 0 to 0 instead of 0
/// to 4, and nothing about the count it produces says which function it ran.
#[test]
fn clear_reads_the_original_board() {
    let b = board(2, 2, "XX/XX");
    let swept = clear(&b, Color::Black);
    assert_eq!(swept.stone_count(Color::Black), 0, "every stone comes off");

    let in_place = clear_in_place(&b, Color::Black);
    assert_eq!(in_place.stone_count(Color::Black), 3);
    assert_ne!(swept, in_place);
}

/// The other function: remove as the sweep goes, so a later point is judged
/// against a board earlier removals have changed. It lives in the test because
/// it is not a rule of Go and no rule here may call it.
fn clear_in_place(b: &Position, c: Color) -> Position {
    let mut out = b.clone();
    for q in b.dims().points() {
        if out.get(q) == Some(c) && !has_liberty(&out, q) {
            out.set(q, None);
        }
    }
    out
}

/// `clear` with the sweep written out: the test of what comes off reads `b`,
/// the original, whatever order the points are visited in.
fn clear_in_order(b: &Position, c: Color, order: &[Point]) -> Position {
    let mut out = b.clone();
    for &q in order {
        if b.get(q) == Some(c) && !has_liberty(b, q) {
            out.set(q, None);
        }
    }
    out
}

// --- The root seed and the two rules on 1x1 ---------------------------------

/// A single-stone play on 1x1 under `RemoveOwn` takes the stone straight back
/// off, so the position after the play is the empty board again. Under
/// situational superko the move is legal, because the *situation* it recreates
/// has White to move and the archived one has Black. Under positional superko
/// it is a repetition, because the root seed put the empty board in the
/// archive — and that is what holds the published 1x1 game count at 1.
#[test]
fn single_stone_suicide_on_one_by_one() {
    let dims = Dims::new(1, 1);
    let empty = Position::empty(dims);
    let st = State::start(&empty, Color::Black);
    let mv = Move::Play(pt(0, 0));

    let after = resolve_remove_own(&empty, Color::Black, pt(0, 0));
    assert_eq!(after, empty, "the stone comes straight back off");

    assert!(ssk(&st, mv, Suicide::RemoveOwn), "legal under SSK");
    assert!(!psk(&st, mv, Suicide::RemoveOwn), "a repetition under PSK");
    assert_eq!(
        legality(&st, mv, Repetition::Psk, Suicide::RemoveOwn),
        Err(Refusal::Repetition)
    );

    assert!(
        !ssk(&st, mv, Suicide::Forbid),
        "under the rule Defs.lean states, the play is suicide and never reaches the archive"
    );
    assert_eq!(
        legality(&st, mv, Repetition::Ssk, Suicide::Forbid),
        Err(Refusal::Suicide)
    );
}

// --- Passes -----------------------------------------------------------------

/// A pass is legal though its situation is archived. Two passes from the root
/// return to the root situation, which `start` seeded, and the move is still
/// legal under both rules — the exemption is OPEN-1, claim C-18, `cited`.
#[test]
fn pass_is_legal_though_its_situation_is_archived() {
    let dims = Dims::new(2, 2);
    let st0 = State::start(&Position::empty(dims), Color::Black);
    let st1 = step(&st0, Move::Pass, Suicide::Forbid);
    let back: Situation = st1.now().clone();
    assert!(st1.seen().contains(&back));

    let st2 = step(&st1, Move::Pass, Suicide::Forbid);
    assert!(
        st1.seen().contains(st2.now()),
        "the pass re-entered a seen situation"
    );
    for rep in [Repetition::Ssk, Repetition::Psk] {
        assert!(permits(&st1, Move::Pass, rep, Suicide::Forbid));
        assert!(legality(&st1, Move::Pass, rep, Suicide::Forbid).is_ok());
    }
    assert!(ended(&st2));
}

/// `step` counts passes and does not saturate: `Superko.step` writes
/// `passes + 1`, and `Ended` reads `2 <= passes`. An engine that clamped the
/// counter at 2 would agree on every ending and could still differ wherever
/// the count is read for anything else.
#[test]
fn step_does_not_saturate_passes() {
    let dims = Dims::new(2, 2);
    let mut st = State::start(&Position::empty(dims), Color::Black);
    for expected in 1..=5u32 {
        st = step(&st, Move::Pass, Suicide::Forbid);
        assert_eq!(st.passes(), expected);
    }
    assert!(ended(&st));

    let played = step(&st, Move::Play(pt(0, 0)), Suicide::Forbid);
    assert_eq!(played.passes(), 0, "a play resets the counter");
}

/// The archive holds the situation now, on every move including a pass.
#[test]
fn step_archives_on_every_move() {
    let dims = Dims::new(2, 2);
    let st0 = State::start(&Position::empty(dims), Color::Black);
    assert!(st0.seen().contains(st0.now()), "the root is seeded");
    for mv in [Move::Pass, Move::Play(pt(0, 0))] {
        let st = step(&st0, mv, Suicide::Forbid);
        assert!(st.seen().contains(st.now()));
        assert_eq!(st.seen().len(), 2);
    }
}

// --- Score ------------------------------------------------------------------

/// The winner is exact over a rational komi, and a tie goes to White. The
/// integer test at the floor of komi gives the same verdict, which is
/// `Superko.winnerZ_eq_winner`, `proved`.
#[test]
fn winner_is_exact_over_rational_komi() {
    let lone = board(2, 2, "X./..");
    let split = board(2, 2, "X./.O");

    assert_eq!(margin(&lone, Rat::from_int(4)), Rat::ZERO);
    assert_eq!(
        winner(&lone, Rat::from_int(4)),
        Color::White,
        "a tie goes to White"
    );
    assert_eq!(winner(&lone, Rat::new(7, 2)), Color::Black);
    assert_eq!(winner(&lone, Rat::new(9, 2)), Color::White);
    assert_eq!(winner(&lone, Rat::from_int(3)), Color::Black);

    assert_eq!(margin(&split, Rat::ZERO), Rat::ZERO);
    assert_eq!(
        winner(&split, Rat::ZERO),
        Color::White,
        "a tie goes to White"
    );
    assert_eq!(winner(&split, Rat::new(-1, 2)), Color::Black);

    for komi in [
        Rat::ZERO,
        Rat::from_int(3),
        Rat::from_int(4),
        Rat::new(7, 2),
        Rat::new(9, 2),
        Rat::new(-1, 2),
        Rat::new(-7, 3),
    ] {
        for b in [&lone, &split] {
            assert_eq!(
                winner_z(b, komi.floor()),
                winner(b, komi),
                "at komi {komi} on {}",
                superko_rules::code::render(b)
            );
        }
    }
}

// --- The board is part of a position ----------------------------------------

/// `Superko.Position m n` is indexed by the board, so `Position 1 4` and
/// `Position 2 2` are different types and Lean never compares them. Here the
/// dimensions are runtime values, and the four cells of the empty 1x4 board are
/// the same four cells as the empty 2x2 board's. They are still different
/// positions: the two boards have different adjacency, different areas at some
/// positions, and different game counts, so an equality that read the cells
/// alone would let a situation of one board answer for a situation of the
/// other wherever a `State`'s archive was handed a foreign board.
#[test]
fn a_position_is_not_equal_to_one_of_another_board() {
    let wide = Position::empty(Dims::new(1, 4));
    let square = Position::empty(Dims::new(2, 2));
    assert_eq!(wide.cells(), square.cells(), "the cells are the same");
    assert_ne!(wide, square, "the boards are not");
    assert_ne!(
        Situation {
            board: wide.clone(),
            to_move: Color::Black,
        },
        Situation {
            board: square.clone(),
            to_move: Color::Black,
        }
    );

    // The two boards really do differ at these cells: on 1x4 the two ends are
    // not adjacent, on 2x2 every point has two neighbors.
    let stones = board(1, 4, "XX..");
    let folded = board(2, 2, "XX/..");
    assert_eq!(stones.cells(), folded.cells());
    assert_eq!(chain(&stones, pt(0, 0)).len(), 2);
    assert_eq!(chain(&folded, pt(0, 0)).len(), 2);
    assert_ne!(stones, folded);
}
