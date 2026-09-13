//! The second row of the acceptance suite: the 1×n empty-board minimax scores
//! under positional superko.
//!
//! `docs/trusted-base.md` validates the definitions against quantities other
//! people computed independently. The game counts are one such quantity and
//! `superko_graph` reproduces them; this is the other one the fixture
//! `test_data/literature/linear-go-scores.toml` records — Weninger and
//! Hayward's table, attributed there and **transcribed second-hand**, with no
//! primary source held (C-24, `cited`, and C-11 for its disputed entry).
//!
//! The fixture's own header names the ruleset: Tromp rules, no suicide,
//! positional superko, no komi. `Suicide::Forbid` is that convention and is
//! `Defs.lean`'s, so the comparison is against the same rule on both sides —
//! unlike the 1×4 game count, where the suicide convention is observable
//! (C-36, `computed`).
//!
//! The table is transcribed here rather than parsed: the workspace has no TOML
//! parser and no dependencies, and a hand-rolled parser for nine rows would be
//! more code than the rows. The test that the two agree is
//! `the_fixture_still_says_what_this_test_says`, which greps the fixture, so a
//! change to it fails the build rather than leaving this list stale.
//!
//! # What this establishes
//!
//! `computed`, and only over the boards the solver reaches within the test's
//! budget. Reproducing a published table under the publication's own ruleset
//! is evidence that this workspace computes what that publication computed; it
//! is not evidence that either is a correct account of Go, and no kernel has
//! checked a score.

use superko_rules::code::PosCode;
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::Color;
use superko_rules::symmetry::Symmetries;
use superko_rules::table::RuleTable;
use superko_solve::search::Solver;

/// The fixture's rows, `(n, black)`, for the entries it states outright. The
/// disputed `n = 9` entry is not here: it has no single published value.
const PUBLISHED: [(usize, i32); 8] = [
    (1, 0),
    (2, 0),
    (3, 3),
    (4, 4),
    (5, 0),
    (6, 1),
    (7, 2),
    (8, 3),
];

/// How many nodes one score search may visit before the test gives up on that
/// board and records it as out of reach rather than failing.
///
/// A test that failed on a board the solver cannot reach would be a test of
/// the compute available. A test that silently stopped checking would be
/// worse: the assertion at the end holds the number of boards actually checked
/// to a floor, so shrinking reach breaks the build.
const BUDGET: u64 = 40_000_000;

/// The boards this test has reached, at the budget above. Raising it is a
/// change to the solver's reach and must be a visible one.
const REACHED: usize = 6;

#[test]
fn the_published_linear_scores_reproduce_as_far_as_the_solver_reaches() {
    reproduce(false);
}

/// The same rows, reached by the solver with mirrored moves on
/// (`superko_solve::search`'s module docs), at the same budget. The reach is
/// pinned to the same six boards; the node counts of the searches that
/// resolved are pinned too, so that the variant cannot quietly become the
/// plain search.
#[test]
fn the_published_linear_scores_reproduce_with_mirrored_moves() {
    let nodes = reproduce(true);
    assert_eq!(nodes, MIRRORED_NODES);
}

/// Nodes the mirrored search visited on each board it resolved, 1×1 to 1×6.
const MIRRORED_NODES: [u64; REACHED] = [3, 15, 39, 183, 1_125, 13_311_270];

/// Solve each published board, hold every resolved value to the table and the
/// number resolved to [`REACHED`], and return the node counts of the
/// resolved searches in board order.
fn reproduce(mirrored: bool) -> Vec<u64> {
    let mut checked = 0usize;
    let mut out_of_reach = Vec::new();
    let mut nodes = Vec::new();
    for (n, published) in PUBLISHED {
        let dims = Dims::new(1, n);
        let table = RuleTable::build(dims, Suicide::Forbid).expect("a board of at most 12 points");
        let sym = Symmetries::new(dims).expect("a board of at most 12 points");
        let mut solver = Solver::new(&table, Repetition::Psk).with_budget(Some(BUDGET));
        if mirrored {
            solver = solver.with_mirrored_moves(&sym);
        }
        let solved = solver.solve_root(PosCode(0), Color::Black);
        match solved.value {
            None => out_of_reach.push(n),
            Some(value) => {
                assert_eq!(
                    value, published,
                    "1x{n} under positional superko (mirrored moves {mirrored}): this workspace \
                     computes {value}, the \
                     table of test_data/literature/linear-go-scores.toml says {published}. \
                     One of the two is wrong and nothing downstream is worth anything until \
                     which is known."
                );
                checked += 1;
                nodes.push(solved.nodes);
            }
        }
    }
    assert_eq!(
        checked, REACHED,
        "the solver (mirrored moves {mirrored}) reached {checked} of the published rows, not \
         the {REACHED} this test records; {out_of_reach:?} were out of reach at a budget of \
         {BUDGET} nodes"
    );
    nodes
}

/// The transcription above is the fixture's, still.
#[test]
fn the_fixture_still_says_what_this_test_says() {
    let text = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../test_data/literature/linear-go-scores.toml"
    ))
    .expect("the fixture is in the repository");
    for (n, black) in PUBLISHED {
        let row = format!("n = {n}\nblack = {black}");
        assert!(
            text.contains(&row),
            "the fixture no longer carries `{row}`; PUBLISHED in this test is stale"
        );
    }
    assert!(
        text.contains("disputed = true"),
        "the fixture no longer marks the n = 9 entry disputed; C-11 may have moved"
    );
}
