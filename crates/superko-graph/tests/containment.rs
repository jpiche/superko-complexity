//! Positional superko is the stricter rule: every move it permits, situational
//! superko permits.
//!
//! The containment is a one-line consequence of the definitions —
//! `Superko.PSK` forbids a board that occurs with *either* color to move,
//! `Superko.SSK` only that board with *this* color to move — and it is not
//! proved anywhere in this project. It is checked here, at every move of every
//! state a walk reaches, so the ledger status of what this file establishes is
//! `computed` on those boards and nothing more.
//!
//! The check is worth making because it is the invariant the two-bit archive
//! rests on: one structure serves both rules, PSK reading the two bits of a
//! code together and SSK reading one. A difference between an SSK count and a
//! PSK count is attributable to the rule only if that structure is right.
//!
//! A containment that never bites asserts nothing, so the walk counts the moves
//! on which the two rules actually differ and the test fails if that count is
//! zero.

use superko_rules::code::render;
use superko_rules::config::{Dims, Point, Repetition, Suicide};
use superko_rules::reference::{
    Color, Move, Position, State, all_moves, ended, permits, psk, ssk, step,
};

#[derive(Default, Clone, Copy, Debug)]
struct Tally {
    /// Moves where SSK permits and PSK refuses: the rules differing.
    ssk_only: u64,
    /// Moves either rule permits, as the denominator.
    moves: u64,
}

/// Walk under situational superko — the weaker rule, so the tree contains
/// every state the stricter one reaches — and check the containment at every
/// move of every state.
fn walk(dims: Dims, suicide: Suicide, st: &State, depth: usize, tally: &mut Tally) {
    for mv in all_moves(dims) {
        let by_psk = psk(st, mv, suicide);
        let by_ssk = ssk(st, mv, suicide);
        assert!(
            !by_psk || by_ssk,
            "psk permits a move ssk refuses: {mv} at depth {depth}"
        );
        tally.moves += 1;
        if by_ssk && !by_psk {
            tally.ssk_only += 1;
        }
    }

    if ended(st) || depth == 0 {
        return;
    }

    for mv in all_moves(dims) {
        if !permits(st, mv, Repetition::Ssk, suicide) {
            continue;
        }
        let next = step(st, mv, suicide);
        walk(dims, suicide, &next, depth - 1, tally);
    }
}

/// A pass is exempt from both rules (OPEN-1, C-18, `cited`), so the containment
/// is only informative about plays — but the walk checks it on every move, pass
/// included, because a rule that started refusing passes would be a change this
/// project would want to see fail.
fn tally_of(rows: usize, cols: usize, suicide: Suicide, depth: usize) -> Tally {
    let dims = Dims::new(rows, cols);
    let start = State::start(&Position::empty(dims), Color::Black);
    let mut tally = Tally::default();
    walk(dims, suicide, &start, depth, &mut tally);
    assert!(tally.moves > 0);
    tally
}

#[test]
fn psk_legal_implies_ssk_legal() {
    let mut differing = 0u64;
    for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
        for cols in 1..=3 {
            differing += tally_of(1, cols, suicide, 12).ssk_only;
        }
        differing += tally_of(2, 2, suicide, 6).ssk_only;
    }
    assert!(
        differing > 0,
        "no move on any board tested separated the two rules, so the containment \
         was asserted about a case that never arises"
    );
}

/// Replay a sequence of moves from the empty board, checking each is legal
/// under situational superko as it goes.
fn play_out(dims: Dims, suicide: Suicide, moves: &[Move]) -> State {
    let mut st = State::start(&Position::empty(dims), Color::Black);
    for &mv in moves {
        assert!(
            permits(&st, mv, Repetition::Ssk, suicide),
            "{mv} is not legal under ssk in this line"
        );
        st = step(&st, mv, suicide);
    }
    st
}

const fn pt(row: usize, col: usize) -> Point {
    Point::new(row, col)
}

/// The shallowest line on which the two rules differ under `Defs.lean`'s own
/// suicide convention, written out, so that the counter above is not the only
/// evidence that they do.
///
/// On 1×3: Black passes, White takes the left point, Black the right, White
/// the middle — capturing Black's stone — and Black retakes at the right,
/// capturing both White stones. The board is now `..X` with White to move, and
/// White's play at the left point would recreate `O.X`, which stood earlier
/// with *White* to move. Positional superko forbids it; situational superko
/// does not, because the player to move differs. The parity was changed by the
/// opening pass, which is exactly the resource OPEN-1's pass exemption
/// supplies (C-18, `cited`).
#[test]
fn the_rules_differ_on_one_by_three() {
    let dims = Dims::new(1, 3);
    let suicide = Suicide::Forbid;
    let st = play_out(
        dims,
        suicide,
        &[
            Move::Pass,
            Move::Play(pt(0, 0)),
            Move::Play(pt(0, 2)),
            Move::Play(pt(0, 1)),
            Move::Play(pt(0, 2)),
        ],
    );
    assert_eq!(render(&st.now().board), "..X");
    assert_eq!(st.now().to_move, Color::White);

    let retake = Move::Play(pt(0, 0));
    assert!(ssk(&st, retake, suicide), "ssk refuses the retake");
    assert!(!psk(&st, retake, suicide), "psk permits the retake");
}

/// The same difference at its smallest, where the suicide convention supplies
/// the parity instead of a pass: on 1×1 with self-capture legal, Black's only
/// play returns the empty board. The root is archived, so positional superko
/// refuses it and the count stays at the published 1; situational superko
/// permits it, because the empty board has not stood with White to move.
#[test]
fn the_rules_differ_on_one_by_one_under_remove_own() {
    let dims = Dims::new(1, 1);
    let suicide = Suicide::RemoveOwn;
    let st = play_out(dims, suicide, &[]);
    let play = Move::Play(pt(0, 0));
    assert!(ssk(&st, play, suicide));
    assert!(!psk(&st, play, suicide));
}
