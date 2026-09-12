//! The invariants every state of a search carries, checked at every state a
//! small enumeration visits.
//!
//! Four of them, all of them facts about `Defs.lean` that the fast enumerator
//! quietly relies on:
//!
//! * `now ∈ seen` — `Superko.start` seeds the root and `Superko.step` inserts
//!   on every move, so the situation standing is always archived. This is why
//!   the make-and-unmake archive must remember whether its insert was new: a
//!   pass archives a key that may already be set, and clearing it
//!   unconditionally would forget a genuine earlier occurrence.
//! * A legal play grows the archive by exactly one, and a pass by zero or one.
//! * Every leaf is `Ended`. The walk checks the equivalent and stronger claim
//!   that a state which has not ended has at least one legal move — the pass,
//!   which OPEN-1 exempts from the repetition rule (C-18, `cited`).
//! * The two area scores sum to at most `m · n`, since a point counts for a
//!   color only when it does not reach the other.
//!
//! The walk is exhaustive to its depth cap and no further. What it establishes
//! is `computed` about 1×3 at full depth and 2×2 to eight moves, and nothing
//! about a full 2×2 enumeration, which no test in this workspace runs.

use std::collections::BTreeSet;

use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::{
    Color, Move, Position, State, all_moves, area, ended, permits, step,
};

/// The archive the rule actually reads: situations under SSK, positions under
/// PSK, which is `Superko.PSK` consulting `seen` only through `.board`.
fn archive(st: &State, rep: Repetition) -> usize {
    match rep {
        Repetition::Ssk => st.seen().len(),
        Repetition::Psk => st
            .seen()
            .iter()
            .map(|s| &s.board)
            .collect::<BTreeSet<_>>()
            .len(),
    }
}

struct Invariants {
    dims: Dims,
    rep: Repetition,
    suicide: Suicide,
    nodes: u64,
    leaves: u64,
}

impl Invariants {
    fn visit(&mut self, st: &State, depth: usize) {
        self.nodes += 1;

        assert!(st.seen().contains(st.now()), "now is not archived");

        let black = area(&st.now().board, Color::Black);
        let white = area(&st.now().board, Color::White);
        assert!(
            black + white <= self.dims.point_count(),
            "the two areas overlap"
        );

        let legal: Vec<Move> = all_moves(self.dims)
            .into_iter()
            .filter(|&mv| permits(st, mv, self.rep, self.suicide))
            .collect();

        if ended(st) {
            self.leaves += 1;
            return;
        }
        assert!(
            legal.contains(&Move::Pass),
            "a state that has not ended refuses the pass"
        );

        if depth == 0 {
            return;
        }

        for mv in legal {
            let next = step(st, mv, self.suicide);
            let grown = archive(&next, self.rep) - archive(st, self.rep);
            match mv {
                Move::Play(_) => assert_eq!(
                    grown, 1,
                    "a legal play grew the archive by {grown}, not by one"
                ),
                Move::Pass => assert!(
                    grown <= 1,
                    "a pass grew the archive by {grown}, not by zero or one"
                ),
            }
            self.visit(&next, depth - 1);
        }
    }
}

fn check(rows: usize, cols: usize, depth: usize) {
    let dims = Dims::new(rows, cols);
    for rep in [Repetition::Ssk, Repetition::Psk] {
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let mut walk = Invariants {
                dims,
                rep,
                suicide,
                nodes: 0,
                leaves: 0,
            };
            walk.visit(&State::start(&Position::empty(dims), Color::Black), depth);
            assert!(
                walk.nodes > 1,
                "{rows}x{cols}/{rep}/{suicide} visited nothing"
            );
            assert!(
                walk.leaves > 0,
                "{rows}x{cols}/{rep}/{suicide} reached no finished game, so \
                 'every leaf is ended' was asserted about nothing"
            );
        }
    }
}

/// 1×3 at full depth: 21 moves is the longest game the enumerator reports
/// there, so this cap truncates nothing.
#[test]
fn invariants_hold_on_one_by_three() {
    check(1, 3, 30);
}

/// 2×2 to eight moves, deep enough that the archive holds several positions
/// and refuses plays.
#[test]
fn invariants_hold_on_two_by_two() {
    check(2, 2, 8);
}
