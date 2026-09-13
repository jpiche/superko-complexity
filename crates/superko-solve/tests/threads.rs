//! A sweep says the same thing at every thread count.
//!
//! `superko separate` prints a body `tools/verify-results.sh` diffs, and a
//! results file records one run whose thread count is not in the body. A body
//! that moved with the thread count would make those files unfalsifiable, so
//! each check here compares a sweep run on one thread with the same sweep run
//! on two and on fourteen threads, twice over: the [`Sweep`] as a struct, node
//! counts included, and the lines [`Sweep::lines`] prints, byte for byte.
//!
//! Every comparison also pins what the sweep resolved, so that none passes
//! vacuously: a comparison of two sweeps that resolved nothing would say
//! nothing about the fold of resolved roots.
//!
//! What this establishes is that the code is thread-independent on the boards
//! and budgets named, and nothing about Go.

use superko_rules::config::{Dims, Suicide};
use superko_rules::table::RuleTable;
use superko_solve::separate::{Options, Sweep, sweep_on_above, sweep_with};

/// The thread counts compared with the one-thread sweep: one again (through
/// [`sweep_with`] rather than [`sweep_on_above`]), two, and the machine's
/// fourteen, which is more threads than 1×1 has roots.
const THREADS: [usize; 3] = [1, 2, 14];

/// Sweep a board on one thread and on each of [`THREADS`], hold every result
/// equal to the first, and return it.
fn same_at_every_thread_count(
    dims: Dims,
    suicide: Suicide,
    budget: Option<u64>,
    min_stones: u32,
) -> Sweep {
    let table = RuleTable::build(dims, suicide).expect("a small board");
    let serial = sweep_on_above(&table, budget, min_stones);
    let body = serial.lines(&table, budget).join("\n");
    for threads in THREADS {
        let opts = Options {
            budget,
            min_stones,
            threads,
        };
        let parallel = sweep_with(&table, opts);
        assert_eq!(
            parallel, serial,
            "{dims} {suicide} budget {budget:?} min-stones {min_stones}: \
             the sweep at {threads} threads differs from the sweep at one"
        );
        assert_eq!(
            parallel.lines(&table, budget).join("\n"),
            body,
            "{dims} {suicide} budget {budget:?} min-stones {min_stones}: \
             the body at {threads} threads differs from the body at one"
        );
    }
    serial
}

/// 1×1, 1×3 and 2×2 under both suicide conventions, with no budget: every
/// root resolves, so every root's values reach the fold.
#[test]
fn an_unbudgeted_sweep_is_the_same_at_every_thread_count() {
    for (rows, cols, roots) in [(1, 1, 6), (1, 3, 54), (2, 2, 162)] {
        for suicide in [Suicide::Forbid, Suicide::RemoveOwn] {
            let sweep = same_at_every_thread_count(Dims::new(rows, cols), suicide, None, 0);
            assert_eq!(sweep.roots, roots);
            assert_eq!(sweep.resolved, roots, "{rows}x{cols} {suicide}");
            assert_eq!(sweep.unresolved, 0);
        }
    }
}

/// 1×2 with suicide removed is a board with separating roots
/// (`results/separate-1x2-remove-own.txt` records four), so the minimum the
/// fold picks and the witness lines `Sweep::lines` adds for it are compared
/// too.
#[test]
fn a_separating_sweep_is_the_same_at_every_thread_count() {
    let sweep = same_at_every_thread_count(Dims::new(1, 2), Suicide::RemoveOwn, None, 0);
    assert_eq!(sweep.separating, 4);
    assert!(sweep.minimal.is_some());
}

/// The empty 1×5 board under a budget of 1 000 nodes per search leaves 82 of
/// the 486 roots unresolved, so the unresolved counts and the least
/// unresolved root are compared across thread counts.
#[test]
fn a_budgeted_sweep_is_the_same_at_every_thread_count() {
    let sweep = same_at_every_thread_count(Dims::new(1, 5), Suicide::Forbid, Some(1_000), 0);
    assert_eq!(sweep.roots, 486);
    assert_eq!(sweep.unresolved, 82);
    assert_eq!(sweep.resolved, 404);
    assert!(sweep.unresolved_least.is_some());
    assert!(sweep.unresolved_least_liberties.is_some());
}

/// A stone-count floor skips roots on some threads and searches them on
/// others; the skipped count and the domain it restricts are compared.
#[test]
fn a_floored_sweep_is_the_same_at_every_thread_count() {
    let sweep = same_at_every_thread_count(Dims::new(2, 2), Suicide::Forbid, None, 2);
    // Codes with fewer than two stones on 2×2: the empty board and the eight
    // one-stone boards, each with both colors to move.
    assert_eq!(sweep.skipped, 18);
    assert_eq!(sweep.resolved, 144);
}
