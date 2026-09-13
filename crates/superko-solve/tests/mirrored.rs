//! Mirrored moves, checked against the solver without them root by root.
//!
//! [`Solver::with_mirrored_moves`] skips a play at a state a board symmetry
//! fixes, archive included, when the symmetry maps an earlier play onto it.
//! The skip is sound if the rules commute with the board's symmetries, which
//! is not proved (`superko_solve::search`'s module docs). What this file
//! checks instead:
//!
//! 1. **Differential.** For every root of a board, both rules, both suicide
//!    conventions: the value with mirrored moves equals the value without,
//!    and so does each color's verdict at every komi floor from `-(m·n) - 1`
//!    to `m·n + 1`. Debug runs cover 1×1 to 1×4 and 2×2; 2×1 to 4×1, 1×5 and
//!    5×1 are in an ignored release test. On every board but five-point ones
//!    under suicide removing its own stones the mirrored solver runs with
//!    [`Solver::with_unmatched_self_check`] on its value searches, and on the
//!    lines of at most four points on its verdict searches too, which recounts
//!    every symmetry's unmatched archived keys at every node, so the
//!    incremental counts are held to the archive on 2×2, whose quarter turns
//!    are not their own inverses.
//!
//! 2. **Non-vacuity.** Plays are skipped on the empty 1×4 and 2×2 boards, and
//!    the self-check runs, so the differential compares searches that differ.
//!
//! Every comparison that can skip a root — one search over its budget — counts
//! what it compared and pins the count, so no check passes vacuously.
//!
//! Naive agreement with mirrored moves on is in `tests/agreement.rs`, beside
//! the naive agreement without them; the sweep with mirrored moves is in
//! `tests/symmetry.rs`; the published 1×n values with them are in
//! `tests/published.rs`.
//!
//! What this establishes is `computed`, on the boards and budgets named, and
//! nothing about any other board.

use superko_rules::code::{PosCode, code_space};
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::Color;
use superko_rules::symmetry::Symmetries;
use superko_rules::table::RuleTable;
use superko_solve::search::Solver;

const RULES: [Repetition; 2] = [Repetition::Psk, Repetition::Ssk];
const SUICIDES: [Suicide; 2] = [Suicide::Forbid, Suicide::RemoveOwn];

/// The komi floors worth asking about, as in `tests/agreement.rs`.
fn floors(dims: Dims) -> std::ops::RangeInclusive<i64> {
    let span = i64::try_from(dims.point_count()).expect("point count fits an i64");
    (-span - 1)..=(span + 1)
}

/// What one differential run compared.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Compared {
    /// Roots whose value both searches resolved, and roots where one did not.
    values: (u64, u64),
    /// Verdicts, per root, komi floor and color, both searches resolved, and
    /// verdicts where one did not.
    verdicts: (u64, u64),
    /// Plays the mirrored searches skipped, value and verdict searches both.
    skips: u64,
}

/// The differential on one board under one suicide convention, both rules:
/// every root's value, and with `verdicts` every root's verdicts at every komi
/// floor for both colors, with mirrored moves on and off, held equal wherever
/// both resolved within `budget`.
fn differential(
    dims: Dims,
    suicide: Suicide,
    budget: Option<u64>,
    verdicts: bool,
    self_check: bool,
) -> Compared {
    let table = RuleTable::build(dims, suicide).expect("a small board");
    let sym = Symmetries::new(dims).expect("a small board");
    let mut out = Compared::default();
    for rep in RULES {
        let mut plain = Solver::new(&table, rep).with_budget(budget);
        let mut mirrored = Solver::new(&table, rep)
            .with_budget(budget)
            .with_mirrored_moves(&sym);
        if self_check {
            mirrored = mirrored.with_unmatched_self_check();
        }
        for raw in 0..code_space(dims) {
            let code = PosCode(raw);
            for to_move in [Color::Black, Color::White] {
                let a = plain.solve_root(code, to_move);
                let b = mirrored.solve_root(code, to_move);
                assert_eq!(a.mirrored_skips, 0);
                out.skips += b.mirrored_skips;
                match (a.value, b.value) {
                    (Some(v), Some(w)) => {
                        assert_eq!(
                            v, w,
                            "{dims} {rep} {suicide} root {code} {to_move} to move: value {v} \
                             without mirrored moves and {w} with them"
                        );
                        out.values.0 += 1;
                    }
                    _ => out.values.1 += 1,
                }
                if !verdicts {
                    continue;
                }
                for k in floors(dims) {
                    for c in [Color::Black, Color::White] {
                        let x = plain.decide_root(code, to_move, k, c);
                        let y = mirrored.decide_root(code, to_move, k, c);
                        assert_eq!(x.mirrored_skips, 0);
                        out.skips += y.mirrored_skips;
                        match (x.wins, y.wins) {
                            (Some(p), Some(q)) => {
                                assert_eq!(
                                    p, q,
                                    "{dims} {rep} {suicide} root {code} {to_move} to move: {c} \
                                     wins at komi floor {k} is {p} without mirrored moves and \
                                     {q} with them"
                                );
                                out.verdicts.0 += 1;
                            }
                            _ => out.verdicts.1 += 1,
                        }
                    }
                }
            }
        }
        if self_check {
            assert!(
                mirrored.self_checks() > 0,
                "{dims} {rep} {suicide}: the self-check never ran"
            );
        }
    }
    out
}

/// Nodes a search may spend in the debug differential. Every search of these
/// boards resolves within it, so every root is compared.
const BUDGET: Option<u64> = Some(1_000_000);

/// One line board, both conventions, both rules: values, and verdicts at
/// every komi floor for both colors, with the self-check on. Everything
/// resolves, so every value and verdict is compared. Returns the plays
/// skipped under the no-suicide rule and with suicide removing its own stones.
fn a_checked_board_agrees(dims: Dims) -> (u64, u64) {
    let mut skips = [0u64; 2];
    for (slot, suicide) in SUICIDES.into_iter().enumerate() {
        let c = differential(dims, suicide, BUDGET, true, true);
        assert_eq!(
            (c.values, c.verdicts),
            everything(dims),
            "{dims} {suicide}: comparisons made and skipped"
        );
        skips[slot] = c.skips;
    }
    (skips[0], skips[1])
}

/// Every value and every verdict of a board compared, none skipped, both rules.
fn everything(dims: Dims) -> ((u64, u64), (u64, u64)) {
    let roots = u64::from(code_space(dims)) * 2 * 2;
    let per_root = u64::try_from(floors(dims).count()).unwrap() * 2;
    ((roots, 0), (roots * per_root, 0))
}

/// 1×1, 1×2 and 1×3 as [`a_checked_board_agrees`] checks them, with the plays
/// skipped pinned per board. 1×1 has no symmetry but the identity and skips
/// nothing. The same boards turned on their side are in the ignored test
/// [`mirrored_moves_agree_on_turned_lines_and_boards_of_five_points`].
#[test]
fn mirrored_moves_agree_on_lines_of_three_points() {
    let got: Vec<_> = LINE_SKIPS[..3]
        .iter()
        .map(|&(n, _, _)| {
            let (forbid, remove_own) = a_checked_board_agrees(Dims::new(1, n));
            (n, forbid, remove_own)
        })
        .collect();
    assert_eq!(got, LINE_SKIPS[..3], "plays skipped per board");
}

/// 1×4 as [`a_checked_board_agrees`] checks it, with the plays skipped pinned.
#[test]
fn mirrored_moves_agree_on_1x4() {
    let (_, forbid, remove_own) = LINE_SKIPS[3];
    assert_eq!(
        a_checked_board_agrees(Dims::new(1, 4)),
        (forbid, remove_own)
    );
}

/// Plays skipped on 1×n, for n from one to four, under the no-suicide rule and
/// with suicide removing its own stones, value and verdict searches both. The
/// counts are the same on n×1.
const LINE_SKIPS: [(usize, u64, u64); 4] = [(1, 0, 0), (2, 64, 64), (3, 128, 356), (4, 224, 264)];

/// 2×2, the one square board a debug build reaches and the one whose quarter
/// turns are not their own inverses, under one convention, both rules: values
/// with the self-check on, then values and verdicts at every komi floor for
/// both colors with it off, which a debug build cannot afford with the recount
/// at every node of every verdict search. Everything resolves. Returns the
/// plays the second run skipped, value and verdict searches both.
fn the_square_agrees(suicide: Suicide) -> u64 {
    let dims = Dims::new(2, 2);
    let checked = differential(dims, suicide, BUDGET, false, true);
    assert_eq!(checked.values, everything(dims).0, "{dims} {suicide}");
    assert!(checked.skips > 0, "{dims} {suicide}");
    let c = differential(dims, suicide, BUDGET, true, false);
    assert_eq!((c.values, c.verdicts), everything(dims), "{dims} {suicide}");
    c.skips
}

/// [`the_square_agrees`] under the no-suicide rule, the plays skipped pinned.
#[test]
fn mirrored_moves_agree_on_2x2_without_suicide() {
    assert_eq!(the_square_agrees(Suicide::Forbid), 4_568);
}

/// [`the_square_agrees`] with suicide removing its own stones, the plays
/// skipped pinned.
#[test]
fn mirrored_moves_agree_on_2x2_with_suicide_removing_its_own_stones() {
    assert_eq!(the_square_agrees(Suicide::RemoveOwn), 6_944);
}

/// 2×1, 3×1 and 4×1 as [`a_checked_board_agrees`] checks them, each skipping
/// what its line on its side skips.
fn turned_lines_agree() {
    for (n, forbid, remove_own) in LINE_SKIPS {
        assert_eq!(
            a_checked_board_agrees(Dims::new(n, 1)),
            (forbid, remove_own),
            "{n}x1"
        );
    }
}

/// The lines of at most four points turned on their side (as
/// [`turned_lines_agree`]), then 1×5 and 5×1, both conventions, both rules:
/// values at 10⁵ nodes per search,
/// and values and verdicts at 10⁴. The self-check runs on the value searches
/// under the no-suicide rule, where every search resolves in a few thousand
/// nodes, and not under the rest, where a recount at every node of the searches
/// that run to their budget costs minutes even in release. Under the no-suicide rule every
/// search resolves; with suicide removing its own stones many do not, and the
/// comparisons made and skipped are pinned, the same on both orientations.
///
/// `cargo test --release -p superko-solve --test mirrored -- --ignored mirrored_moves_agree_on_turned_lines_and_boards_of_five_points`
#[test]
#[ignore = "release only: thirteen komi floors, two colors and two rules per root, and budgets run out"]
fn mirrored_moves_agree_on_turned_lines_and_boards_of_five_points() {
    turned_lines_agree();
    for (rows, cols) in [(1, 5), (5, 1)] {
        let dims = Dims::new(rows, cols);
        let got: Vec<_> = SUICIDES
            .into_iter()
            .map(|suicide| {
                let check = suicide == Suicide::Forbid;
                let v = differential(dims, suicide, Some(100_000), false, check);
                let w = differential(dims, suicide, Some(10_000), true, false);
                (suicide, v.values, (w.verdicts, v.skips + w.skips))
            })
            .collect();
        assert_eq!(
            got, FIVE_PINNED,
            "{dims}: values compared and skipped at 10^5 nodes, verdicts compared and skipped \
             at 10^4, and plays skipped"
        );
    }
}

/// A convention, the value roots compared and skipped, and the verdicts
/// compared and skipped with the plays skipped.
type FiveRow = (Suicide, (u64, u64), ((u64, u64), u64));

/// Per convention on 1×5 (and identically 5×1): value roots compared and
/// skipped at 10⁵ nodes a search; verdicts compared and skipped at 10⁴, with the
/// plays skipped by both runs.
const FIVE_PINNED: [FiveRow; 2] = [
    (Suicide::Forbid, (972, 0), ((25_272, 0), 1_736)),
    (Suicide::RemoveOwn, (440, 532), ((15_976, 9_296), 1_406)),
];

/// The empty 1×4 and 2×2 boards, Black to move, under the no-suicide rule and
/// both repetition rules: the value, and Black's verdict at the komi floor
/// equal to the value, with mirrored moves equal those without, plays are
/// skipped in both searches, fewer nodes are visited, and the self-check runs.
/// The counts are pinned.
#[test]
fn mirrored_moves_skip_plays_on_the_empty_1x4_and_2x2_boards() {
    let mut got = Vec::new();
    for (rows, cols) in [(1, 4), (2, 2)] {
        let dims = Dims::new(rows, cols);
        let table = RuleTable::build(dims, Suicide::Forbid).expect("a small board");
        let sym = Symmetries::new(dims).expect("a small board");
        for rep in RULES {
            let mut plain = Solver::new(&table, rep);
            let mut mirrored = Solver::new(&table, rep)
                .with_mirrored_moves(&sym)
                .with_unmatched_self_check();
            let empty = PosCode(0);
            let a = plain.solve_root(empty, Color::Black);
            let b = mirrored.solve_root(empty, Color::Black);
            assert_eq!(a.value, b.value, "{dims} {rep}");
            assert!(b.mirrored_skips > 0, "{dims} {rep}: nothing skipped");
            assert!(b.nodes < a.nodes, "{dims} {rep}: no node saved");
            // At the value itself Black does not win, so the verdict search
            // tries every Black play at the root and reaches the skipped ones.
            let floor = i64::from(a.value.expect("no budget"));
            let x = plain.decide_root(empty, Color::Black, floor, Color::Black);
            let y = mirrored.decide_root(empty, Color::Black, floor, Color::Black);
            assert_eq!(x.wins, Some(false), "{dims} {rep}");
            assert_eq!(y.wins, Some(false), "{dims} {rep}");
            assert!(y.mirrored_skips > 0, "{dims} {rep}: no verdict skip");
            assert!(mirrored.self_checks() > 0, "{dims} {rep}");
            got.push((
                format!("{dims} {rep}"),
                b.value.unwrap(),
                a.nodes,
                b.nodes,
                b.mirrored_skips,
                y.mirrored_skips,
            ));
        }
    }
    let expected: Vec<(String, i32, u64, u64, u64, u64)> = EMPTY_PINNED
        .iter()
        .map(|&(name, v, a, b, s, t)| (name.to_string(), v, a, b, s, t))
        .collect();
    assert_eq!(got, expected);
}

/// Per board and rule: the value, the nodes without and with mirrored moves,
/// and the plays the value search and the verdict search skipped.
const EMPTY_PINNED: [(&str, i32, u64, u64, u64, u64); 4] = [
    ("1x4 psk", 4, 303, 183, 4, 2),
    ("1x4 ssk", 4, 303, 183, 4, 2),
    ("2x2 psk", 1, 981, 342, 14, 5),
    ("2x2 ssk", 1, 1101, 378, 14, 5),
];
