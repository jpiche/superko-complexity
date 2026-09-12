//! The arbiter: exact minimax over `superko_rules::reference::State`, with
//! nothing done for speed.
//!
//! Every node clones a state and its `BTreeSet` archive, every legality
//! question calls the rule predicate itself, and nothing is pruned. That is
//! the point: this module shares no control flow with [`crate::search`], so
//! the two agreeing on a value is evidence about both, and it is the only
//! evidence this crate offers that the fast path's make-and-unmake and its
//! alpha-beta cutoffs preserve the answer.
//!
//! Unusable past about four points. `tests/agreement.rs` runs it over every
//! root position of every board with `m · n <= 3`.
//!
//! # Two recursions, deliberately separate
//!
//! [`wins_for`] mirrors `Superko.decideWins`: a two-valued game at a fixed
//! komi floor, which is the question `Superko.WinsFor` asks. [`value`] returns
//! the minimax area difference, which `Defs.lean` does not define at all.
//! Neither is computed from the other here, so that `tests/agreement.rs` can
//! check that they agree.

use superko_rules::config::{Repetition, Suicide};
use superko_rules::reference::{
    Color, Position, State, all_moves, area, ended, permits, step, winner_z,
};

/// Black's area less White's — the quantity [`value`] optimizes.
///
/// Area scoring counts every point for at most one color, so this lies in
/// `-(m · n) ..= m · n`.
///
/// # Panics
///
/// Panics when an area score does not fit an `i32`, which no board this
/// workspace admits can produce.
#[must_use]
pub fn score(b: &Position) -> i32 {
    let black = i32::try_from(area(b, Color::Black)).expect("area overflowed");
    let white = i32::try_from(area(b, Color::White)).expect("area overflowed");
    black - white
}

/// The minimax area difference from this state: Black maximizes, White
/// minimizes, and a finished game is worth [`score`] of its position.
///
/// A pass is always legal, so a state that has not ended always has a move and
/// the recursion never takes a maximum over nothing. Termination is C-13,
/// `proved`: every move either archives a situation not yet archived or raises
/// the pass count.
#[must_use]
pub fn value(st: &State, rep: Repetition, suicide: Suicide) -> i32 {
    if ended(st) {
        return score(&st.now().board);
    }
    let maximizing = st.now().to_move == Color::Black;
    let mut best: Option<i32> = None;
    for mv in all_moves(st.now().board.dims()) {
        if !permits(st, mv, rep, suicide) {
            continue;
        }
        let v = value(&step(st, mv, suicide), rep, suicide);
        best = Some(match best {
            None => v,
            Some(b) if maximizing => b.max(v),
            Some(b) => b.min(v),
        });
    }
    best.expect("a state that has not ended permits a pass")
}

/// Whether `c` has a winning strategy from this state at this komi floor —
/// the question `Superko.WinsFor` asks, by the recursion
/// `Superko.decideWins` uses.
///
/// The leaf test is `superko_rules::reference::winner_z`, the integer form
/// `Superko.winnerZ_eq_winner` licenses.
#[must_use]
pub fn wins_for(st: &State, rep: Repetition, suicide: Suicide, komi_floor: i64, c: Color) -> bool {
    if ended(st) {
        return winner_z(&st.now().board, komi_floor) == c;
    }
    let mover = st.now().to_move == c;
    for mv in all_moves(st.now().board.dims()) {
        if !permits(st, mv, rep, suicide) {
            continue;
        }
        let won = wins_for(&step(st, mv, suicide), rep, suicide, komi_floor, c);
        if mover {
            if won {
                return true;
            }
        } else if !won {
            return false;
        }
    }
    // The mover found no winning move; the waiter found no losing one. A pass
    // is always legal, so the waiter's case is never vacuous.
    !mover
}

/// The minimax area difference from a position taken as the root of play —
/// `Superko.start`, encoding (C).
#[must_use]
pub fn value_from(b: &Position, to_move: Color, rep: Repetition, suicide: Suicide) -> i32 {
    value(&State::start(b, to_move), rep, suicide)
}

/// Whether `c` wins from a position taken as the root of play.
#[must_use]
pub fn wins_from(
    b: &Position,
    to_move: Color,
    rep: Repetition,
    suicide: Suicide,
    komi_floor: i64,
    c: Color,
) -> bool {
    wins_for(&State::start(b, to_move), rep, suicide, komi_floor, c)
}
