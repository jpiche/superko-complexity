//! Two runs produce the same report, and so does a parallel run.
//!
//! Determinism is a hard requirement rather than a nicety: `verify-results.sh`
//! diffs a results body, and a body that varied between runs would make every
//! witness in the repository unfalsifiable. The workspace forbids
//! `std::collections::HashMap` and `HashSet` for the same reason, and the
//! parallel search sums its parts in prefix order rather than in completion
//! order.

use superko_graph::enumerate::{self, Options};
use superko_graph::walk;
use superko_rules::config::{Dims, Repetition, Suicide};

fn run(dims: Dims, rep: Repetition, suicide: Suicide, opts: Options) -> Vec<String> {
    enumerate::count_games(dims, rep, suicide, opts)
        .expect("the board has a table")
        .lines()
}

#[test]
fn two_runs_are_byte_identical() {
    for (rows, cols) in [(1, 3), (2, 2)] {
        let dims = Dims::new(rows, cols);
        for rep in [Repetition::Ssk, Repetition::Psk] {
            let opts = Options {
                depth_cap: Some(7),
                threads: 1,
            };
            let first = run(dims, rep, Suicide::Forbid, opts);
            let second = run(dims, rep, Suicide::Forbid, opts);
            assert_eq!(first, second, "{rows}x{cols}/{rep} differed between runs");
            assert!(first.iter().any(|l| l.starts_with("games=")));
        }
    }
}

/// The naive enumerator is deterministic too, and its report is the same
/// shape, so the two can be diffed line by line.
#[test]
fn the_naive_report_has_the_same_shape() {
    let dims = Dims::new(1, 3);
    let fast = run(dims, Repetition::Psk, Suicide::Forbid, Options::default());
    let naive = walk::count_games(dims, Repetition::Psk, Suicide::Forbid, None).lines();
    assert_eq!(fast, naive);
}

/// A parallel count equals the serial one, field for field, at several thread
/// counts. The thread count is not in the body, and must not be observable in
/// it.
#[test]
fn threads_do_not_change_the_count() {
    for (rows, cols) in [(1, 3), (2, 2)] {
        let dims = Dims::new(rows, cols);
        for rep in [Repetition::Ssk, Repetition::Psk] {
            for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
                let depth_cap = if (rows, cols) == (2, 2) {
                    Some(8)
                } else {
                    None
                };
                let serial = enumerate::count_games(
                    dims,
                    rep,
                    suicide,
                    Options {
                        depth_cap,
                        threads: 1,
                    },
                )
                .expect("the board has a table");
                for threads in [2usize, 3, 8] {
                    let parallel =
                        enumerate::count_games(dims, rep, suicide, Options { depth_cap, threads })
                            .expect("the board has a table");
                    assert_eq!(
                        serial, parallel,
                        "{rows}x{cols}/{rep}/{suicide} differed at {threads} threads"
                    );
                }
            }
        }
    }
}

/// A board whose whole tree lies above the split depth has no prefixes to hand
/// out, and the parallel path falls back to the stem it already counted.
#[test]
fn a_tiny_board_survives_being_parallelized() {
    let dims = Dims::new(1, 1);
    let serial = enumerate::count_games(dims, Repetition::Psk, Suicide::Forbid, Options::default())
        .expect("the board has a table");
    let parallel = enumerate::count_games(
        dims,
        Repetition::Psk,
        Suicide::Forbid,
        Options {
            depth_cap: None,
            threads: 4,
        },
    )
    .expect("the board has a table");
    assert_eq!(serial, parallel);
    assert_eq!(serial.games, 1);
}
