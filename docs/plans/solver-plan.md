# Reach for `superko-solve`

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`) wrote this plan from a
reading of the crate in a review session; the implementation is delegated to
a Claude Opus 5 (`claude-opus-5`) session running the workflow in §6.

The plan for taking `superko-solve` from boards of five points to the boards
the two open computations need: the 2×3 separation sweep, which hit its node
cap on 2026-09-12, and the 1×9 empty-board value of C-11. The solver stays a
minimax **value** solver; the boolean verdict search stays alongside it
unchanged.

**Status of everything here: design.** Nothing below is a claim about Go.
The crate is not trusted, and no number it produces enters the ledger above
`computed`. Every change below is a performance change: none may alter a
value or a verdict, and the tests of §5 are what hold that.

## 1. Where the solver stands

[`crates/superko-solve/src/search.rs`](../../crates/superko-solve/src/search.rs)
is fail-soft alpha-beta over `RuleTable`, with make and unmake on the dense
`Archive`, a full-range window, one-ply area ordering with the pass first,
a node budget, and no memory of any kind. The naive engine in `naive.rs` is
the arbiter on boards of at most three points; on larger boards the
order-reversal test is the only evidence the cutoffs preserve the value.

Measured: the empty 1×5 board under PSK costs 2 115 nodes, the empty 1×6
board 27 925 122. 1×7 and 1×8 do not resolve at 4 × 10⁷ nodes. The 2×3
sweep, 1 458 roots, did not resolve every root at its budget. The 2×2 sweep
takes about ten seconds on one core in release.

Constraints the crate inherits and this plan keeps:

- **No `HashMap`, no `HashSet`** anywhere in the workspace (`clippy.toml`):
  per-process random iteration order breaks determinism. Any table is a
  hand-written open-addressing array over `Vec`, with a hash this crate
  computes itself.
- **No dependencies.** Random numbers for hashing come from a fixed-seed
  `splitmix64` written in the crate.
- **The archive is never a hash.** `archive.rs` says why, and that stays: the
  archive is exact. The transposition table of §2.5 is a *cache* beside it,
  with its own collision policy stated in its own module doc.
- **Every unlicensed shortcut is a divergence.** A search that relies on a
  fact about the rules that `Defs.lean` does not state — symmetry invariance,
  the pass-alive bound — registers it in `superko_rules::divergence` so the
  `divergences=` line of every result names it, `unlicensed` until a Lean
  lemma licenses it.
- **`MAX_TABLE_POINTS` is 12.** 1×9, 2×4, 3×3 and 3×4 fit; 5×5 does not.
  Raising it is out of scope.
- **The machine:** 14 cores, 36 GiB. Nothing here needs more than a few
  hundred MB.

## 2. The six changes, in the order to make them

Each is a feature behind a `Solver` builder method, off by default until §5
says it earns its place. The order puts the certain wins first and the
speculative one last, so that a session that stops early has still moved the
sweeps.

### 2.1 Parallel sweeps

The sweep in `separate.rs` solves every root of a board independently. Run
the roots over a fixed thread count, as `count-games --threads N` already
does in `superko-graph`, each thread owning its own `Solver` (the archive is
2·3^(m·n) bits: kilobytes) over a shared `&RuleTable`. Use
`std::thread::scope`. Collect per-root results into a `Vec` indexed by root
so that the output is byte-identical to the single-threaded run: node counts
per root are properties of the search, not of the thread.

Add `--threads N` to `separate` and to `solve`'s root split of §2.6. The
result header records the thread count; the body must not change with it,
and a test holds a 1×3 sweep equal across thread counts.

This is the change that reopens 2×3: the same wall time buys roughly ten
times the budget per root.

### 2.2 Symmetry

Two invariances, neither stated in `Defs.lean`, both registered as
divergences (`board-symmetry:unlicensed`, `color-swap:unlicensed`), both
checked by test against the plain solver on every root of every board of at
most five points.

**Canonical roots in a sweep.** A board's symmetry group (the dihedral group
of the rectangle: two elements for a line or a non-square rectangle, eight
for a square) acts on position codes, and the color swap acts on codes and
negates the value with the color to move swapped. Precompute the orbit map
once per board as a `Vec<PosCode>` of canonical representatives. A sweep
solves canonical roots only and fills the rest by transport: value negated
under swap, unchanged under reflection, verdicts at floor k transported to
the mirror image and, under swap, to floor −k−1 with the colors exchanged.
Every root still appears in the output, marked `transported`, and the
`ssk-only` count of a transported root is copied from its representative.
The gain is a factor of two to eight in roots.

**Mirrored moves at a symmetric state.** A state is symmetric under a
reflection σ when the board is σ-fixed and every archived key is σ-fixed or
archived together with its image. Track that incrementally: keep, per
reflection, a count of archived keys whose image is not archived; the state
is symmetric when the count is zero and the board is σ-fixed. At a symmetric
state the σ-images of a move lead to σ-images of subtrees, so search one
move of each pair and skip the other. On the empty 1×n board this halves the
root's children and recurs down the line while play stays symmetric. The
saving is modest past the first few plies; the test is the same as above.

### 2.3 Sound static bounds

A bound the search may cut on must hold under every continuation of the
rules, superko included. The bound here is Benson's pass-alive test, which
is a theorem about the position alone under the no-suicide convention: a
chain that is pass-alive cannot be captured by any sequence of opponent
moves while its owner passes. Under `Suicide::Forbid` only; under
`RemoveOwn` the feature is refused, not adapted.

Build a per-code table alongside `RuleTable`: for each position code and
color, the number of points that are **secure** for that color, meaning
stones of pass-alive chains plus single-point eyes of those chains at which
the opponent's play is refused as suicide. Compute it by a reference
implementation of Benson over the decoded position, once per code, the way
`RuleTable::build` calls the transliteration: slow and readable, and never
run in the inner loop. Register it as `pass-alive-bound:unlicensed`.

At a node with `S_B` secure Black points and `S_W` secure White points on a
board of `n` points, under the strategy "the owner passes forever" the
owner's area at the end is at least its secure count and the opponent's at
most the rest, so

    value ≥ 2·S_B − n      and      value ≤ n − 2·S_W .

In the value search, return the lower bound when it reaches `beta` and the
upper bound when it is at or below `alpha`, and otherwise tighten the window
to them. In the verdict search at floor k, Black wins when `2·S_B − n > k`
and White wins when `n − 2·S_W ≤ k`. The test compares values and verdicts
with the feature on and off on every root of every board of at most five
points, and separately holds the secure count against a hand-written list on
a dozen positions of 1×5 and 2×2, including a two-point eye, which is *not*
secure, and a chain with one eye, which is not pass-alive.

Where this pays is the tail of the game, where the search spends its nodes;
how much it pays is the measurement of §5.

### 2.4 Ordering

Three cheap additions to `ordered_moves`, kept only if §5 shows they cut
nodes:

- **Killer moves.** Per depth, the last two moves that produced a cutoff;
  visit them first among the plays. The pass keeps its place.
- **History table.** Per point and color, a counter raised at each cutoff
  the move produced; break ties in the area sort by it instead of by
  `all_moves` order. Reset per root.
- **Window bisection.** An alternative driver to the full-range search: a
  null-window search at a guessed value answers "above or below" and costs
  far less than a full window; bisecting the score range takes at most
  ⌈log₂(2n+3)⌉ of them, five on 1×9. Without a transposition table each
  bisection step re-searches from scratch, so whether five null windows beat
  one full window is an empirical question, and the answer likely differs
  between the empty board and the near-full roots of a sweep. Implement it
  as `Solver::solve_root_bisect`, and let `solve` take `--driver full|bisect`.
  For C-11 the first null window sits at zero, and that one search decides
  between the two published candidates before the value is known.

Every ordering change must leave `MoveOrder::Static` untouched: the
order-reversal test needs a genuinely different order to compare against.

### 2.5 A transposition table, measured before it is believed

The crate's own doc gives the reason to doubt this one, and it is right to:
two paths to the same board carry the same archive only when they visited
the same *set* of boards. Commuting moves do not transpose, because the
intermediate boards differ. Under PSK a game is a simple path, so a
transposition is two simple paths through the same vertex set between the
same endpoints, which captures make possible and which may be rare. So the
table is built to be measured, and the measurement decides whether it stays
on.

Design: an open-addressing array of fixed power-of-two size, chosen by a
`--tt-bits B` flag, entries of `(check: u128, code: PosCode, to_move,
passes, value: i8, kind: bound-or-exact, age)`. The key is the archive
digest: a 128-bit Zobrist value, the XOR over archived keys of a per-key
constant drawn once per board from `splitmix64` at a fixed seed, maintained
incrementally in `make` and `unmake`. The bucket index is folded from the
digest and the position fields; the stored `check` is the full digest, and
`code`, `to_move` and `passes` are stored in the clear and compared exactly.
A false hit therefore needs two distinct archives with the same 128-bit
digest at the same position, which at 10¹² lookups has probability of order
2⁻⁸⁸. Say exactly that in the module doc, next to the archive's refusal of
64-bit keys, and register `tt-digest:unlicensed`. Replacement: prefer the
entry with the larger subtree, else the older. Store fail-soft bounds as
bounds and use them only within the window they hold for.

Measure the hit rate and the node count on the 1×5 and 1×6 empty boards and
on a 2×3 sweep at a fixed budget. If hits prune less than the lookups cost,
the feature stays off by default and the notebook records the null result
with the numbers, which is a result about the shape of superko game trees
worth having.

### 2.6 Root splitting for a single solve

For one hard root, split its legal children across threads, each thread
solving one child with a full window in its own `Solver`, then combine at
the root. This forgoes alpha-beta's cross-child cutoffs at the root only, so
the node count rises while the wall time falls, and it is the only way
`solve` uses more than one core. For the C-11 board it is the difference
between one core for a day and fourteen cores for two hours, if the search
is within reach at all. Node counts stay deterministic per child; the
result records both the total and the per-child counts.

## 3. What changes in the CLI and the results

- `separate` and `solve` gain `--threads N`, `--symmetry on|off`,
  `--bounds on|off`, `--ordering static|heuristic|killer`, `--driver
  full|bisect`, `--tt-bits B`. Defaults are whatever §5 settles on; a
  headline number in `results/` records every flag in its header.
- A new `bench` subcommand runs the fixed suite of §5 and prints one line
  per case: nodes, wall time, and whichever feature counters apply (TT hit
  rate, symmetric-state skips, bound cutoffs). Its output goes under
  `data/bench/`, gitignored; the notebook quotes it.
- `results/` files whose search used a transported root, a bound, or the
  table say so in the `divergences=` line, by the register.

## 4. What does not change

- `naive.rs`: the arbiter is untouched.
- The archive: exact, dense, no hash.
- `MoveOrder::Static` and the order-reversal test.
- The node budget's meaning: over budget means no value, and a sweep counts
  the root unresolved. No feature may report a bound as a value.
- The verdict recursion's branch structure, which mirrors `decideWins`.
- `superko-rules` beyond the divergence register and the pass-alive table,
  which lives in a new module and calls the reference, as `table.rs` does.

## 5. Tests and the acceptance gate

Before any feature is on by default, all of these pass in release:

1. **Differential, every root.** For every board of at most five points,
   every root, both colors to move, both rules, and both suicide conventions
   where the feature admits them: value and verdicts at every komi floor
   with the feature on equal those with it off. This is the test that
   catches a wrong cutoff, and it is the reason the plain solver stays
   reachable behind flags.
2. **Naive agreement** on boards of at most three points, as now, run with
   every feature on.
3. **Thread invariance.** A sweep's output is byte-identical at one, two and
   fourteen threads.
4. **Published values.** `published.rs` still reproduces the 1×n values for
   n ≤ 6; if 1×7 and 1×8 now resolve within budget, extend the test and
   update C-24's witness, marked `computed`, with the fixture's provenance
   caveat intact.
5. **The bench suite**, before and after each feature: the empty 1×5 and
   1×6 boards under both rules; the empty 2×3 board; a 2×3 sweep at a budget
   of 10⁶ nodes per root, reporting resolved roots; and, once it resolves,
   the empty 1×7 board. A feature is kept on by default only if it lowers
   nodes or wall time on the suite without raising either by more than a
   small factor on any case; otherwise it stays off and the notebook records
   why.

`cargo build --workspace` and `cargo test --workspace` clean, no new
warnings, `cargo clippy --workspace` clean under the workspace lints, and
`tools/check-ledger.sh` and `tools/check-docs.sh` passing.

## 6. The workflow for the implementing session

One implementer agent per feature, in the order of §2, in sequence, because
every feature touches `search.rs` and parallel edits would conflict.

After each feature, one read-only reviewer agent whose brief is §4 and §5:
does the diff change anything §4 protects, and do the tests of §5 actually
exercise the feature. The main session merges nothing the reviewer refused.

Rules for every agent, to be written into each brief verbatim:

- **Do not write your own solver, enumerator or game-tree search in any
  language to check a claim.** A verifier that wanted to check a value on
  2026-09-12 wrote a memoized Python search that took 78 GiB and crashed the
  machine. Check values with `cargo test` and `superko solve` under a node
  budget only.
- Every `superko` invocation carries `--budget`, at most 10⁸ nodes for a
  single solve and 10⁶ per root for a sweep, and at most one sweep runs at a
  time on the machine.
- Working output goes under `data/`, not `/tmp` and not the repository root.
- Commit per feature with a subject of the form `solve: <what the change
  establishes>`; a null result is a commit too, with its numbers.
- The notebook entry for the session carries the `**Author:**` and
  `**Models:**` lines, the bench table before and after each feature, and a
  plain list of which features are on by default at the end and which are
  not, with the reason.

Deliverables, in order of value if the session stops early:

1. §2.1 with the 2×3 sweep rerun at fourteen threads and a budget large
   enough to resolve every root, or the least unresolved root recorded if
   not. This alone answers whether C-53 extends to six points on 2×3.
2. §2.2 and §2.3, with the bench table.
3. §2.4, with the C-11 null-window search at floor zero attempted on 1×9 at
   the budget cap, and its outcome, resolved or not, recorded.
4. §2.5 with its hit-rate measurement, kept or refused.
5. §2.6, and a second attempt at 1×9 if the first did not resolve.

## 7. What this plan does not establish

A resolved 1×9 value is `computed` under `Defs.lean`'s rules and decides
between the two published candidates for this ruleset only; whether the
sources' disagreement is an error or a ruleset difference waits on the
primary source, still `sought`. A resolved 2×3 sweep extends C-53's null
result to one board of six points and bounds nothing at seven. No feature
here is a theorem: the symmetry and pass-alive facts are divergences until
Lean states them, and a result produced with the table on carries a stated,
unproved collision bound.
