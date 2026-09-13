//! The transport a symmetric sweep relies on, checked against the plain
//! solver root by root, and the symmetric sweep checked against the plain
//! sweep and against the naive engine.
//!
//! Three kinds of check:
//!
//! 1. **Per-root invariance.** Every root of a board is searched with the
//!    plain solver, and for every root `r`, every board symmetry `s` and each
//!    choice of color swap, the value at the image of `r` is the value at `r`,
//!    negated when the swap is taken; and at every komi floor `k` of the range
//!    `tests/agreement.rs` asks about, for both colors `x`, "`x` wins `r` at
//!    `k`" equals "`x` wins `s(r)` at `k`" and "`x.other` wins `swap(r)` at
//!    `-k - 1`". Under the swap alone the search is the same search, so node
//!    counts, depths and ssk-only counts are held equal too.
//!
//! 2. **Differential.** The sweep with symmetry on against the sweep with it
//!    off, at one and fourteen threads.
//!
//! 3. **Naive agreement.** The value the symmetric sweep records at every root
//!    of every board of at most three points is the naive engine's.
//!
//! Every comparison that can skip a root — one search over its budget — counts
//! what it compared and pins the count, so no check passes vacuously.
//!
//! What this establishes is `computed`, on the boards and budgets named, and
//! nothing about any other board. It is the evidence the `board-symmetry` and
//! `color-swap` divergences cite for values; the evidence for the transition
//! table is `superko-rules`'s `tests/symmetry.rs`.

use superko_rules::code::{PosCode, code_space, decode};
use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::Color;
use superko_rules::symmetry::Symmetries;
use superko_rules::table::RuleTable;
use superko_solve::naive;
use superko_solve::search::{Solution, Solver};
use superko_solve::separate::{
    Options, Separating, Sweep, SymmetryMode, root_outcomes, sweep_with, transported_floor,
    transported_value,
};

const RULES: [Repetition; 2] = [Repetition::Psk, Repetition::Ssk];
const SUICIDES: [Suicide; 2] = [Suicide::Forbid, Suicide::RemoveOwn];

/// The place of a root in the sweep order.
fn place(code: PosCode, to_move: Color) -> usize {
    code.0 as usize * 2 + usize::from(to_move == Color::White)
}

/// The root at a place in the sweep order.
fn root_at(index: usize) -> (PosCode, Color) {
    let code = PosCode(u32::try_from(index / 2).expect("a code fits a u32"));
    let to_move = if index.is_multiple_of(2) {
        Color::Black
    } else {
        Color::White
    };
    (code, to_move)
}

/// Every image of a root other than the root under the identity: the place
/// of the image, whether the swap was taken, and the board symmetry.
fn images(sym: &Symmetries, index: usize) -> Vec<(usize, bool, usize)> {
    let (code, to_move) = root_at(index);
    let mut out = Vec::new();
    for g in 0..sym.order() {
        if g != 0 {
            out.push((place(sym.code(g, code), to_move), false, g));
        }
        out.push((place(sym.code(g, sym.swap(code)), to_move.other()), true, g));
    }
    out
}

/// The komi floors worth asking about, as in `tests/agreement.rs`.
fn floors(dims: Dims) -> std::ops::RangeInclusive<i64> {
    let span = i64::try_from(dims.point_count()).expect("point count fits an i64");
    (-span - 1)..=(span + 1)
}

/// Per-root value invariance on one board under one suicide convention, both
/// rules. Returns `(compared, skipped)`: pairs of a root and an image both
/// resolved within `budget`, and pairs where one was not.
fn values_are_invariant(dims: Dims, suicide: Suicide, budget: Option<u64>) -> (u64, u64) {
    let table = RuleTable::build(dims, suicide).expect("a small board");
    let sym = Symmetries::new(dims).expect("a small board");
    let count = code_space(dims) as usize * 2;
    let (mut compared, mut skipped) = (0u64, 0u64);
    for rep in RULES {
        let mut solver = Solver::new(&table, rep).with_budget(budget);
        let sols: Vec<Solution> = (0..count)
            .map(|i| {
                let (code, to_move) = root_at(i);
                solver.solve_root(code, to_move)
            })
            .collect();
        for (i, a) in sols.iter().enumerate() {
            for (j, swapped, g) in images(&sym, i) {
                let b = sols[j];
                if g == 0 {
                    // The swap alone: the same search, node for node.
                    assert_eq!(
                        (b.nodes, b.max_depth, b.ssk_only),
                        (a.nodes, a.max_depth, a.ssk_only),
                        "{dims} {rep} {suicide}: root {i} and its color swap {j} were \
                         searched differently"
                    );
                }
                match (a.value, b.value) {
                    (Some(v), Some(w)) => {
                        assert_eq!(
                            w,
                            transported_value(v, swapped),
                            "{dims} {rep} {suicide}: root {i} has value {v} and its image {j} \
                             under element {g}, swap {swapped}, has {w}"
                        );
                        compared += 1;
                    }
                    _ => skipped += 1,
                }
            }
        }
    }
    (compared, skipped)
}

/// Per-root verdict transport on one board under one suicide convention, both
/// rules: every root, every komi floor of [`floors`] whose transported floor
/// is also in the range, both colors, every image. Returns `(compared,
/// skipped)` as [`values_are_invariant`] does, counted per image and floor and
/// color.
fn verdicts_are_transported(dims: Dims, suicide: Suicide, budget: Option<u64>) -> (u64, u64) {
    let table = RuleTable::build(dims, suicide).expect("a small board");
    let sym = Symmetries::new(dims).expect("a small board");
    let count = code_space(dims) as usize * 2;
    let lo = *floors(dims).start();
    let width = floors(dims).count();
    let slot = |i: usize, k: i64, c: Color| {
        let fi = usize::try_from(k - lo).expect("a floor in range");
        (i * width + fi) * 2 + usize::from(c == Color::White)
    };
    let (mut compared, mut skipped) = (0u64, 0u64);
    for rep in RULES {
        let mut solver = Solver::new(&table, rep).with_budget(budget);
        let mut wins: Vec<Option<bool>> = vec![None; count * width * 2];
        for i in 0..count {
            let (code, to_move) = root_at(i);
            for k in floors(dims) {
                for c in [Color::Black, Color::White] {
                    wins[slot(i, k, c)] = solver.decide_root(code, to_move, k, c).wins;
                }
            }
        }
        for i in 0..count {
            for (j, swapped, g) in images(&sym, i) {
                for k in floors(dims) {
                    let k2 = transported_floor(k, swapped);
                    if !floors(dims).contains(&k2) {
                        continue;
                    }
                    for c in [Color::Black, Color::White] {
                        let c2 = if swapped { c.other() } else { c };
                        match (wins[slot(i, k, c)], wins[slot(j, k2, c2)]) {
                            (Some(a), Some(b)) => {
                                assert_eq!(
                                    a, b,
                                    "{dims} {rep} {suicide}: {c} wins root {i} at komi floor \
                                     {k} is {a}, and {c2} wins its image {j} under element \
                                     {g}, swap {swapped}, at {k2} is {b}"
                                );
                                compared += 1;
                            }
                            _ => skipped += 1,
                        }
                    }
                }
            }
        }
    }
    (compared, skipped)
}

/// Nodes a per-root search may spend in the checks of this file.
const BUDGET: Option<u64> = Some(1_000_000);

/// Every board of at most four points and 2×2, both orientations of each
/// line, both conventions, both rules: values. Every search resolves at the
/// budget, so every pair is compared: `2 · 3^(m·n)` roots times `2|G| - 1`
/// images, times two rules and two conventions.
#[test]
fn values_are_invariant_on_boards_of_four_points() {
    for (rows, cols, order) in [
        (1, 1, 1u64),
        (1, 2, 2),
        (2, 1, 2),
        (1, 3, 2),
        (3, 1, 2),
        (1, 4, 2),
        (4, 1, 2),
        (2, 2, 8),
    ] {
        let dims = Dims::new(rows, cols);
        let roots = u64::from(code_space(dims)) * 2;
        for suicide in SUICIDES {
            let (compared, skipped) = values_are_invariant(dims, suicide, BUDGET);
            assert_eq!(
                (compared, skipped),
                (roots * (2 * order - 1) * 2, 0),
                "{dims} {suicide}"
            );
        }
    }
}

/// 1×5 and 5×1, both conventions, both rules: values. Under the no-suicide
/// rule every search resolves at the budget; with suicide removing its own
/// stones many roots do not, and the pairs compared are pinned.
///
/// `cargo test --release -p superko-solve --test symmetry -- --ignored values_are_invariant_on_boards_of_five_points`
#[test]
#[ignore = "release only: the 1x5 remove-own searches run to their budget"]
fn values_are_invariant_on_boards_of_five_points() {
    const FIVE_BUDGET: Option<u64> = Some(100_000);
    for (rows, cols) in [(1, 5), (5, 1)] {
        let dims = Dims::new(rows, cols);
        let all = u64::from(code_space(dims)) * 2 * 3 * 2;
        assert_eq!(
            values_are_invariant(dims, Suicide::Forbid, FIVE_BUDGET),
            (all, 0),
            "{dims} forbid"
        );
        let (compared, skipped) = values_are_invariant(dims, Suicide::RemoveOwn, FIVE_BUDGET);
        assert_eq!(compared + skipped, all, "{dims} remove-own");
        assert_eq!(
            (compared, skipped),
            REMOVE_OWN_FIVE_VALUES,
            "{dims} remove-own: pairs compared and skipped at {FIVE_BUDGET:?} nodes"
        );
    }
}

/// Pairs compared and skipped on 1×5 (and, identically, 5×1) with suicide
/// removing its own stones, at 10⁵ nodes per search.
const REMOVE_OWN_FIVE_VALUES: (u64, u64) = (1_288, 1_628);

/// Every board of at most four points, 2×2 among them, both orientations of
/// each line, both conventions, both rules: verdicts at every komi floor of
/// [`floors`] whose transported floor is also in it, for both colors. Every
/// search resolves at the budget, and the comparisons made are pinned per
/// board — the same under both conventions, since the count is roots × images
/// × floors whose transport stays in range × colors × rules.
#[test]
fn verdicts_are_transported_on_boards_of_four_points() {
    for (rows, cols, pinned) in [
        (1, 1, 96u64),
        (1, 2, 1_368),
        (2, 1, 1_368),
        (1, 3, 5_400),
        (3, 1, 5_400),
        (1, 4, 20_088),
        (4, 1, 20_088),
        (2, 2, 101_736),
    ] {
        let dims = Dims::new(rows, cols);
        for suicide in SUICIDES {
            assert_eq!(
                verdicts_are_transported(dims, suicide, BUDGET),
                (pinned, 0),
                "{dims} {suicide}"
            );
        }
    }
}

/// 1×5 and 5×1, both conventions: verdicts at 10⁴ nodes per search. Under
/// the no-suicide rule every search resolves; with suicide removing its own
/// stones many do not. The comparisons made and skipped are pinned, the same
/// on both orientations.
///
/// `cargo test --release -p superko-solve --test symmetry -- --ignored verdicts_are_transported_on_boards_of_five_points`
#[test]
#[ignore = "release only: thirteen komi floors, two colors and two rules per root"]
fn verdicts_are_transported_on_boards_of_five_points() {
    const FIVE_BUDGET: Option<u64> = Some(10_000);
    for (rows, cols) in [(1, 5), (5, 1)] {
        let dims = Dims::new(rows, cols);
        for (suicide, pinned) in [
            (Suicide::Forbid, (71_928, 0)),
            (Suicide::RemoveOwn, (43_792, 28_136)),
        ] {
            assert_eq!(
                verdicts_are_transported(dims, suicide, FIVE_BUDGET),
                pinned,
                "{dims} {suicide}: comparisons made and skipped at {FIVE_BUDGET:?} nodes"
            );
        }
    }
}

/// A sweep with the fields symmetry legitimately changes set to what a plain
/// sweep has, so that the rest can be compared with `==`.
///
/// Changed legitimately, and so excluded: the node count, which counts only
/// searches that ran; `symmetry`, `searched` and `transported`, which say how
/// the sweep ran; each witness's `transported` flag; and the three ssk-only
/// counts, which a transported root copies from its representative's search,
/// and which a board symmetry can change by permuting the move order's
/// tie-break (`superko_solve::separate`'s module docs). They do change on the
/// boards below: on 1×3 with suicide removing its own stones the symmetric
/// sweep counts 168 ssk-only plays against the plain sweep's 172, and on 1×5
/// under the no-suicide rule 330 roots with one against 328. Nothing else is
/// excluded.
fn as_plain(sweep: &Sweep, plain: &Sweep) -> Sweep {
    let clear = |sep: Option<Separating>| {
        sep.map(|s| Separating {
            transported: false,
            ..s
        })
    };
    Sweep {
        nodes: plain.nodes,
        ssk_only_plays: plain.ssk_only_plays,
        roots_with_ssk_only: plain.roots_with_ssk_only,
        resolved_with_ssk_only: plain.resolved_with_ssk_only,
        symmetry: false,
        searched: plain.searched,
        transported: 0,
        mirrored_moves: false,
        mirrored_skips: 0,
        minimal: clear(sweep.minimal),
        minimal_liberties: clear(sweep.minimal_liberties),
        ..sweep.clone()
    }
}

/// The symmetric sweep of one board at one and fourteen threads, held equal to
/// each other and, on every field it does not legitimately change, to the
/// plain sweep; and every root's values held equal between the two where both
/// resolved; and every transported root held to carry its representative's
/// searches. Returns the roots whose values were compared, and the
/// transported roots whose carried situational search made an ssk-only play,
/// so that a caller can pin the carry as reached.
///
/// With `mirrored` the symmetric sweep also skips mirrored plays, both halves
/// of the feature on, as `superko separate --symmetry on` runs; the plain sweep
/// never does.
fn symmetric_sweep_agrees(
    dims: Dims,
    suicide: Suicide,
    budget: Option<u64>,
    mirrored: bool,
) -> (u64, u64) {
    let table = RuleTable::build(dims, suicide).expect("a small board");
    let plain_opts = Options {
        budget,
        ..Options::default()
    };
    let plain = sweep_with(&table, plain_opts);
    let sym_opts = Options {
        symmetry: true,
        mirrored_moves: mirrored,
        ..plain_opts
    };
    let on = sweep_with(&table, sym_opts);
    let spread = sweep_with(
        &table,
        Options {
            threads: 14,
            ..sym_opts
        },
    );
    assert_eq!(
        spread, on,
        "{dims} {suicide}: the symmetric sweep moved with threads"
    );
    assert_eq!(
        spread.lines(&table, budget),
        on.lines(&table, budget),
        "{dims} {suicide}"
    );
    assert!(on.symmetry && !plain.symmetry);
    assert_eq!(on.mirrored_moves, mirrored);
    assert!(!plain.mirrored_moves && plain.mirrored_skips == 0);
    assert!(
        on.transported > 0,
        "{dims} {suicide}: nothing was transported"
    );
    assert_eq!(on.searched + on.transported + on.skipped, on.roots);
    assert_eq!(plain.searched + plain.skipped, plain.roots);
    assert!(on.nodes <= plain.nodes || budget.is_some());

    if plain.unresolved == 0 && on.unresolved == 0 {
        assert_eq!(as_plain(&on, &plain), plain, "{dims} {suicide}");
    } else {
        assert!(
            budget.is_some(),
            "{dims} {suicide}: an unbudgeted sweep left a root unresolved"
        );
    }

    let a = root_outcomes(&table, plain_opts);
    let b = root_outcomes(&table, sym_opts);

    // The representative is its orbit's least root: a root is transported
    // exactly when some image of it comes earlier in the sweep order.
    let sym = Symmetries::new(dims).expect("a small board");
    for (i, y) in b.iter().enumerate() {
        let earlier = images(&sym, i).iter().any(|&(j, _, _)| j < i);
        assert_eq!(
            y.transported, earlier,
            "{dims} {suicide}: root {i} is transported {} but has an earlier image {earlier}",
            y.transported
        );
    }

    // A transported root carries its representative's two searches: the
    // representative is the least image, found here with this file's own
    // `images`, and the copy has its nodes, depth, ssk-only count and
    // resolution, with the value transported along a route that reaches it.
    // Without this a transport that zeroed the copied counts would agree with
    // the fold, which reads the same zeros.
    let mut carried = 0u64;
    for (i, y) in b.iter().enumerate().filter(|(_, y)| y.transported) {
        let routes = images(&sym, i);
        let j = routes
            .iter()
            .map(|&(j, _, _)| j)
            .min()
            .expect("a transported root has an image");
        let x = &b[j];
        assert!(
            !x.transported,
            "{dims} {suicide}: root {i}'s least image {j} was itself transported"
        );
        for (rep, from, to) in [("psk", x.psk, y.psk), ("ssk", x.ssk, y.ssk)] {
            let (Some(from), Some(to)) = (from, to) else {
                assert!(
                    from.is_none() && to.is_none(),
                    "{dims} {suicide} {rep}: root {i} and its representative {j} \
                     disagree on being skipped"
                );
                continue;
            };
            assert_eq!(
                (to.nodes, to.max_depth, to.ssk_only, to.value.is_some()),
                (
                    from.nodes,
                    from.max_depth,
                    from.ssk_only,
                    from.value.is_some()
                ),
                "{dims} {suicide} {rep}: root {i} did not carry its representative {j}'s search"
            );
            if let (Some(v), Some(w)) = (from.value, to.value) {
                assert!(
                    routes
                        .iter()
                        .any(|&(k, swapped, _)| k == j && w == transported_value(v, swapped)),
                    "{dims} {suicide} {rep}: root {i} has {w}, not a transport of \
                     {j}'s {v}"
                );
            }
            if rep == "ssk" && to.ssk_only > 0 {
                carried += 1;
            }
        }
    }

    // The ssk-only counts the symmetric sweep excludes from its comparison
    // with the plain sweep are still what the fold makes of its own per-root
    // results, every transported root counted.
    let ssk: Vec<Solution> = b.iter().filter_map(|o| o.ssk).collect();
    let resolved_with = b
        .iter()
        .filter(|o| {
            matches!((o.psk, o.ssk), (Some(p), Some(q)) if p.value.is_some() && q.value.is_some() && q.ssk_only > 0)
        })
        .count();
    assert_eq!(
        on.ssk_only_plays,
        ssk.iter().map(|s| s.ssk_only).sum::<u64>(),
        "{dims} {suicide}"
    );
    assert_eq!(
        on.roots_with_ssk_only,
        u64::try_from(ssk.iter().filter(|s| s.ssk_only > 0).count()).unwrap(),
        "{dims} {suicide}"
    );
    assert_eq!(
        on.resolved_with_ssk_only,
        u64::try_from(resolved_with).unwrap(),
        "{dims} {suicide}"
    );
    let mut compared = 0u64;
    for (x, y) in a.iter().zip(&b) {
        assert_eq!((x.code, x.to_move), (y.code, y.to_move));
        assert!(!x.transported);
        let (Some(xp), Some(xs), Some(yp), Some(ys)) = (x.psk, x.ssk, y.psk, y.ssk) else {
            continue;
        };
        if let (Some(v), Some(w)) = (xp.value, yp.value) {
            assert_eq!(v, w, "{dims} {suicide} psk at {:?}", (x.code, x.to_move));
        }
        if let (Some(v), Some(w)) = (xs.value, ys.value) {
            assert_eq!(v, w, "{dims} {suicide} ssk at {:?}", (x.code, x.to_move));
        }
        if xp.value.is_some() && xs.value.is_some() && yp.value.is_some() && ys.value.is_some() {
            compared += 1;
        }
    }
    assert_eq!(
        u64::try_from(b.iter().filter(|o| o.transported).count()).unwrap(),
        on.transported
    );
    (compared, carried)
}

/// Every board of at most five points, both orientations, both conventions,
/// with no budget except on 1×5 and 5×1 with suicide removing its own stones,
/// which is the ignored test below. Every root resolves, so every root is
/// compared.
#[test]
fn a_symmetric_sweep_agrees_with_the_plain_sweep() {
    assert_eq!(sweeps_of_five_points_agree(false), (CARRIED_SSK_ONLY, 0));
}

/// As [`a_symmetric_sweep_agrees_with_the_plain_sweep`], with mirrored moves
/// on as well. The carried roots with an ssk-only play and the plays skipped
/// are pinned.
#[test]
fn a_symmetric_sweep_with_mirrored_moves_agrees_with_the_plain_sweep() {
    assert_eq!(
        sweeps_of_five_points_agree(true),
        (MIRRORED_CARRIED_SSK_ONLY, MIRRORED_SWEEP_SKIPS)
    );
}

/// Transported roots whose carried situational search made an ssk-only play,
/// and plays skipped as mirror images, summed over the boards of
/// [`a_symmetric_sweep_with_mirrored_moves_agrees_with_the_plain_sweep`].
const MIRRORED_CARRIED_SSK_ONLY: u64 = 947;
const MIRRORED_SWEEP_SKIPS: u64 = 668;

/// The sweeps of [`a_symmetric_sweep_agrees_with_the_plain_sweep`], returning
/// the carried roots with an ssk-only play and the plays the symmetric sweeps
/// skipped.
fn sweeps_of_five_points_agree(mirrored: bool) -> (u64, u64) {
    let mut carried = 0u64;
    let mut skips = 0u64;
    for (rows, cols) in [
        (1, 1),
        (1, 2),
        (2, 1),
        (1, 3),
        (3, 1),
        (1, 4),
        (4, 1),
        (2, 2),
        (1, 5),
        (5, 1),
    ] {
        let dims = Dims::new(rows, cols);
        for suicide in SUICIDES {
            if rows * cols == 5 && suicide == Suicide::RemoveOwn {
                continue;
            }
            let roots = u64::from(code_space(dims)) * 2;
            let (compared, copied) = symmetric_sweep_agrees(dims, suicide, None, mirrored);
            assert_eq!(compared, roots, "{dims} {suicide}");
            carried += copied;
            if mirrored {
                let table = RuleTable::build(dims, suicide).expect("a small board");
                skips += sweep_with(
                    &table,
                    Options {
                        symmetry: true,
                        mirrored_moves: true,
                        ..Options::default()
                    },
                )
                .mirrored_skips;
            }
        }
    }
    (carried, skips)
}

/// Transported roots whose carried situational search made an ssk-only play,
/// summed over the boards of [`a_symmetric_sweep_agrees_with_the_plain_sweep`].
const CARRIED_SSK_ONLY: u64 = 947;

/// Which roots a symmetric sweep searches, pinned on one board: 1×3 under the
/// no-suicide rule. Its group of reversal and color swap has four elements;
/// the identity fixes all 54 roots, the reversal the 18 whose two end points
/// agree, and neither map with the swap fixes any, since each changes who
/// moves, so by Burnside's count there are (54 + 18) / 4 = 18 orbits.
#[test]
fn a_symmetric_sweep_searches_one_root_of_each_orbit() {
    let table = RuleTable::build(Dims::new(1, 3), Suicide::Forbid).expect("a small board");
    let on = sweep_with(
        &table,
        Options {
            symmetry: true,
            ..Options::default()
        },
    );
    assert_eq!((on.searched, on.transported), (18, 36));
}

/// A stone-count floor with symmetry on skips the roots it skips without it:
/// every field but those [`as_plain`] names is the plain sweep's, on 1×4 and
/// 2×2 under both conventions at a floor of two stones.
#[test]
fn a_symmetric_sweep_with_a_stone_floor_agrees_with_the_plain_sweep() {
    for (rows, cols) in [(1, 4), (2, 2)] {
        let dims = Dims::new(rows, cols);
        for suicide in SUICIDES {
            let table = RuleTable::build(dims, suicide).expect("a small board");
            let plain_opts = Options {
                min_stones: 2,
                ..Options::default()
            };
            let plain = sweep_with(&table, plain_opts);
            let on = sweep_with(
                &table,
                Options {
                    symmetry: true,
                    ..plain_opts
                },
            );
            assert!(
                plain.skipped > 0,
                "{dims} {suicide}: the floor skipped nothing"
            );
            assert_eq!(plain.unresolved, 0, "{dims} {suicide}");
            assert_eq!(on.searched + on.transported + on.skipped, on.roots);
            assert!(on.transported > 0, "{dims} {suicide}");
            assert_eq!(as_plain(&on, &plain), plain, "{dims} {suicide}");
        }
    }
}

/// 1×5 and 5×1 with suicide removing its own stones, at 10⁴ nodes per search:
/// roots are left unresolved, so only the roots resolved under both rules in
/// both sweeps are compared, and their number, 72 of 486 on each orientation,
/// is pinned.
///
/// `cargo test --release -p superko-solve --test symmetry -- --ignored a_budgeted_symmetric_sweep_agrees_with_the_plain_sweep`
#[test]
#[ignore = "release only: many searches run to their budget"]
fn a_budgeted_symmetric_sweep_agrees_with_the_plain_sweep() {
    for (rows, cols) in [(1, 5), (5, 1)] {
        let dims = Dims::new(rows, cols);
        assert_eq!(
            symmetric_sweep_agrees(dims, Suicide::RemoveOwn, Some(10_000), false).0,
            72,
            "{dims} remove-own"
        );
    }
}

/// As [`a_budgeted_symmetric_sweep_agrees_with_the_plain_sweep`], with mirrored
/// moves on as well. The roots compared — resolved under both rules in both
/// sweeps — are pinned. Mirrored moves could change which roots the symmetric
/// sweep resolves within the budget; on these boards the count is the same 72
/// as without them.
///
/// `cargo test --release -p superko-solve --test symmetry -- --ignored a_budgeted_symmetric_sweep_with_mirrored_moves_agrees_with_the_plain_sweep`
#[test]
#[ignore = "release only: many searches run to their budget"]
fn a_budgeted_symmetric_sweep_with_mirrored_moves_agrees_with_the_plain_sweep() {
    for (rows, cols) in [(1, 5), (5, 1)] {
        let dims = Dims::new(rows, cols);
        assert_eq!(
            symmetric_sweep_agrees(dims, Suicide::RemoveOwn, Some(10_000), true).0,
            72,
            "{dims} remove-own"
        );
    }
}

/// The value the symmetric sweep records at every root of the named boards,
/// both conventions, both rules, is the naive engine's value at that root.
/// Returns the roots compared and the transported roots among them.
fn symmetric_sweep_agrees_with_naive(boards: &[(usize, usize)]) -> (u64, u64) {
    let mut transported = 0u64;
    let mut roots = 0u64;
    for &(rows, cols) in boards {
        let dims = Dims::new(rows, cols);
        for suicide in SUICIDES {
            let table = RuleTable::build(dims, suicide).expect("a small board");
            let opts = Options {
                symmetry: true,
                ..Options::default()
            };
            for outcome in root_outcomes(&table, opts) {
                let board = decode(dims, outcome.code);
                for (rep, sol) in [
                    (Repetition::Psk, outcome.psk),
                    (Repetition::Ssk, outcome.ssk),
                ] {
                    let fast = sol.and_then(|s| s.value).expect("no budget, no floor");
                    let slow = naive::value_from(&board, outcome.to_move, rep, suicide);
                    assert_eq!(
                        fast, slow,
                        "{dims} {rep} {suicide} root {} {} to move (transported {}): \
                         sweep {fast} naive {slow}",
                        outcome.code, outcome.to_move, outcome.transported
                    );
                }
                roots += 1;
                transported += u64::from(outcome.transported);
            }
        }
    }
    (roots, transported)
}

/// 1×1, 1×2 and 2×1 against the naive engine, every root. The transported
/// roots are pinned, so the check reaches the transport.
#[test]
fn a_symmetric_sweep_agrees_with_the_naive_engine_on_two_points() {
    assert_eq!(
        symmetric_sweep_agrees_with_naive(&[(1, 1), (1, 2), (2, 1)]),
        (84, NAIVE_TWO_TRANSPORTED)
    );
}

/// Transported roots of 1×1, 1×2 and 2×1 under both conventions.
const NAIVE_TWO_TRANSPORTED: u64 = 54;

/// 1×3 and 3×1 against the naive engine, every root. The naive engine visits
/// the whole tree, which on three points takes about a minute in a debug
/// build.
///
/// `cargo test --release -p superko-solve --test symmetry -- --ignored a_symmetric_sweep_agrees_with_the_naive_engine_on_three_points`
#[test]
#[ignore = "release only: the naive engine on three points"]
fn a_symmetric_sweep_agrees_with_the_naive_engine_on_three_points() {
    assert_eq!(
        symmetric_sweep_agrees_with_naive(&[(1, 3), (3, 1)]),
        (216, NAIVE_THREE_TRANSPORTED)
    );
}

/// Transported roots of 1×3 and 3×1 under both conventions.
const NAIVE_THREE_TRANSPORTED: u64 = 144;

/// The four-element group under the solver, the group the 2×3 sweep runs
/// under: 2×3 and 3×2 under the no-suicide rule, both rules, every root, 10⁵
/// nodes a search. Against the plain sweep's per-root values
/// ([`root_outcomes`]): the sweep with mirrored moves, and the symmetric sweeps
/// with canonical roots, without and with mirrored moves, at every root and
/// rule both resolved. Per board and mode the comparisons made and not made,
/// the transported roots and the plays skipped are pinned, so no comparison
/// passes vacuously. Then, on each board, the value search from the empty root
/// with mirrored moves at 10⁴ nodes returns the same [`Solution`] with
/// [`Solver::with_unmatched_self_check`] as without it, and the recount runs,
/// which holds the incremental unmatched counts to the archive for the
/// half-turn and the two reversals.
///
/// `cargo test --release -p superko-solve --test symmetry -- --ignored the_four_element_group_agrees_with_the_plain_sweep_on_2x3_and_3x2`
#[test]
#[ignore = "release only: 1 458 roots a board, and many searches run to their budget"]
fn the_four_element_group_agrees_with_the_plain_sweep_on_2x3_and_3x2() {
    let threads = std::thread::available_parallelism().map_or(1, usize::from);
    let mut got = Vec::new();
    for (rows, cols) in [(2, 3), (3, 2)] {
        let dims = Dims::new(rows, cols);
        let table = RuleTable::build(dims, Suicide::Forbid).expect("a board within the table");
        let sym = Symmetries::new(dims).expect("a board within the table");
        assert_eq!(sym.order(), 4, "{dims}");
        let base = Options {
            budget: Some(100_000),
            threads,
            ..Options::default()
        };
        let plain = root_outcomes(&table, base);
        for mode in [SymmetryMode::Moves, SymmetryMode::Roots, SymmetryMode::On] {
            let outcomes = root_outcomes(
                &table,
                Options {
                    symmetry: mode.roots(),
                    mirrored_moves: mode.moves(),
                    ..base
                },
            );
            assert_eq!(outcomes.len(), plain.len(), "{dims} {}", mode.name());
            let (mut compared, mut uncompared, mut transported, mut skips) = (0u64, 0, 0, 0);
            for (a, b) in plain.iter().zip(&outcomes) {
                assert_eq!((a.code, a.to_move), (b.code, b.to_move));
                assert!(!a.transported);
                transported += u64::from(b.transported);
                for (rep, x, y) in [("psk", a.psk, b.psk), ("ssk", a.ssk, b.ssk)] {
                    let (x, y) = (x.expect("no floor"), y.expect("no floor"));
                    assert_eq!(x.mirrored_skips, 0);
                    if !b.transported {
                        skips += y.mirrored_skips;
                    }
                    match (x.value, y.value) {
                        (Some(v), Some(w)) => {
                            assert_eq!(
                                v,
                                w,
                                "{dims} {} {rep} root {} {} to move: {v} plain, {w} with symmetry",
                                mode.name(),
                                b.code,
                                b.to_move
                            );
                            compared += 1;
                        }
                        _ => uncompared += 1,
                    }
                }
            }
            got.push((
                format!("{dims} {}", mode.name()),
                compared,
                uncompared,
                transported,
                skips,
            ));
        }
        for rep in RULES {
            let unchecked = Solver::new(&table, rep)
                .with_budget(Some(10_000))
                .with_mirrored_moves(&sym)
                .solve_root(PosCode(0), Color::Black);
            let mut checked = Solver::new(&table, rep)
                .with_budget(Some(10_000))
                .with_mirrored_moves(&sym)
                .with_unmatched_self_check();
            let solution: Solution = checked.solve_root(PosCode(0), Color::Black);
            assert_eq!(solution, unchecked, "{dims} {rep}");
            assert!(checked.self_checks() > 0, "{dims} {rep}");
        }
    }
    let expected: Vec<(String, u64, u64, u64, u64)> = FOUR_ELEMENT_PINNED
        .iter()
        .map(|&(name, a, b, c, d)| (name.to_string(), a, b, c, d))
        .collect();
    assert_eq!(
        got, expected,
        "per board and mode: values compared, not compared, roots transported, plays skipped"
    );
}

/// Per board and mode of
/// [`the_four_element_group_agrees_with_the_plain_sweep_on_2x3_and_3x2`]: the
/// root-and-rule values compared and not compared, the roots transported, and
/// the plays skipped by the searches that ran.
const FOUR_ELEMENT_PINNED: [(&str, u64, u64, u64, u64); 6] = [
    ("2x3 moves", 1_504, 1_412, 0, 788),
    ("2x3 roots", 1_504, 1_412, 1_242, 0),
    ("2x3 on", 1_504, 1_412, 1_242, 228),
    ("3x2 moves", 1_504, 1_412, 0, 764),
    ("3x2 roots", 1_504, 1_412, 1_242, 0),
    ("3x2 on", 1_504, 1_412, 1_242, 216),
];
