# 003 — Can complexitylib's window calculus witness the archive bound?

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`); the probes this pre-registration rests on were run by Claude Opus 5 (`claude-opus-5`) agents on 2026-09-10 and 2026-09-11
**Status:** planned
**Opened:** 2026-09-11
**Claims touched:** C-3, C-20, C-32

## Question

Given the archive decider proved correct against `Superko.BlackWins` (C-29)
and the string encoding with its length bounds (C-31), both over `Defs.lean`
and Mathlib alone, does `SamuelSchlesinger/complexitylib`'s generic
space-membership lemma `Complexity.TM.decidesInSpace_of_keepsWindow` together
with `enc` suffice to state and prove

```
goLang ∈ DSPACE (fun n => 2 ^ (c * n))
```

in a side workspace at this project's own Mathlib pin, using only the
library's publicly exported declarations?

This is the question experiment 002 asked, with the parts of 002 that were
wrong removed: the premise (see 002's Verdict) and the measures (a time box and
a line threshold, which the maintainer ruled out on 2026-09-10).

## Why this is not load-bearing

Nothing in the trusted base changes on any outcome. The grounding decision
recorded in [`../../docs/plans/complexity-grounding.md`](../../docs/plans/complexity-grounding.md)
keeps every complexity library out of `lean/`; this probe lives under `data/`
and, if it succeeds, produces a *second* witness for C-3 in a disclosed side
base, never a dependency of a headline theorem. The trusted-base cost of the
library's definitions (`Cfg.WithinDecisionSpace`, `TM.DecidesInSpace`,
`DSPACE`, `BigO`) is the reason it stays a side witness whatever happens here.

## Hypothesis

**No, not with public declarations alone.** The landing gear — the window
calculus in `Combinators/Internal/Window.lean` — sits in a module the
library's own lint declares non-public, and at least one of the four named
obstacles below fires.

Reasons it might be wrong, recorded because they are the interesting case
(all `computed`, 2026-09-11, in a scratch copy of complexitylib `dev` 6c248df
re-pinned to Mathlib `f5e9087`):

- `Complexity.TM.decidesInSpace_of_keepsWindow` and
  `Complexity.TM.loopTM_keepsWindow_indexed` are generic in the space bound,
  carry no polynomial hypothesis, and report exactly
  `[propext, Classical.choice, Quot.sound]` at this project's Mathlib pin.
- A four-line specialization to an exponential bound elaborates with the same
  axioms.
- A file importing both `SuperkoComplexity.Defs` and
  `Complexitylib.Classes.Space` in one workspace elaborates a `goLang`
  language and the membership statement, axiom-clean.
- `Combinators/Internal/Window.lean` carries no `Unreviewed` marker.

## Falsification

**Fixed before the probe runs. No threshold, no time box.** Each outcome is a
result, and every outcome is informative:

| Outcome | Consequence |
|---|---|
| The instantiation goes through with public declarations only | A second, redundant witness for C-3 in a disclosed side base. The cited-textbook statement stays the headline. C-20 is rewritten again with this fact in its witness. The boundary section of `docs/trusted-base.md` gains a sentence naming the side base. |
| It requires `Internal` modules | Recorded as an API-stability finding. The trusted base cannot take declarations the library itself declares non-public. C-20's witness records which. |
| It requires the direction SPACE(textbook) ⊆ DSPACE(library) | The library's head-position space convention can witness membership only in the charged-more direction; it could carry C-3 and must not carry C-2. Record and stop. |
| One of the four named obstacles fires: `loopTM_keepsWindow_indexed`'s hypotheses cannot be instantiated for an exponential bitmap archive; `DecidesInSpace`'s unconditional-halting conjunct cannot be discharged from a tape-level fuel counter; malformed-input rejection cannot be kept inside the window; the three-symbol write alphabet defeats the head-position accounting | Name the missing lemma and carry it as an explicit named hypothesis in the probe's statement rather than working around it — the pattern `LeanMillenniumPrizeProblems` uses for Mathlib's missing polynomial-time composition. The named hypothesis is the finding. |

If the probe stops on an obstacle not in this table, that obstacle is the
result and it goes here.

**A null result is a result.** Record it and leave it in place.

## Method

1. Use the existing scratch copy of complexitylib `dev` 6c248df with its
   Mathlib requirement rewritten to `f5e908760f367cc0ad66f64fb3b3a689965bc7a1`
   and its inert cslib requirement removed
   (`data/grounding/scratch/complexitylib-audit/complexitylib-f5e9087/`, built
   2026-09-10: `Classes.NP.Reduction`, `Classes.Containments`, `Encoding`,
   `Containments.PSPACESubsetEXP` and the window calculus compile there). If
   the copy is gone, recreate it as described in
   `data/grounding/scratch/complexitylib-audit/commands-record.txt`; **not**
   into `lean/`.
2. Put both workspaces on `LEAN_PATH` and re-elaborate the cross-import
   bridge file from the survey (`goLang` from this project's `BlackWins`,
   `S1` over the library's `DSPACE`) to confirm the starting point still holds
   at the current revisions.
3. State the target: `Superko.Enc.goLang ∈ DSPACE (fun n => 2 ^ (c * n))`
   for some `c`, over `Superko.Enc.enc`, with `goLang` as promoted in
   [`../../lean/SuperkoComplexity/Encoding.lean`](../../lean/SuperkoComplexity/Encoding.lean).
4. Attempt the proof through `decidesInSpace_of_keepsWindow` and
   `loopTM_keepsWindow_indexed`, using the decider of
   [`../../lean/SuperkoComplexity/Decide.lean`](../../lean/SuperkoComplexity/Decide.lean)
   as the step function's specification. Import only modules outside
   `Internal/` first; record the first point at which an `Internal` import
   becomes necessary.
5. On any obstacle, state the missing lemma as a named hypothesis of the
   probe's theorem and stop; do not prove around it.
6. Record every `#print axioms` output and every module imported, and the
   revision of both workspaces, whatever the outcome.

Every artifact stays under `./data/`, which is gitignored. Only this file is
committed.

## Result

Not yet run.

## Verdict

Pending.
