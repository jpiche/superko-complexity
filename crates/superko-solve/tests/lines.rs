//! Searches after a line of moves, and the lines along which the two six-point
//! witnesses of C-56 separate the rules.
//!
//! [`Solver::solve_line`] and [`Solver::decide_line`] seat a root, make each
//! move of a line with its situation archived, and search from there. Three
//! checks that they search the state the line leaves:
//!
//! 1. **The empty line is the root.** On every root of 1×3 and 2×2, under
//!    both rules, the value and each color's verdict at every komi floor equal
//!    [`Solver::solve_root`]'s and [`Solver::decide_root`]'s.
//! 2. **Against the naive engine.** On every root of 1×3 under both rules,
//!    after every line of at most two moves the rules permit, the value equals
//!    [`naive::value`] of the reference state the line steps to.
//! 3. **The witness lines.** The line the SSK winner needs at each six-point
//!    witness, followed into the part of the tree where PSK says otherwise,
//!    ends in a play SSK permits and PSK refuses, and the values and verdicts
//!    along it are the ones `notebook/2026-09-13-why-the-witnesses-separate.md`
//!    records. And the 1×6 witness reached from the empty board by a game
//!    follows a White pass, after which Black wins by passing under either
//!    rule.
//!
//! What this establishes is `computed`, on the boards and lines named.

use superko_rules::code::{PosCode, code_space, encode, parse};
use superko_rules::config::{Dims, Point, Repetition, Suicide};
use superko_rules::reference::{
    Color, Move, Position, State, all_moves, ended, permits, psk, ssk, step,
};
use superko_rules::table::RuleTable;
use superko_solve::naive;
use superko_solve::search::Solver;

const RULES: [Repetition; 2] = [Repetition::Psk, Repetition::Ssk];
const SUICIDE: Suicide = Suicide::Forbid;

fn floors(dims: Dims) -> std::ops::RangeInclusive<i64> {
    let span = i64::try_from(dims.point_count()).expect("point count fits an i64");
    (-span - 1)..=(span + 1)
}

const fn play(row: usize, col: usize) -> Move {
    Move::Play(Point::new(row, col))
}

/// Step a reference state along a line, asserting each move is permitted.
fn walk(root: &Position, to_move: Color, rep: Repetition, line: &[Move]) -> State {
    let mut st = State::start(root, to_move);
    for &mv in line {
        assert!(!ended(&st), "the line moves from an ended state");
        assert!(
            permits(&st, mv, rep, SUICIDE),
            "the line holds a refused move: {mv}"
        );
        st = step(&st, mv, SUICIDE);
    }
    st
}

#[test]
fn an_empty_line_is_the_root() {
    let mut compared = 0u64;
    for dims in [Dims::new(1, 3), Dims::new(2, 2)] {
        let table = RuleTable::build(dims, SUICIDE).expect("small board");
        for rep in RULES {
            let mut solver = Solver::new(&table, rep);
            for code in 0..code_space(dims) {
                let code = PosCode(code);
                for to_move in [Color::Black, Color::White] {
                    let root = solver.solve_root(code, to_move);
                    let line = solver.solve_line(code, to_move, &[]);
                    assert_eq!(line, root, "{dims} {rep:?} {code:?} {to_move:?}");
                    for k in floors(dims) {
                        for c in [Color::Black, Color::White] {
                            let root = solver.decide_root(code, to_move, k, c);
                            let line = solver.decide_line(code, to_move, &[], k, c);
                            assert_eq!(line, root, "{dims} {rep:?} {code:?} {to_move:?} {k} {c:?}");
                        }
                    }
                    compared += 1;
                }
            }
        }
    }
    assert_eq!(compared, 2 * 2 * (27 + 81));
}

#[test]
fn a_line_agrees_with_the_naive_value_after_stepping() {
    let dims = Dims::new(1, 3);
    let table = RuleTable::build(dims, SUICIDE).expect("small board");
    let mut compared = 0u64;
    for rep in RULES {
        let mut solver = Solver::new(&table, rep);
        for code in 0..code_space(dims) {
            let code = PosCode(code);
            let root = superko_rules::code::decode(dims, code);
            for to_move in [Color::Black, Color::White] {
                let st0 = State::start(&root, to_move);
                for first in all_moves(dims) {
                    if !permits(&st0, first, rep, SUICIDE) {
                        continue;
                    }
                    let st1 = step(&st0, first, SUICIDE);
                    let mut lines = vec![vec![first]];
                    if !ended(&st1) {
                        for second in all_moves(dims) {
                            if permits(&st1, second, rep, SUICIDE) {
                                lines.push(vec![first, second]);
                            }
                        }
                    }
                    for line in lines {
                        let st = walk(&root, to_move, rep, &line);
                        let expected = naive::value(&st, rep, SUICIDE);
                        let got = solver.solve_line(code, to_move, &line).value;
                        assert_eq!(got, Some(expected), "{rep:?} {code:?} {to_move:?} {line:?}");
                        compared += 1;
                    }
                }
            }
        }
    }
    assert_eq!(compared, 568);
}

/// The 1×6 witness: Black's return to the root board with White to move.
const ONE_BY_SIX: [Move; 13] = [
    play(0, 1),
    play(0, 3),
    play(0, 1),
    play(0, 5),
    play(0, 2),
    play(0, 0),
    play(0, 2),
    play(0, 1),
    play(0, 4),
    Move::Pass,
    play(0, 2),
    play(0, 1),
    play(0, 0),
];

#[test]
fn the_one_by_six_witness_returns_to_its_root() {
    let dims = Dims::new(1, 6);
    let root = parse(dims, "X.X.X.").expect("the witness");
    let table = RuleTable::build(dims, SUICIDE).expect("six points");
    let (before, last) = (&ONE_BY_SIX[..12], ONE_BY_SIX[12]);
    let st = walk(&root, Color::Black, Repetition::Ssk, before);
    assert_eq!(superko_rules::code::render(&st.now().board), ".OX.X.");
    assert!(ssk(&st, last, SUICIDE) && !psk(&st, last, SUICIDE));
    let after = walk(&root, Color::Black, Repetition::Ssk, &ONE_BY_SIX);
    assert_eq!(after.now().board, root, "the board comes back");
    assert_eq!(after.now().to_move, Color::White);
    for mv in all_moves(dims) {
        assert_eq!(
            permits(&after, mv, Repetition::Ssk, SUICIDE),
            mv == Move::Pass,
            "{mv}"
        );
    }

    let code = encode(&root);
    let mut s = Solver::new(&table, Repetition::Ssk);
    assert_eq!(s.solve_line(code, Color::Black, &[]).value, Some(6));
    assert_eq!(s.solve_line(code, Color::Black, before).value, Some(6));
    assert_eq!(s.solve_line(code, Color::Black, &ONE_BY_SIX).value, Some(6));
    assert_eq!(
        s.decide_line(code, Color::Black, &ONE_BY_SIX, 1, Color::Black)
            .wins,
        Some(true)
    );
    let mut p = Solver::new(&table, Repetition::Psk);
    assert_eq!(p.solve_line(code, Color::Black, &[]).value, Some(1));
    assert_eq!(p.solve_line(code, Color::Black, before).value, Some(1));
    assert_eq!(
        p.decide_line(code, Color::Black, before, 1, Color::White)
            .wins,
        Some(true)
    );
}

/// The 2×3 witness: White's return to the board after the second move, with
/// Black to move.
const TWO_BY_THREE: [Move; 11] = [
    Move::Pass,
    play(1, 0),
    play(1, 2),
    play(1, 1),
    Move::Pass,
    play(1, 0),
    play(0, 1),
    Move::Pass,
    play(0, 2),
    Move::Pass,
    play(0, 0),
];

#[test]
fn the_two_by_three_witness_returns_to_an_earlier_board() {
    let dims = Dims::new(2, 3);
    let root = parse(dims, "OOO/.X.").expect("the witness");
    let table = RuleTable::build(dims, SUICIDE).expect("six points");
    let (before, last) = (&TWO_BY_THREE[..10], TWO_BY_THREE[10]);
    let st = walk(&root, Color::White, Repetition::Ssk, before);
    assert_eq!(superko_rules::code::render(&st.now().board), ".OO/XX.");
    assert_eq!(
        st.passes(),
        1,
        "Black's pass stands: a White pass would end the game"
    );
    assert!(ssk(&st, last, SUICIDE) && !psk(&st, last, SUICIDE));
    let after = walk(&root, Color::White, Repetition::Ssk, &TWO_BY_THREE);
    let second = walk(&root, Color::White, Repetition::Ssk, &TWO_BY_THREE[..2]);
    assert_eq!(
        after.now().board,
        second.now().board,
        "the board after move 2 comes back"
    );
    assert_eq!(second.now().to_move, Color::White);
    assert_eq!(after.now().to_move, Color::Black);

    let code = encode(&root);
    let mut s = Solver::new(&table, Repetition::Ssk).with_budget(Some(10_000_000));
    assert_eq!(
        s.decide_line(code, Color::White, &TWO_BY_THREE, -1, Color::White)
            .wins,
        Some(true)
    );
    let mut p = Solver::new(&table, Repetition::Psk).with_budget(Some(10_000_000));
    assert_eq!(
        p.decide_line(code, Color::White, before, -1, Color::White)
            .wins,
        Some(false)
    );
    assert_eq!(
        p.decide_line(code, Color::White, before, -1, Color::Black)
            .wins,
        Some(true)
    );
}

#[test]
fn the_one_by_six_witness_in_a_game_follows_a_pass() {
    let dims = Dims::new(1, 6);
    let empty = Position::empty(dims);
    let target = parse(dims, "X.X.X.").expect("the witness");
    let table = RuleTable::build(dims, SUICIDE).expect("six points");
    let game = [
        play(0, 0),
        Move::Pass,
        play(0, 2),
        Move::Pass,
        play(0, 4),
        Move::Pass,
    ];
    for rep in RULES {
        let st = walk(&empty, Color::Black, rep, &game);
        assert_eq!(st.now().board, target);
        assert_eq!(st.now().to_move, Color::Black);
        assert_eq!(st.passes(), 1);
        let mut solver = Solver::new(&table, rep);
        let code = encode(&empty);
        assert_eq!(
            solver.solve_line(code, Color::Black, &game).value,
            Some(6),
            "{rep:?}"
        );
        assert_eq!(
            solver
                .decide_line(code, Color::Black, &game, 1, Color::Black)
                .wins,
            Some(true)
        );
        let over = walk(
            &empty,
            Color::Black,
            rep,
            &[&game[..], &[Move::Pass]].concat(),
        );
        assert!(ended(&over));
        assert_eq!(naive::score(&over.now().board), 6);
    }
}

/// `cargo test --release -p superko-solve --test lines -- --ignored the_two_by_three_witness_in_a_game_does_not_separate_the_rules`
#[test]
#[ignore = "release only: searches of tens of millions of nodes"]
fn the_two_by_three_witness_in_a_game_does_not_separate_the_rules() {
    let dims = Dims::new(2, 3);
    let empty = Position::empty(dims);
    let target = parse(dims, "OOO/.X.").expect("the witness");
    let table = RuleTable::build(dims, SUICIDE).expect("six points");
    let games: [&[Move]; 2] = [
        &[
            Move::Pass,
            play(0, 0),
            Move::Pass,
            play(0, 1),
            Move::Pass,
            play(0, 2),
            play(1, 1),
        ],
        &[
            Move::Pass,
            play(0, 0),
            Move::Pass,
            play(0, 1),
            play(1, 1),
            play(0, 2),
            Move::Pass,
        ],
    ];
    let expected = [1, -2];
    for (game, want) in games.iter().zip(expected) {
        for rep in RULES {
            let st = walk(&empty, Color::Black, rep, game);
            assert_eq!(st.now().board, target);
            assert_eq!(st.now().to_move, Color::White);
            let mut solver = Solver::new(&table, rep).with_budget(Some(100_000_000));
            let code = encode(&empty);
            assert_eq!(
                solver.solve_line(code, Color::Black, game).value,
                Some(want),
                "{rep:?}"
            );
            let black = solver
                .decide_line(code, Color::Black, game, -1, Color::Black)
                .wins;
            assert_eq!(black, Some(want > -1), "{rep:?}");
        }
    }
}
