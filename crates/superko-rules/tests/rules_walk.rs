//! Move order, the decomposed legality verdict, and the invariants a state
//! carries, over every state a short walk from the empty board reaches.
//!
//! The walk is exhaustive to its depth cap and no further: what it establishes
//! is `computed` about 1x3 to seven moves and 2x2 to six, and nothing about a
//! full enumeration, which this crate does not have.

use superko_rules::config::{Dims, Point, Repetition, Suicide};
use superko_rules::reference::{
    Color, Move, Position, Refusal, State, all_moves, area, ended, has_liberty, legality, permits,
    resolve, situation_after, step,
};

const fn pt(row: usize, col: usize) -> Point {
    Point::new(row, col)
}

/// Every count this workspace produces depends on this order, so it is pinned
/// against a literal list rather than against a second computation of it.
/// `Superko.allMoves` is `Move.pass :: (finRange m ×ˢ finRange n).map Move.play`,
/// and `List.product` runs the left factor slowest, which is row-major.
#[test]
fn all_moves_order_is_pinned() {
    assert_eq!(
        all_moves(Dims::new(1, 1)),
        vec![Move::Pass, Move::Play(pt(0, 0))]
    );

    assert_eq!(
        all_moves(Dims::new(1, 3)),
        vec![
            Move::Pass,
            Move::Play(pt(0, 0)),
            Move::Play(pt(0, 1)),
            Move::Play(pt(0, 2)),
        ]
    );

    assert_eq!(
        all_moves(Dims::new(2, 2)),
        vec![
            Move::Pass,
            Move::Play(pt(0, 0)),
            Move::Play(pt(0, 1)),
            Move::Play(pt(1, 0)),
            Move::Play(pt(1, 1)),
        ]
    );

    assert_eq!(
        all_moves(Dims::new(3, 2)),
        vec![
            Move::Pass,
            Move::Play(pt(0, 0)),
            Move::Play(pt(0, 1)),
            Move::Play(pt(1, 0)),
            Move::Play(pt(1, 1)),
            Move::Play(pt(2, 0)),
            Move::Play(pt(2, 1)),
        ]
    );
}

/// The refusal the diagnostic reports is the first conjunct to fail, in the
/// order `Occupied`, `Suicide`, `Repetition`, and `legality(..).is_ok()` is the
/// rule predicate itself.
fn expected_refusal(
    st: &State,
    mv: Move,
    rep: Repetition,
    suicide: Suicide,
) -> Result<(), Refusal> {
    let Move::Play(p) = mv else { return Ok(()) };
    let b = &st.now().board;
    let c = st.now().to_move;
    if b.get(p).is_some() {
        return Err(Refusal::Occupied);
    }
    if matches!(suicide, Suicide::Forbid) && !has_liberty(&resolve(b, c, p), p) {
        return Err(Refusal::Suicide);
    }
    let after = situation_after(st.now(), mv, suicide);
    let repeats = match rep {
        Repetition::Ssk => st.seen().contains(&after),
        Repetition::Psk => st.seen().iter().any(|s| s.board == after.board),
    };
    if repeats {
        return Err(Refusal::Repetition);
    }
    Ok(())
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
struct Census {
    occupied: usize,
    suicide: usize,
    repetition: usize,
}

struct Walk {
    dims: Dims,
    rep: Repetition,
    suicide: Suicide,
    nodes: usize,
    census: Census,
}

impl Walk {
    fn visit(&mut self, st: &State, depth: usize) {
        self.nodes += 1;

        // `now` is archived: the invariant `start` establishes and `step`
        // preserves, and the reason a make-and-unmake archive needs the
        // was-new answer.
        assert!(st.seen().contains(st.now()), "now is not archived");

        // The areas partition no more than the board.
        let black = area(&st.now().board, Color::Black);
        let white = area(&st.now().board, Color::White);
        assert!(black + white <= self.dims.point_count());

        let mut legal = 0usize;
        for mv in all_moves(self.dims) {
            let verdict = legality(st, mv, self.rep, self.suicide);
            assert_eq!(
                verdict.is_ok(),
                permits(st, mv, self.rep, self.suicide),
                "legality disagrees with the rule on {mv}"
            );
            assert_eq!(
                verdict,
                expected_refusal(st, mv, self.rep, self.suicide),
                "refusal priority differs on {mv}"
            );
            match verdict {
                Ok(()) => legal += 1,
                Err(Refusal::Occupied) => self.census.occupied += 1,
                Err(Refusal::Suicide) => self.census.suicide += 1,
                Err(Refusal::Repetition) => self.census.repetition += 1,
            }
        }

        if ended(st) {
            return;
        }
        assert!(
            legal >= 1,
            "a state that has not ended has at least the pass"
        );

        if depth == 0 {
            return;
        }

        for mv in all_moves(self.dims) {
            if !permits(st, mv, self.rep, self.suicide) {
                continue;
            }
            let next = step(st, mv, self.suicide);
            match mv {
                Move::Play(_) => assert_eq!(
                    next.seen().len(),
                    st.seen().len() + 1,
                    "a legal play grows the archive by one"
                ),
                Move::Pass => {
                    let grown = next.seen().len() - st.seen().len();
                    assert!(grown <= 1, "a pass grows the archive by zero or one");
                }
            }
            self.visit(&next, depth - 1);
        }
    }
}

/// Walk every state the board reaches within `depth` moves, under all four
/// combinations of the two axes, and return the refusal census of each. A
/// combination whose census is all zeros has tested the agreement on nothing.
fn walk_board(rows: usize, cols: usize, depth: usize) -> Vec<(Repetition, Suicide, Census)> {
    let dims = Dims::new(rows, cols);
    let mut out = Vec::new();
    for rep in [Repetition::Ssk, Repetition::Psk] {
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let mut walk = Walk {
                dims,
                rep,
                suicide,
                nodes: 0,
                census: Census::default(),
            };
            let start = State::start(&Position::empty(dims), Color::Black);
            walk.visit(&start, depth);
            assert!(walk.nodes > 1);
            out.push((rep, suicide, walk.census));
        }
    }
    out
}

/// Every refusal kind the configuration admits is actually reached, so the
/// agreement above is not asserted about an empty set of moves. Suicide is a
/// refusal only where the convention forbids it.
fn assert_every_branch_was_taken(census: &[(Repetition, Suicide, Census)]) {
    for (rep, suicide, c) in census {
        assert!(c.occupied > 0, "no occupied refusal under {rep}/{suicide}");
        assert!(
            c.repetition > 0,
            "no repetition refusal under {rep}/{suicide}"
        );
        match suicide {
            Suicide::Forbid => assert!(c.suicide > 0, "no suicide refusal under {rep}/{suicide}"),
            Suicide::RemoveOwn => {
                assert_eq!(c.suicide, 0, "RemoveOwn refuses no move as suicide");
            }
        }
    }
}

#[test]
fn legality_agrees_on_one_by_three() {
    assert_every_branch_was_taken(&walk_board(1, 3, 7));
}

#[test]
fn legality_agrees_on_two_by_two() {
    assert_every_branch_was_taken(&walk_board(2, 2, 6));
}

/// Priority is observable: an occupied point that would also be suicide is
/// refused as occupied.
#[test]
fn occupied_outranks_suicide() {
    let dims = Dims::new(1, 1);
    let mut b = Position::empty(dims);
    b.set(pt(0, 0), Some(Color::Black));
    let st = State::start(&b, Color::Black);
    let mv = Move::Play(pt(0, 0));
    assert!(!has_liberty(&resolve(&b, Color::Black, pt(0, 0)), pt(0, 0)));
    assert_eq!(
        legality(&st, mv, Repetition::Ssk, Suicide::Forbid),
        Err(Refusal::Occupied)
    );
}
