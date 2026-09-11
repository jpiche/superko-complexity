# 002 — What does one machine cost over `complexitylib`?

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)
**Status:** done — retired unrun on 2026-09-11; see Verdict
**Opened:** 2026-09-09
**Closed:** 2026-09-11
**Claims touched:** C-3, C-20

## Question

Experiment 001 declined row 1 of its own falsification table on a cost
inference rather than a measurement: that building a Turing machine and bounding
its resources over a downstream Lean library costs months, not weeks. Is that
true?

Concretely: how many lines, and how many days, does it take to construct one
decider over
[`SamuelSchlesinger/complexitylib`](https://github.com/SamuelSchlesinger/complexitylib)
and prove a space bound for it?

## Why this and not something larger

The load-bearing number in 001's verdict is an inference from four data points
in someone else's repository — `Palindromes` at 1,147 lines, `AnBn` at 1,142,
`ZeroPrefix` at 1,066, `Balanced` at 1,040, each proving a trivial language is
in P. Whether that transfers to a Go decider was never tested, and 001 says so.

This experiment tests it at the smallest scale that is still about Go. It is a
**spike**: throwaway, unmerged, and no part of it enters `lean/` or the ledger.
Nothing about the trusted base changes on its result. What changes is whether
C-3 is attempted over a library or written as prose.

The mathematical input is already done. C-26 (`proved`,
`formalized:C13_length_bound_explicit`) bounds a game from a root position at
4·3^(m·n) moves. That is the entire content of the archive argument: bound the
play length, and depth-first search carrying an archive of visited situations
runs in exponential space. What remains is machine engineering, and machine
engineering is exactly what this measures.

## Hypothesis

The 1,100-line figure understates a Go decider by a large factor, and the whole
spike exceeds its time box. Reasons to expect it:

- The four anchors decide languages whose membership test is a single pass with
  finite state. A superko decider needs board encoding, liberty and capture
  computation, situation comparison against an archive, and a bounded game-tree
  search with backtracking.
- `complexitylib`'s own `docs/N0-MachineAuthoring.md` records that no
  high-level machine-authoring frontend has been adopted, so every machine is
  built from transition tables by hand.

Reasons it might be wrong, recorded because they are the interesting case: the
anchors build machines with no reuse, and the marginal cost of a machine in a
library that has built dozens is not the cost of the first. Nobody has tried.

## Falsification

**Fixed before the spike runs.** Time box is **ten working days**. The board is
**1×n**, the rule is **PSK**, and the target is the pair:

1. a decider `M` over `complexitylib`'s `TM` for the language of 1×n positions
   from which Black wins;
2. a proof that `M` is space-bounded by `2 ^ p(n)` for some `Polynomial ℕ`,
   hence `SUPERKO_GO_1xn ∈ EXPSPACE` in that library's sense.

| Outcome | Consequence |
|---|---|
| Both land inside the time box, in **under 2,500 lines** | 001's cost inference is refuted at this scale. Re-open the question of attempting C-3 over `complexitylib` at general `m × n`, and record the measured rate. |
| Both land, but over 2,500 lines or over the time box | 001's verdict stands, now on a measurement rather than an inference. Record the actual figures; they are the anchor the next audit starts from. |
| (1) lands and (2) does not | The obstacle is the space bound, not the machine. Record precisely which lemma was missing; that is a contribution to `complexitylib` regardless. |
| Neither lands | 001's verdict stands a fortiori. Record where it stopped. |

2,500 is chosen as roughly twice the largest anchor, which is the point beyond
which "the marginal machine is cheaper than the first" stops being a defense.

A null result is a result. If this stops on day three against an immovable
obstacle, that obstacle is the finding and it goes here.

## Method

1. Vendor `complexitylib` at `6c248df` into a scratch Lake project under
   `./data/` — **not** into `lean/`. Note the Mathlib-revision conflict:
   `complexitylib` pins a Mathlib revision this project does not, and Lake
   resolves one Mathlib per workspace, so the spike gets its own workspace.
2. Confirm it builds on `leanprover/lean4:v4.34.0-rc2` before writing anything.
   If it does not build, stop and record that.
3. Read `Complexitylib/Classes/P/Internal/Palindromes.lean` end to end and
   record what fraction of its 1,147 lines is machine construction, correctness,
   and resource bound respectively. This is the baseline the estimate is
   compared against.
4. Encode 1×n positions as `List Bool` and state the language.
5. Build the decider and prove the space bound.
6. Record lines and elapsed days at each step, whatever the outcome.

Every artifact stays under `./data/`, which is gitignored. Only this file is
committed.

## Result

Not run. The grounding survey of 2026-09-10 and 2026-09-11
([`../../notebook/2026-09-11-complexity-grounding.md`](../../notebook/2026-09-11-complexity-grounding.md),
decision in [`../../docs/plans/complexity-grounding.md`](../../docs/plans/complexity-grounding.md))
established three things about this experiment before it could start, each
`computed` against the pinned revisions named there:

1. **The premise was false when written.** "The mathematical input is already
   done … What remains is machine engineering" assumed a decider existed to be
   compiled. At 2026-09-09 nothing about the game was computable:
   `Situation.after` is noncomputable (it goes through the classical
   `resolve`), `State.seen` is a `Set`, and there was no move enumeration, no
   decider and no bridge to `WinsFor`. Those were built during the survey,
   over `Defs.lean` and Mathlib alone, and are now C-29 and C-31. The
   machine-engineering question sits behind them, not in place of them.
2. **The measures were the ones the maintainer ruled out.** The falsification
   table keys on a ten-working-day box and a 2,500-line threshold. Both were
   scaffolding figures, and on 2026-09-10 the maintainer directed that such
   estimates be ignored. With them removed the table has no rows.
3. **The baseline was the wrong baseline.** The four ~1,100-line anchors
   hand-build transition tables. complexitylib's actual route to a space bound
   is its window calculus — `Complexity.TM.decidesInSpace_of_keepsWindow` and
   `loopTM_keepsWindow_indexed` in `Combinators/Internal/Window.lean` — which
   is generic in the bound, carries no `Unreviewed` marker, and reports only
   the three admitted axioms at this project's own Mathlib pin in a re-pinned
   scratch copy. Measuring against the anchors would have measured something
   no proof would use.

## Verdict

**Retired as written; null result recorded.** Rewriting a pre-registered
criterion after the evidence arrives is the failure the falsification field
exists to prevent, so this file is not edited into a different experiment. It
stays in place with its table intact, and the question it asked — can a
decider for SUPERKO-GO be space-bounded over complexitylib — is re-opened as
[`../003-complexitylib-window-probe/`](../003-complexitylib-window-probe/)
with a corrected premise, the library's real landing gear as the target, and
falsifiers that are fixed in advance and free of thresholds.

What 001's verdict rested on — a cost inference — is no longer what any
decision rests on. The grounding decision is made on trusted-base cost,
fidelity to the literature's classes, and concrete proof obligations, and
003 is not load-bearing for it.
