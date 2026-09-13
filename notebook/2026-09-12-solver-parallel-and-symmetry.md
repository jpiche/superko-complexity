# 2026-09-12 — superko-solve: the bench, parallel sweeps and symmetry

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`) wrote
`docs/plans/solver-plan.md`. Claude Opus 5 (`claude-opus-5`) ran the session:
a main session orchestrated workflows whose Opus 5 subagents implemented,
reviewed and fixed each feature and wrote these records. The main session
wrote the briefs and the corrections to the plan, checked each result between
workflows, and ran the final gate.

The first session on [`../docs/plans/solver-plan.md`](../docs/plans/solver-plan.md).
In scope: the `bench` subcommand (plan §3 and §5.5), parallel sweeps (§2.1) and
symmetry, both halves (§2.2). Out of scope: §2.3 to §2.6.

**Status of everything here.** Nothing below enters the ledger above
`computed`, and no ledger row changed. The symmetry facts the new features rely
on are unlicensed divergences: tested exhaustively on the transition tables of
the boards named below, and on solver values and verdicts (at komi floors
`-(m·n) - 1` to `m·n + 1`) on boards of at most five points with the exceptions
under "What is not established"; not proved.
Every bench timing is a single release run on the 14-core, 36 GiB machine,
with the noise unmeasured.

## What was implemented, feature by feature

`git log feed6ce~1..51d9914`:

| commit | subject | review |
|---|---|---|
| `feed6ce` | docs: the plan for extending superko-solve's reach to 2x3 and 1x9 | the plan, committed verbatim by the main session |
| `0490183` | cli: add the superko bench subcommand and record a baseline measurement | refused in round 1, approved in round 2 |
| `2bf8ef4` | solve: sweep output does not depend on the thread count | approved in round 1 |
| `09dcc60` | solve: search one root per symmetry orbit in a sweep when symmetry is on | refused in rounds 1 and 2, approved in round 3 |
| `51d9914` | solve: skip plays mirrored by a board symmetry the state is symmetric under | approved in round 1 |

One Opus 5 subagent implemented each feature. Two or three Opus 5 reviewers
reviewed it, each through one lens: the plan's protections (§4), the tests (§5),
and for the two symmetry features, soundness. An Opus 5 fixer answered each
refusal, and an Opus 5 commit agent committed.

1. **`superko bench`** (`crates/superko-cli/src/bench.rs`). A fixed suite under
   `Suicide::Forbid`: the empty 1×5, 1×6, 1×7 and 2×3 boards under PSK and SSK
   at 10⁸ nodes, and a 2×3 separation sweep at 10⁶ nodes per search. It prints a
   flags line, then one `key=value` line per case with wall seconds. The output
   is a measurement for `data/bench/`, not a results body.
2. **Parallel sweeps** (`crates/superko-solve/src/separate.rs`).
   `Options { budget, min_stones, threads, symmetry }` and `sweep_with`.
   Worker threads each own a PSK and an SSK solver over the shared
   `&RuleTable`, claim roots from a shared counter, and store each root's
   result at its place in the sweep. One fold builds the `Sweep` in sweep
   order. `superko separate --threads N`, default 1; the thread count goes to
   standard error and never into the body. `tests/threads.rs` holds sweeps
   and bodies equal at one, two and fourteen threads on 1×1, 1×3 and 2×2 under
   both conventions, 1×2 with suicide removing its own stones, 1×5 under the
   no-suicide rule at 1 000 nodes per search, and 2×2 at a floor of two
   stones with no budget. A bench test in `crates/superko-cli/src/bench.rs`
   runs the 2×2 sweep at 50 nodes per search on 2 and 14 threads. Outside the
   tests, the 1×5 body at 14 threads was byte-identical
   to `results/separate-1x5-forbid.txt` (`2bf8ef4`'s commit body), and so was
   the budgeted 2×3 body to its file (below). Thread invariance is `computed`
   on those cases only, and not proved.
3. **Canonical roots** (`crates/superko-rules/src/symmetry.rs`, `separate.rs`).
   Each board's symmetry group as position-code permutations, plus the color
   swap, built the slow and readable way (decode, permute or swap, encode).
   They are registered as the divergences `board-symmetry` and `color-swap`, so
   `Divergence::ALL` has seven entries. With symmetry on, the sweep searches the
   least sweep index of each orbit under G × {id, swap}. Every other root gets
   its representative's `Solution`s, with the value negated under the swap.
4. **Mirrored moves** (`crates/superko-solve/src/search.rs`,
   `Solver::with_mirrored_moves`). Each non-identity board symmetry `s` keeps
   the count `u_s` of archived keys whose image is not archived. A state is
   `s`-symmetric when `u_s = 0` and `s(code) = code`. At a state whose subgroup
   `H` of such symmetries is nontrivial, a play that some element of `H` maps
   from an earlier play in the ordered list is skipped; the pass never is. Both
   the value search and the verdict search skip.
   `Solver::with_unmatched_self_check` recounts `u_s` at every node.

**Two departures from the plan besides the corrections.** Plan §2.2 says
"every root still appears in the output, marked `transported`"; the
implementation marks only the witness roots, in `<prefix>-transported=`,
because correction 6 forbids new body lines per root. Plan §2.1 asks for
`--threads` on `solve` as well; it belongs to §2.6's root split, which is out
of scope, so `solve` rejects `--threads`.

**Correction 9's protections held.** `git diff --stat feed6ce..51d9914 --
crates/superko-solve/src/naive.rs crates/superko-rules/src/archive.rs` prints
nothing: neither file changed.

On the command line, `superko separate --symmetry on` turns on both halves,
and `superko solve --symmetry on` turns on mirrored moves alone. `superko
bench --symmetry off|roots|moves|on` can select either half, so from the CLI
only `bench` can run canonical roots without mirrored moves.

## Corrections the main session made to the plan

The plan file is left as written. These override it where they conflict.

1. **The symmetry group, which §2.2 gets wrong.** §2.2 gives "two elements for
   a line or a non-square rectangle". A non-square rectangle with both sides at
   least 2 has **four** symmetries: identity, the two reflections, the
   half-turn. 1×1 has one, a line 1×n or n×1 with n ≥ 2 has two, and a square
   n×n with n ≥ 2 has eight. The diagonal reflections and quarter turns exist
   only when m = n. Every map used must be shown by test to commute with the
   transition table: `succ`, `playable` and `area`, for every code, color and
   point.
2. **The color swap and the verdict transport.** The table must satisfy
   `succ(swap c, x.other, p) = swap(succ(c, x, p))`, and likewise for
   `playable` and `area`, shown exhaustively by test. The value negates under
   the swap. `winner_at` makes Black win at floor `k` exactly when
   `k < (Black area − White area)`. So "`x` wins `(c, t)` at floor `k`" holds
   exactly when "`x.other` wins `(swap c, t.other)` at floor `−k−1`". Board
   symmetries leave values and verdicts unchanged. This transport is exact for
   the integer test and does not rest on C-55.
3. **Where the divergence markers live.** `tools/check-mirror.sh` scans only
   `crates/superko-rules/src`, so the code maps live in a new `superko-rules`
   module carrying the markers `board-symmetry` and `color-swap`. That departs
   from plan §4's list of what `superko-rules` may gain, and the module doc
   says so. Every prose count of divergences was updated.
4. **Mirrored moves use board symmetries only**, never the color swap, and the
   unmatched count must be right for maps that are not involutions (a square's
   quarter turns). Inserting a new key `k` changes `u_s` by
   `+[s(k) ≠ k and s(k) ∉ A] − [s⁻¹(k) ≠ k and s⁻¹(k) ∈ A]`, with `A` taken
   before the insert. The root key counts. Keys are archive keys, projected to
   Black under PSK. Only the first play of each `H`-orbit is searched, and the
   pass is never skipped. Plan §2.2 described one reflection σ, a per-pair skip
   and an involutive count.
5. **The sweep compares values only.** The witness verdicts in `Sweep::lines`
   are separate searches on the actual root, so the sweep transports no
   verdict. Tests exercise the verdict transport of correction 2.
6. **Output compatibility.** With default flags, and with symmetry off at any
   thread count, `superko separate` prints a body byte-identical to every
   existing `results/separate-*.txt`, and `superko solve` prints its current
   body. The thread count belongs in a `# produced-with:` header, never in a
   body. (Plan §3 wanted every flag in the header of a headline number.)
7. **Defaults.** Threads default to 1 and symmetry to off, whatever the bench
   shows, because turning symmetry on by default would change the
   `divergences=` line of every existing witness body. The bench numbers go to
   this notebook with a recommendation, and the maintainer decides.
8. **Test time.** Debug `cargo test --workspace` took about 154 s before the
   session (`notebook/2026-09-12-c17-separation-search.md`), and the session
   was to add under about 30 s. A test too slow for debug gets `#[ignore]`
   with its exact release command in its doc comment. A budgeted test pins the
   number of roots it compares.
9. **Protected and out of scope.** No rerun of the 2×3 sweep beyond the bench
   case at 10⁶ nodes per search, and no 1×9. No change to `naive.rs`, to the
   archive's exactness, to `MoveOrder::Static` or the reversed-order test, to
   the node budget's meaning, or to the verdict recursion's branch structure.

## The bench, before and after each feature

Each file was produced by `cargo build --release -p superko-cli`, then
`target/release/superko bench` with the flags its header line records, output
to `data/bench/<file>`. Files A to H were run on the working tree before the
commit named, not at it: by file time, `0-baseline` at 19:29 against `0490183`
at 19:37, `1-parallel` at 19:48 against `2bf8ef4` at 19:55, `2-roots` at 20:17
against `09dcc60` at 20:51, and `3-moves` at 21:36 to 21:37 against `51d9914`
at 21:48. D and E predate canonical roots' two fix rounds; their sweep nodes
equal J's, run after the commit, so the searched code looks unchanged, which
is a reading of equal counts and not a check. Only I and J were run at the
commit, after the gate. The exact invocations were not logged
beyond those headers. `0-baseline` predates `--threads` and ran on one thread.
**`2-roots-*` carry the header `symmetry=on`, which at `09dcc60` meant canonical
roots only.** From `51d9914` on, `on` means both halves. The rows below are
labeled by what each run measured, not by its header spelling:

| label | file | commit (A to H: the working tree before it) | threads | measured |
|---|---|---|---|---|
| A | `0-baseline.txt` | `0490183` | 1 | plain |
| B | `1-parallel-t1.txt` | `2bf8ef4` | 1 | plain |
| C | `1-parallel-t14.txt` | `2bf8ef4` | 14 | plain |
| D | `2-roots-t1.txt` | `09dcc60` | 1 | roots |
| E | `2-roots-t14.txt` | `09dcc60` | 14 | roots |
| F | `3-moves-only-t1.txt` | `51d9914` | 1 | moves |
| G | `3-moves-t1.txt` | `51d9914` | 1 | both |
| H | `3-moves-t14.txt` | `51d9914` | 14 | both |
| I | `4-gate-off-t14.txt` | `51d9914` | 14 | plain (main session, after the gate) |
| J | `4-gate-roots-t14.txt` | `51d9914` | 14 | roots (main session, after the gate) |

### The 2×3 sweep, 10⁶ nodes per search, 1 458 roots

| run | measured | threads | seconds | nodes | resolved / unresolved | searched / transported | ssk-only plays | resolved with an ssk-only play | mirrored skips |
|---|---|---|---|---|---|---|---|---|---|
| A | plain | 1 | 39.351 | 1 409 326 892 | 754 / 704 | — | 6 558 702 | 146 | — |
| B | plain | 1 | 39.349 | 1 409 326 892 | 754 / 704 | — | 6 558 702 | 146 | — |
| C | plain | 14 | 3.977 | 1 409 326 892 | 754 / 704 | — | 6 558 702 | 146 | — |
| D | roots | 1 | 5.791 | 204 561 993 | 760 / 698 | 216 / 1 242 | 6 911 700 | 152 | — |
| E | roots | 14 | 0.564 | 204 561 993 | 760 / 698 | 216 / 1 242 | 6 911 700 | 152 | — |
| F | moves | 1 | 62.609 | 1 409 314 340 | 754 / 704 | 1 458 / 0 | 6 558 758 | 146 | 788 |
| G | both | 1 | 9.061 | 204 558 495 | 760 / 698 | 216 / 1 242 | 6 911 736 | 152 | 228 |
| H | both | 14 | 0.878 | 204 558 495 | 760 / 698 | 216 / 1 242 | 6 911 736 | 152 | 228 |
| I | plain | 14 | 4.166 | 1 409 326 892 | 754 / 704 | 1 458 / 0 | 6 558 702 | 146 | 0 |
| J | roots | 14 | 0.610 | 204 561 993 | 760 / 698 | 216 / 1 242 | 6 911 700 | 152 | 0 |

No run found a separating root. Under a budget a transported root is resolved
exactly when its representative is, so canonical roots change which roots
resolve (754 to 760) and the ssk-only counts. Mirrored moves change those too.
None of these runs is a results body.

### The empty boards, 10⁸ nodes per search

Canonical roots and threads do not touch a single search, so these cases
differ only with mirrored moves:

| case | plain (A to E, I, J) | mirrored moves (F, G, H) |
|---|---|---|
| 1×5 PSK | 0 in 2 115 nodes | 0 in 1 125 nodes, 8 skips |
| 1×5 SSK | 0 in 4 407 nodes, ssk-only 62 | 0 in 2 373 nodes, ssk-only 34, 8 skips |
| 1×6 PSK | 1 in 27 925 122 nodes | 1 in 13 311 270 nodes, 6 skips |
| 1×6 SSK | unresolved, ssk-only 2 292 947 | unresolved, ssk-only 2 292 947, 0 skips |
| 1×7 PSK | unresolved | unresolved, 0 skips |
| 1×7 SSK | unresolved, ssk-only 2 711 477 | unresolved, ssk-only 2 711 477, 0 skips |
| 2×3 PSK | unresolved, max depth 99 | **0 in 93 137 907 nodes**, max depth 99, 22 skips |
| 2×3 SSK | unresolved, ssk-only 995 596 | unresolved, ssk-only 1 136 456, 11 skips |

Wall seconds per run (1×5 took 0.000 s everywhere):

| case | A | B | C | D | E | F | G | H | I | J |
|---|---|---|---|---|---|---|---|---|---|---|
| 1×6 PSK | 0.760 | 0.769 | 0.775 | 0.798 | 0.784 | 0.490 | 0.501 | 0.487 | 0.847 | 0.852 |
| 1×6 SSK | 2.875 | 2.813 | 2.849 | 2.922 | 2.898 | 3.991 | 3.954 | 3.973 | 3.145 | 3.138 |
| 1×7 PSK | 2.850 | 2.900 | 2.891 | 2.921 | 2.908 | 3.776 | 3.779 | 3.772 | 3.172 | 3.176 |
| 1×7 SSK | 2.957 | 2.932 | 2.906 | 2.996 | 3.066 | 4.025 | 4.001 | 4.014 | 3.219 | 3.230 |
| 2×3 PSK | 2.721 | 2.721 | 2.738 | 2.785 | 2.780 | 3.816 | 3.829 | 3.819 | 3.078 | 3.015 |
| 2×3 SSK | 2.840 | 2.762 | 2.766 | 2.833 | 2.836 | 4.732 | 4.736 | 4.739 | 3.076 | 3.090 |

**The empty 2×3 root under PSK is not a result.** With mirrored moves it
resolved to 0 at 93 137 907 nodes, where the plain search is unresolved at 10⁸
here and at 10⁹ in `notebook/2026-09-12-ssk-docstring-and-2x3.md`. That is a
bench measurement under the unlicensed `board-symmetry` divergence, not a value
for the ledger. To cite it, the main session would rerun
`superko solve --board 2x3 --rule psk --suicide forbid --root .../... --budget
100000000 --symmetry on` and promote the body.

### What the features cost where they save little

**Mirrored moves cost time per node.** Wall seconds per node relative to a run
without them, from these files:

| case | F against B (1 thread, earlier build) | G against D | H against I (same build, 14 threads) |
|---|---|---|---|
| 1×6 PSK | +33.7 % | +31.7 % | +20.6 % |
| 1×6 SSK | +41.9 % | +35.3 % | +26.3 % |
| 1×7 PSK | +30.2 % | +29.4 % | +18.9 % |
| 1×7 SSK | +37.3 % | +33.5 % | +24.7 % |
| 2×3 PSK | +50.6 % | +47.6 % | +33.2 % |
| 2×3 SSK | +71.3 % | +67.2 % | +54.1 % |
| 2×3 sweep | +59.1 % | +56.5 % | — |

Against `J`, from the same build and also 14 threads, `H`'s sweep costs 43.9 %
more per node (0.878 s against 0.610 s). Against runs from an earlier build the
overhead reads 29 to 71 percent; against the same build, 19 to 54 percent. On
the sweep, mirrored moves alone took 62.6 s against the plain 39.35 s at nearly
equal nodes. With canonical roots on, adding them took 0.878 s against 0.564 to
0.610 s at 14 threads.

**The off path got slower after the symmetry commits.** Same flags,
`2bf8ef4`/`09dcc60` builds against `51d9914`, 14 threads, equal nodes:

| case | plain, C to I | roots, E to J |
|---|---|---|
| 1×6 PSK | 0.775 to 0.847 s, +9.3 % | 0.784 to 0.852 s, +8.7 % |
| 1×6 SSK | 2.849 to 3.145 s, +10.4 % | 2.898 to 3.138 s, +8.3 % |
| 1×7 PSK | 2.891 to 3.172 s, +9.7 % | 2.908 to 3.176 s, +9.2 % |
| 1×7 SSK | 2.906 to 3.219 s, +10.8 % | 3.066 to 3.230 s, +5.3 % |
| 2×3 PSK | 2.738 to 3.078 s, +12.4 % | 2.780 to 3.015 s, +8.5 % |
| 2×3 SSK | 2.766 to 3.076 s, +11.2 % | 2.836 to 3.090 s, +9.0 % |
| 2×3 sweep | 3.977 to 4.166 s, +4.8 % | 0.564 to 0.610 s, +8.2 % |

The plain and roots-only paths ran roughly 5 to 12 percent slower in release.
These are single runs, and the cause is not isolated: it may be the mirrored
move bookkeeping's checks on the off path, a layout change, or noise.

**What the bench does and does not measure.** Plan §5.5 asks for the sweep at
10⁶ nodes **per root**. The bench spends 10⁶ **per search**, two searches per
root. The suite has no 1×7 case that resolves. No run was repeated.

## Defaults, and the decision left to the maintainer

At `51d9914`:

- **`--threads`: default 1.** Off by default per correction 7, as
  `count-games`. It never changes a body, so the default only sets wall time.
- **`--symmetry`: default off**, for `separate` and for `solve`. Per correction
  7, turning it on would change the `divergences=` line, and add body lines,
  in every sweep and solve, so no existing witness would verify.
- **`bench`: `--symmetry` defaults to off** and `--threads` to 1.

**The implementers' recommendation, weighed against plan §5.5.** §5.5 keeps a
feature on by default only if it lowers nodes or wall time on the suite
without raising either by more than a small factor on any case.

- **Canonical roots pass that test on this suite.** Seven times fewer nodes on
  the 2×3 sweep (216 of 1 458 roots searched) at about the same time per node
  (D against B: 2.83 × 10⁻⁸ s against 2.79 × 10⁻⁸ s). The single-root cases
  are untouched, apart from the off-path slowdown above that all builds from
  `51d9914` share.
- **Mirrored moves do not pass it.** They lower nodes where they skip: the
  1×6 PSK nodes halve, and the empty 2×3 PSK root resolves within 10⁸. Where
  they skip nothing they only cost: 1×6 SSK and 1×7 under both rules have 0
  skips and take 19 to 42 percent more time per node, a span over two
  comparisons: 19 percent at the low end is H against I, 42 percent at the
  high end is F against B (2.81 s to 3.99 s on 1×6 SSK). Alone on the sweep they took 62.6 s against 39.35 s at nearly
  equal nodes (+59 % per node). With canonical roots the sweep saw 228 skips
  at 44 to 57 percent more time per node.
- So the recommendation is canonical roots in sweeps, and mirrored moves only
  for a single solve where skips are expected — an empty or nearly symmetric
  root — accepting up to about 40 percent more time per node when there are
  none. Whether a rise of 1.6 times on the moves-only sweep is a "small
  factor" is the maintainer's reading of §5.5. Defaults stay off per
  correction 7.

**The decision this leaves.**

- Whether to turn either feature on by default. That would re-record every
  witness body.
- Whether to add a roots-only `--symmetry` value to `separate`. From the CLI
  today a sweep cannot take canonical roots without mirrored moves.
- Whether to make the mirrored-move bookkeeping cheaper before the 2×3 rerun.
  The soundness reviewer's reductions: a per-code bitmask of the elements
  fixing the code, a flattened `Symmetries` code table, and the element count
  hoisted into the mirror state.

## Test times

- **Debug, whole workspace.** `cargo test --workspace` passed in 165 s wall at
  `51d9914` (main-session gate, `data/gate.log`, `debug-test-seconds=165`),
  against about 154 s before the session: 11 s of growth, inside correction
  8's 30 s. That is one run, and `superko-solve`'s debug time varies. Summing
  the `finished in` times of its test binaries: 138.0 s before mirrored moves
  (`data/mirror/baseline-debug-solve.txt`); 178.1, 173.2, 174.0 and 151.4 s
  over four runs during mirrored moves
  (`data/mirror/debug-superko-solve{,-2,-3,-4}.txt`); and 150.7 s in the gate.
  Most of the spread is `tests/agreement.rs`, 100.9 to 122.6 s across those
  runs.
- **Release, the ignored tests one by one.** Each runs as
  `cargo test --release -p <crate> --test <file> -- --ignored <name>`.
  Logged runs, by the mirrored-moves reviewer (`data/review-mirror/`):
  - `superko-solve --test agreement`
    `the_engines_agree_with_mirrored_moves_on_three_points`: 10.00 s.
  - `superko-solve --test mirrored`
    `mirrored_moves_agree_on_turned_lines_and_boards_of_five_points`: 27.08 s.
  - `superko-solve --test symmetry`
    `a_budgeted_symmetric_sweep_with_mirrored_moves_agrees_with_the_plain_sweep`:
    1.29 s.

  Not logged one by one; these per-file times come from session reports, not
  from a file under `data/`:
  - `superko-solve --test symmetry -- --include-ignored`, which runs
    `values_are_invariant_on_boards_of_five_points`,
    `verdicts_are_transported_on_boards_of_five_points`,
    `a_budgeted_symmetric_sweep_agrees_with_the_plain_sweep` and
    `a_symmetric_sweep_agrees_with_the_naive_engine_on_three_points` with the
    rest: 10 passed in 10.6 s (main session, at `09dcc60`, before the
    mirrored-moves variant was added). `data/symmetry/test-release-solve.log`
    holds an earlier run of that file: 8 passed in 11.11 s.
  - `superko-rules --test symmetry -- --include-ignored`, which runs
    `the_table_commutes_with_every_map_on_3x3_and_3x4`: 7 passed in 40.6 s
    (main session, at `09dcc60`).
  - `superko-solve -- --include-ignored`: 60 s at the end (implementer's
    report).
- **The rest of the gate at `51d9914`**, all passing: `cargo fmt --all
  --check`; `cargo build --workspace` with no warnings; `cargo clippy
  --workspace --all-targets`; `tools/check-ledger.sh`, `check-docs.sh`,
  `check-mirror.sh` (7 divergence slugs) and `check-oracle.sh`; and
  `tools/verify-results.sh` on the eleven `separate` results of at most five
  points.
- **The 2×3 body at 14 threads.** `target/release/superko separate --board 2x3
  --suicide forbid --budget 10000000 --threads 14` printed a body
  byte-identical to `results/separate-2x3-forbid-budget-1e7.txt` in 40 s wall
  (39.963 s on standard error, 13 484 773 896 nodes). The recorded one-thread
  run took 363 s.

## Reviewer refusals, and how each was resolved

**`0490183`, bench. Round 1 refused, with two blocking findings:**

1. A cap test compared the suite against the same constants the suite was built
   from, so it could not fail.
2. No test exercised `run_case`.

The fixer pinned the suite against literal caps and added `run_case` tests, and
checked both by mutation. Round 2 approved.

**`2bf8ef4`, parallel sweeps.** Approved in round 1.

**`09dcc60`, canonical roots. Round 1 refused, with six blocking findings:**

- In four places — `divergence.rs`, `separate.rs`, `lib.rs` and
  `results/README.md` — the value-invariance coverage was stated without its
  exception. 1×5 and 5×1 with suicide removing its own stones are compared only
  at resolved roots.
- The fold's three ssk-only counts were untested under symmetry.
- No test showed that the representative is its orbit's least root.
- The default-body guard did not cover the `<prefix>-transported=` witness
  line.

Round 2 refused on one finding: no test that a transported root copies its
representative's `ssk_only`, `nodes` and `max_depth`. The fixer added each test
and qualified each statement, and round 3 approved.

**`51d9914`, mirrored moves.** Approved in round 1. The reviewers flagged
documentation defects as nonblocking, and the records step fixed them in doc
comments and docs only:

- `search.rs`'s sentence about the subgroup `H` was garbled.
- The same doc claimed "the tests run with" the self-check on; it now names the
  searches that do.
- A `tests/threads.rs` doc called the 1×5 sweep "the empty 1×5 board".
- `results/README.md` did not say that two `# produced-with:` forms exist, and
  its long `separate` table row is now prose below the table.
- `lib.rs` said "at most four points and of 2×2".
- "Every komi floor" is now the tested range, `-(m·n) - 1` to `m·n + 1`, in
  `lib.rs` and the three test files.

## What is not established

- **Nothing here is a claim about Go**, and no number here enters the ledger
  above `computed`. No ledger row changed.
- **The symmetry facts are unlicensed divergences, not theorems.** Correction
  1's and 2's commutation of the transition table is `computed` exhaustively
  on boards of at most six points, and on 3×3 and 3×4 in release. Value
  invariance under the group and the swap, the verdict transport, and the
  equality of values and verdicts with mirrored moves on and off are
  `computed` on boards of at most five points only. On 1×5 and 5×1 with suicide
  removing its own stones they hold only at the roots resolved within the
  budgets `tests/symmetry.rs` and `tests/mirrored.rs` name. No Lean lemma states
  any of it.
- **Verdicts are tested at komi floors `-(m·n) - 1` to `m·n + 1` only.** At a
  floor outside that range every leaf's winner test gives what it gives at the
  nearer end floor, since the area difference lies between `-(m·n)` and `m·n`
  (read from `winner_at` and `RuleTable::area`, not tested), but the searches
  are not tested there.
- **The `board-symmetry` consequence sentence overstates the floor range.**
  `Divergence::BoardSymmetry.consequence()` in `divergence.rs` says the
  equality with mirrored moves is `computed` "at every root and komi floor";
  only the range above is tested. `docs/trusted-base.md` now gives the range.
  The string was left alone here because it is a value, not a doc comment.
  `superko-rules`'s crate docs say a witness header prints it, though no CLI
  code calls `consequence()` today, and no committed results file was
  produced with symmetry on; changing it is the maintainer's call.
- **Thread invariance** of the sweep and its body is `computed` on the cases
  `tests/threads.rs` names (feature 2 above) and on the 1×5 and budgeted 2×3
  bodies matched at 14 threads, and is not proved.
- **The known gaps the reviewers recorded:**
  - No test pins a state whose position a symmetry fixes but whose archive it
    does not. The fixing test requires `unmatched == 0`, so a wrong answer
    there shows only through a changed value; 1×2 under suicide removing its
    own stones, PSK, has such a state.
  - No test shows that the witness verdicts in `Sweep::lines` use mirrored
    moves.
  - On 2×2 the self-check runs on the value differential and on one verdict
    search per rule (the empty root, Black to move, at the floor equal to the
    value, in `mirrored_moves_skip_plays_on_the_empty_1x4_and_2x2_boards`),
    not on the 2×2 verdict differential.
  - No board under the solver tests has the four-element group of a non-square
    rectangle. 2×3 is first used by the bench, and the table commutation is
    tested exhaustively on 2×3 and 3×2.
  - No test pins the skip mask under a rotation-only subgroup.
  - The `<prefix>-transported=` witness line can only print `false`, because a
    minimal root is always its orbit's least root. That is a reviewer's reading
    of the code, not a test.
  - With `--symmetry on`, `divergences=` names `board-symmetry` even on 1×1 and
    under floors that skip every root. That over-reports, which is the
    conservative direction.
  - The bench spends 10⁶ nodes per search, where plan §5.5 said per root.
- **A rule this session broke.** `data/review-clippy` and
  `data/review-clippy-bench-r1` are extra Cargo target directories that
  reviewers created under `data/`, which this session's agent rules forbid.
- **Unmeasured.** The bench's noise, the cause of the 5 to 12 percent off-path
  slowdown, and whether the empty 2×3 PSK value survives a promoted
  `superko solve` run.
- **The 2×3 sweep was not rerun at a larger budget.** C-53 and experiment 005
  stand as they were: 792 of 1 458 roots resolved at 10⁷ nodes per search,
  none separating.

## Where the next session picks up

1. **The 2×3 rerun, which the maintainer launches.** The candidate is `superko
   separate --board 2x3 --suicide forbid --threads 14 --symmetry on` at a
   budget the maintainer chooses. Given the per-node overhead above, the
   maintainer decides first whether a roots-only CLI value or cheaper
   mirrored-move bookkeeping should come before it. A body produced with
   `--symmetry on` names both divergences and differs from the recorded
   budgeted body.
2. **The empty 2×3 PSK value under mirrored moves**, if it is to be cited:
   `superko solve --board 2x3 --rule psk --suicide forbid` on the empty root,
   Black to move, `--budget 100000000 --symmetry on`, with the body promoted
   under a witness header.
3. **The `board-symmetry` consequence sentence**, narrowed to the tested floor
   range if the maintainer agrees (above). Beside it, three pre-existing doc
   comments in `superko-rules` are stale: `divergence.rs`'s module doc says
   every witness header prints each consequence sentence, and `lib.rs`'s crate
   docs say the same, though no CLI code calls `consequence()`; and
   `divergence.rs`'s module doc still calls the symmetry divergences facts about
   the rules a search relies on, the wording `docs/trusted-base.md` moved away
   from. Either the docs are corrected or the printing is implemented.
4. **A pre-existing rustdoc warning.** `cargo doc -p superko-solve --no-deps`
   warns that `separate.rs`'s public module doc links to the private `fold`
   (from `09dcc60`).
5. **Plan §2.3 to §2.6**: sound static bounds, ordering, the transposition
   table measured before it is believed, and root splitting for a single solve.
   §2.6's `--threads` for `solve` is still refused. Plan §6's 1×9 deliverables
   go with them: item 3, the C-11 null-window search at floor zero on 1×9, with
   §2.4, and item 5, a second 1×9 attempt, with §2.6. Correction 9 kept 1×9 out
   of this session only.
6. **Scratch.** `data/review-clippy` and `data/review-clippy-bench-r1` are
   reviewer clippy target directories, 13 MB each and gitignored, made against
   the rule above, and can be deleted.
