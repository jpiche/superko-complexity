//! The published game counts, and the two enumerators held against each other.
//!
//! The counts 1, 9 and 907 are Tromp–Farnebäck's Table 7 (C-9, `cited`);
//! reproducing them is what experiment 004 is for. Everything asserted here is
//! `computed`: an agreement of this workspace with a published table on three
//! boards, and an agreement of two enumerators with each other on those boards
//! and on 2×2 to a depth cap.
//!
//! The full [`Report`] is compared, not the game count alone. A count
//! reproduced by accident — two conventions swapped and cancelling — looks
//! exactly like a count reproduced correctly, and the refusal census is what
//! distinguishes them.

use superko_graph::enumerate::{self, Options, Report};
use superko_graph::walk;
use superko_rules::config::{Dims, Repetition, Suicide};

/// The fast report and the naive report of one configuration.
fn both(
    dims: Dims,
    rep: Repetition,
    suicide: Suicide,
    depth_cap: Option<usize>,
) -> (Report, Report) {
    let opts = Options {
        depth_cap,
        threads: 1,
    };
    let fast = enumerate::count_games(dims, rep, suicide, opts).expect("the board has a table");
    let naive = walk::count_games(dims, rep, suicide, depth_cap);
    (fast, naive)
}

/// 1×1, 1×2 and 1×3 under positional superko, under both suicide conventions.
///
/// The suicide convention is invisible in the count on these three boards
/// (`computed`, the assertions below), and it is observable on 1×4
/// (`computed`; see `tests/published.rs`).
///
/// The reason offered for the first is narrower than the hand argument of
/// `notebook/2026-09-11-c9-primary-sources.md` gives: a position-changing
/// self-capture exists already on 1×3 — `XX.`, Black at the right point — but
/// every one of them on a board this small empties the line, and the empty
/// position is archived from the root, so positional superko refuses the play
/// either way. That explanation is `conjecture`: it accounts for the equality
/// the run found and is proved nowhere, and the equality itself is what the
/// test checks.
///
/// The convention is *not* invisible in the refusal census even here, which is
/// why only the game count is compared across conventions and the full report
/// is compared between enumerators.
#[test]
fn published_psk_counts() {
    for (cols, expected) in [(1u64, 1u64), (2, 9), (3, 907)] {
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let dims = Dims::new(1, usize::try_from(cols).expect("a small board"));
            let (fast, naive) = both(dims, Repetition::Psk, suicide, None);
            assert_eq!(
                fast.games, expected,
                "1x{cols} under psk/{suicide} is not the published count"
            );
            assert_eq!(
                fast, naive,
                "the enumerators disagree on 1x{cols}/{suicide}"
            );
        }
    }
}

/// Under situational superko there is no published count to check against, so
/// what is asserted is the agreement of the two enumerators and the direction
/// of the difference: SSK permits everything PSK permits, so it cannot have
/// fewer games.
#[test]
fn ssk_agrees_with_itself_and_dominates_psk() {
    let mut strictly_more = 0;
    for cols in 1..=3 {
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let dims = Dims::new(1, cols);
            let (fast, naive) = both(dims, Repetition::Ssk, suicide, None);
            assert_eq!(
                fast, naive,
                "the enumerators disagree on 1x{cols}/{suicide}"
            );
            let psk = both(dims, Repetition::Psk, suicide, None).0;
            assert!(
                fast.games >= psk.games,
                "ssk has fewer games than psk on 1x{cols}/{suicide}"
            );
            if fast.games > psk.games {
                strictly_more += 1;
            }
        }
    }
    assert!(
        strictly_more > 0,
        "no board tested distinguished the two rules, so the comparison asserted nothing"
    );
}

/// 2×2 is out of the naive enumerator's reach at full depth, so the two are
/// compared to a depth cap deep enough that the archive is doing work. Ten
/// moves is the first cap at which every one of the four combinations refuses
/// a play as a repetition; at six, `Defs.lean`'s convention has not yet
/// recreated a position under either rule, and the comparison would be one
/// between two searches with no superko in them.
#[test]
fn two_by_two_agrees_to_a_depth_cap() {
    for rep in [Repetition::Ssk, Repetition::Psk] {
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let (fast, naive) = both(Dims::new(2, 2), rep, suicide, Some(10));
            assert_eq!(
                fast, naive,
                "the enumerators disagree on 2x2/{rep}/{suicide}"
            );
            assert!(fast.truncated > 0, "a depth cap of ten truncated nothing");
            assert!(
                fast.refused_repetition > 0,
                "no repetition refusal, so the archive was not doing work"
            );
        }
    }
}

/// A depth cap counts truncated leaves apart from games, and never as games.
#[test]
fn a_cap_does_not_invent_games() {
    let full = both(Dims::new(1, 3), Repetition::Psk, Suicide::Forbid, None).0;
    let capped = both(Dims::new(1, 3), Repetition::Psk, Suicide::Forbid, Some(6)).0;
    assert_eq!(full.games, 907);
    assert_eq!(full.truncated, 0);
    assert!(capped.games < full.games);
    assert!(capped.truncated > 0);
    assert!(capped.nodes < full.nodes);
    assert_eq!(capped.max_depth, 6);
}
