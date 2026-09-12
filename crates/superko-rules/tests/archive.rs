//! The two-bits-per-position archive, and the make-and-unmake contract that
//! makes it safe to reuse one archive down a search.

use superko_rules::archive::{Archive, ArchiveKey};
use superko_rules::code::{PosCode, encode};
use superko_rules::config::{Dims, Point, Repetition, Suicide};
use superko_rules::reference::{
    Color, Move, Position, State, all_moves, ended, permits, playable_at, situation_after, step,
};

#[test]
fn insert_reports_novelty_and_undo_clears_only_what_it_set() {
    let dims = Dims::new(2, 2);
    let mut archive = Archive::new(dims);
    let key = ArchiveKey::new(PosCode(0), Color::Black);

    assert!(archive.is_empty());
    assert!(archive.insert(key), "the first insert is new");
    assert_eq!(archive.len(), 1);
    assert!(archive.contains_ssk(key));

    let again = archive.insert(key);
    assert!(!again, "the second insert is not new");
    assert_eq!(archive.len(), 1);

    archive.undo(key, again);
    assert!(archive.contains_ssk(key), "undoing a repeat keeps the bit");
    assert_eq!(archive.len(), 1);

    archive.undo(key, true);
    assert!(!archive.contains_ssk(key));
    assert!(archive.is_empty());
}

#[test]
fn the_two_bits_of_a_code_are_separate() {
    let dims = Dims::new(2, 2);
    let mut archive = Archive::new(dims);
    let black = ArchiveKey::new(PosCode(5), Color::Black);
    let white = ArchiveKey::new(PosCode(5), Color::White);

    assert!(archive.insert(black));
    assert!(archive.contains_ssk(black));
    assert!(
        !archive.contains_ssk(white),
        "situational superko reads one bit"
    );
    assert!(
        archive.contains_psk(PosCode(5)),
        "positional superko reads both"
    );
    assert!(archive.contains(black, Repetition::Ssk));
    assert!(archive.contains(white, Repetition::Psk));
    assert!(!archive.contains(white, Repetition::Ssk));

    assert!(archive.insert(white));
    assert_eq!(archive.len(), 2);
}

/// The case the was-new answer exists for: a pass archives the same position
/// under the other color, so the insert is new and the undo must clear that
/// bit and no other. A make-and-unmake that cleared the position outright
/// would forget the earlier occurrence.
#[test]
fn a_pass_archives_a_position_that_is_already_there() {
    let dims = Dims::new(2, 2);
    let empty = Position::empty(dims);
    let st = State::start(&empty, Color::Black);
    let mut archive = Archive::new(dims);

    let root = ArchiveKey::new(encode(&st.now().board), st.now().to_move);
    assert!(archive.insert(root));

    let after = situation_after(st.now(), Move::Pass, Suicide::Forbid);
    let passed = ArchiveKey::new(encode(&after.board), after.to_move);
    assert_eq!(passed.code, root.code, "a pass changes no stone");
    assert!(
        archive.contains_psk(passed.code),
        "the position is already archived"
    );

    let was_new = archive.insert(passed);
    assert!(was_new, "the situation is not");
    assert_eq!(archive.len(), 2);

    archive.undo(passed, was_new);
    assert_eq!(archive.len(), 1);
    assert!(
        archive.contains_ssk(root),
        "the earlier occurrence survives"
    );
}

/// The archive and `reference::State`'s `BTreeSet` agree over a short walk,
/// which is what lets the fast path carry one and the naive path the other.
#[test]
fn archive_agrees_with_the_reference_state() {
    let dims = Dims::new(2, 2);
    let mut st = State::start(&Position::empty(dims), Color::Black);
    let mut archive = Archive::new(dims);
    let mut trail = Vec::new();

    let root = ArchiveKey::new(encode(&st.now().board), st.now().to_move);
    let was_new = archive.insert(root);
    assert!(was_new);
    trail.push((root, was_new));

    for mv in [
        Move::Play(Point::new(0, 0)),
        Move::Play(Point::new(1, 1)),
        Move::Pass,
        Move::Play(Point::new(0, 1)),
    ] {
        st = step(&st, mv, Suicide::Forbid);
        let key = ArchiveKey::new(encode(&st.now().board), st.now().to_move);
        let was_new = archive.insert(key);
        trail.push((key, was_new));
        assert_eq!(archive.len(), st.seen().len(), "after {mv}");
        for s in st.seen() {
            let k = ArchiveKey::new(encode(&s.board), s.to_move);
            assert!(archive.contains_ssk(k));
        }
    }

    while let Some((key, was_new)) = trail.pop() {
        archive.undo(key, was_new);
    }
    assert!(archive.is_empty(), "unmake restores the empty archive");
}

/// `undo` refuses a key of another board whether or not it would clear a bit.
/// The early return on a repeat insert is the case a drifting make-and-unmake
/// pair would slip through, so the key is checked first.
#[test]
#[should_panic(expected = "code is not a position of this board")]
fn undo_refuses_a_foreign_key_even_when_it_clears_nothing() {
    let mut archive = Archive::new(Dims::new(1, 2));
    archive.undo(ArchiveKey::new(PosCode(9), Color::Black), false);
}

/// The archive reads are the rules, on every state a short walk reaches.
///
/// This is the one check behind `psk-archive-projection`, the divergence whose
/// Lean license is not written: `Superko.PSK` quantifies over archived
/// *situations* and compares their boards, and the archive instead reads the
/// two bits of a position code. That the two agree is `computed` for the boards
/// and depths below and for nothing else. `Superko.SSK`'s membership test is
/// checked alongside it, with the same archive, which is the point of one
/// structure serving both rules.
struct ArchiveWalk {
    dims: Dims,
    rep: Repetition,
    suicide: Suicide,
    archive: Archive,
    nodes: usize,
    repetitions: usize,
}

impl ArchiveWalk {
    fn visit(&mut self, st: &State, depth: usize) {
        self.nodes += 1;
        assert_eq!(
            self.archive.len(),
            st.seen().len(),
            "the archive and the reference set hold the same number of situations"
        );
        for s in st.seen() {
            let key = ArchiveKey::new(encode(&s.board), s.to_move);
            assert!(
                self.archive.contains_ssk(key),
                "an archived situation is missing"
            );
            assert!(self.archive.contains_psk(key.code));
        }

        for mv in all_moves(self.dims) {
            let want = permits(st, mv, self.rep, self.suicide);
            let got = match mv {
                Move::Pass => true,
                Move::Play(p) => {
                    let after = situation_after(st.now(), mv, self.suicide);
                    let key = ArchiveKey::new(encode(&after.board), after.to_move);
                    playable_at(&st.now().board, st.now().to_move, p, self.suicide)
                        && !self.archive.contains(key, self.rep)
                }
            };
            assert_eq!(want, got, "the archive disagrees with {} on {mv}", self.rep);

            let Move::Play(p) = mv else { continue };

            // The archive read was the deciding conjunct here, which is what
            // makes the agreement above worth asserting.
            if playable_at(&st.now().board, st.now().to_move, p, self.suicide) && !want {
                self.repetitions += 1;
            }

            // PSK-legal implies SSK-legal: the positional read is the disjunction
            // of the two bits the situational read consults singly.
            if permits(st, mv, Repetition::Psk, self.suicide) {
                assert!(
                    permits(st, mv, Repetition::Ssk, self.suicide),
                    "a PSK-legal play is refused by SSK on {mv}"
                );
            }
        }

        if ended(st) || depth == 0 {
            return;
        }
        for mv in all_moves(self.dims) {
            if !permits(st, mv, self.rep, self.suicide) {
                continue;
            }
            let next = step(st, mv, self.suicide);
            let key = ArchiveKey::new(encode(&next.now().board), next.now().to_move);
            let was_new = self.archive.insert(key);
            self.visit(&next, depth - 1);
            self.archive.undo(key, was_new);
        }
    }
}

fn archive_walk(rows: usize, cols: usize, depth: usize) {
    let dims = Dims::new(rows, cols);
    for rep in [Repetition::Ssk, Repetition::Psk] {
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let st = State::start(&Position::empty(dims), Color::Black);
            let mut walk = ArchiveWalk {
                dims,
                rep,
                suicide,
                archive: Archive::new(dims),
                nodes: 0,
                repetitions: 0,
            };
            let root = ArchiveKey::new(encode(&st.now().board), st.now().to_move);
            assert!(walk.archive.insert(root), "the root seed is new");
            walk.visit(&st, depth);
            walk.archive.undo(root, true);
            assert!(walk.archive.is_empty(), "unmake restores the empty archive");
            assert!(
                walk.nodes > 1,
                "the walk visited nothing under {rep}/{suicide}"
            );
            assert!(
                walk.repetitions > 0,
                "no play was refused as a repetition under {rep}/{suicide}, so the \
                 archive read was never the deciding conjunct"
            );
        }
    }
}

#[test]
fn archive_reads_are_the_rules_on_one_by_three() {
    archive_walk(1, 3, 7);
}

#[test]
fn archive_reads_are_the_rules_on_two_by_two() {
    archive_walk(2, 2, 6);
}
