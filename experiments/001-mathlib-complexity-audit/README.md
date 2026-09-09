# 001 — What complexity theory does Lean have?

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)
**Status:** planned
**Opened:** 2026-09-09
**Claims touched:** C-20, and the `infra-gap` marking on C-2, C-3, C-4, C-5, C-6, C-12, C-14

## Question

Can a statement of the form "SUPERKO-GO is in EXPTIME" — or any resource-bounded
complexity claim, or a polynomial-time many-one reduction — be stated and proved
in Lean 4 with Mathlib as it stands?

## Hypothesis

No. Mathlib has computability — Turing machines, partial recursive functions,
halting, formal languages — but appears to have no resource-bounded machine
classes, no PSPACE/EXPTIME/EXPSPACE, and no reduction infrastructure. Cook–Levin
has been formalized in Coq (Gäher–Kunze) rather than in Lean.

If that holds, the complexity-theoretic layer of this project cannot be machine
-checked, and the project's shape follows from it:

> Every novel claim lands on the formalizable side. Every claim on the
> unformalized side is one this project cites rather than proves.

## Falsification

Three outcomes, three different projects:

| Finding | Consequence |
|---|---|
| Usable infrastructure exists in Mathlib or a maintained downstream library | The `infra-gap` rows are wrong. Hardness and membership results come back into formalizable scope, and the whole strategy widens. Re-plan. |
| Nothing usable, and building it is more than ~1 month | Hypothesis confirmed. Proceed as planned: formalize the combinatorial spine, cite the complexity scaffolding, and say so explicitly in `docs/trusted-base.md` and in the paper. |
| Nothing usable, but a *minimal* definition sufficient for our statements is ~1 month | Genuinely open decision, and not one to take alone. A bespoke complexity definition that nobody else uses is weak evidence — a reader must audit it too, which enlarges the trusted base by exactly the thing they are least able to check. Probably still not worth it; record the reasoning either way. |

## Method

The toolchain is installed and Mathlib builds — see
[`../../lean/README.md`](../../lean/README.md). What remains:

1. Read `Mathlib/Computability/`. Inventory what is there: machine models,
   whether any carries a step or space bound, whether reductions appear at all.
2. Search Mathlib for `PSPACE`, `EXPTIME`, `polyTime`, `TimeBounded`, and the
   Mathlib archive and `mathlib4` issues for the same.
3. Survey outside Mathlib: any Lean 4 complexity library, and the state of the
   Coq/Isabelle equivalents, to know what a port would cost.
4. Write the finding here and update C-20.

## Result

Not yet run.

## Verdict

Pending. Every `infra-gap` row in the ledger is provisional until this closes.
