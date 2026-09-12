//! 1×4 under positional superko: the published count, and the count
//! `Defs.lean`'s own rule gives.
//!
//! The heavy checks are `#[ignore]`, and take a minute or two each in release
//! and much longer in a debug build; run them with
//! `cargo test --release -p superko-graph -- --ignored`.
//!
//! # What the runs of 2026-09-11 found
//!
//! Tromp–Farnebäck's Table 7 gives 2 098 407 841 games on 1×4 under positional
//! superko (C-9, `cited`). This workspace reproduces that figure **exactly**
//! under `Suicide::RemoveOwn`, the paper's own suicide-permitting convention,
//! and gives 719 178 893 under `Suicide::Forbid`, which is `Defs.lean`'s rule
//! (both `computed`).
//!
//! So the suicide convention is observable in the game count at 1×4
//! (`computed`), and the hand argument recorded in experiment 004's hypothesis
//! — that a position-changing self-capture on a line needs six points — is
//! `refuted`. The witness is small and independent of the counts: on 1×4 the
//! position `.OX.` admits Black at the right point, whose two-stone chain then
//! has no liberty, and the resulting `.O..` is a position the game need not
//! have visited. Under `Forbid` that play is refused as suicide; under
//! `RemoveOwn` it is a legal move to a new position, and the games below it
//! are counted.
//!
//! Why 1, 9 and 907 nonetheless reproduce under both conventions —
//! that on a board this small every such self-capture returns the line to
//! empty, which is archived from the root — is `conjecture`. It accounts for
//! the equality the run found; nothing here proves it.
//!
//! What this does **not** establish: that either figure is right. It
//! establishes that this workspace agrees with a published enumeration under
//! the published convention, on one board, and that `Defs.lean`'s convention
//! gives a different number there. `results/` is where a promoted witness would
//! record which is which.

use superko_graph::enumerate::{self, Options};
use superko_graph::walk;
use superko_rules::config::{Dims, Repetition, Suicide};

/// The published 1×4 count, under the paper's suicide convention.
#[test]
#[ignore = "minutes in release, far longer in debug"]
fn one_by_four_reproduces_table_seven_under_remove_own() {
    let report = enumerate::count_games(
        Dims::new(1, 4),
        Repetition::Psk,
        Suicide::RemoveOwn,
        Options::default(),
    )
    .expect("the board has a table");
    assert_eq!(report.games, 2_098_407_841);
}

/// The same board under `Defs.lean`'s rule, where suicide is illegal. This
/// number is not in any publication; it is this workspace's, and it is
/// `computed`.
#[test]
#[ignore = "minutes in release, far longer in debug"]
fn one_by_four_under_defs_lean_is_not_the_published_count() {
    let report = enumerate::count_games(
        Dims::new(1, 4),
        Repetition::Psk,
        Suicide::Forbid,
        Options::default(),
    )
    .expect("the board has a table");
    assert_eq!(report.games, 719_178_893);
    assert_ne!(report.games, 2_098_407_841);
}

/// The default tier's check at 1×4: the two enumerators agree to a depth cap,
/// under both rules and both conventions, on the board where the conventions
/// first part company.
#[test]
fn one_by_four_agrees_with_the_arbiter_to_a_depth_cap() {
    let dims = Dims::new(1, 4);
    for rep in [Repetition::Ssk, Repetition::Psk] {
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let depth_cap = Some(12);
            let fast = enumerate::count_games(
                dims,
                rep,
                suicide,
                Options {
                    depth_cap,
                    threads: 1,
                },
            )
            .expect("the board has a table");
            let naive = walk::count_games(dims, rep, suicide, depth_cap);
            assert_eq!(fast, naive, "1x4/{rep}/{suicide} to twelve moves");
            assert!(fast.truncated > 0);
            assert!(fast.refused_repetition > 0);
        }
    }
}

/// The conventions differ on 1×4 within twelve moves, and agree on 1×3 at full
/// depth. Cheap, and it pins the finding the `#[ignore]` tests above spell out
/// without running either of them.
#[test]
fn the_suicide_convention_is_observable_at_one_by_four() {
    let capped = |rows, cols, suicide, cap| {
        enumerate::count_games(
            Dims::new(rows, cols),
            Repetition::Psk,
            suicide,
            Options {
                depth_cap: cap,
                threads: 1,
            },
        )
        .expect("the board has a table")
        .games
    };

    assert_eq!(
        capped(1, 3, Suicide::Forbid, None),
        capped(1, 3, Suicide::RemoveOwn, None),
        "the conventions part company on 1x3, where the argument says they do not"
    );
    assert_ne!(
        capped(1, 4, Suicide::Forbid, Some(12)),
        capped(1, 4, Suicide::RemoveOwn, Some(12)),
        "the conventions agree on 1x4 to twelve moves"
    );
}
