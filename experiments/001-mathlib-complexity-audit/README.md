# 001 — What complexity theory does Lean have?

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)
**Status:** done
**Opened:** 2026-09-09
**Closed:** 2026-09-09
**Claims touched:** C-20, and the `infra-gap` marking on C-2, C-3, C-4, C-5, C-6, C-12, C-14, C-21, C-22

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

**The table above stands exactly as it was written before the experiment ran.
It has not been edited. The criticism of it is in the Verdict.**

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

Pinned revisions. Every count below is re-runnable against these:

| | |
|---|---|
| Mathlib | `f5e908760f367cc0ad66f64fb3b3a689965bc7a1`, 2026-09-09, Lean 4.34.0-rc2 |
| `PierreSenellart/descriptive-complexity` | `master` at `2bfbb33`, 2026-08-29 |
| `SamuelSchlesinger/complexitylib` | `dev` at `6c248df`, 2026-09-08 |
| `leanprover/cslib` | `0da3e0e` |

### Mathlib has no complexity layer

Verified by the author directly against the pinned checkout, not by report:

```
grep -rIl --include='*.lean' '\bPSPACE\b' .      # and EXPSPACE, EXPTIME, DSPACE,
                                                 # NSPACE, DTIME, NTIME, LOGSPACE,
                                                 # NPComplete, SpaceBounded, TimeBounded
```

Zero files for every one of those eleven names, across the whole checkout —
`Mathlib/`, `Archive/`, `Counterexamples/`, `MathlibTest/`, `Wanted/`.

Mathlib's single resource-bounded object is `Turing.TM2ComputableInPolyTime`,
in `Mathlib/Computability/TuringMachine/Computable.lean`. That file is 278
lines and contains **zero** `theorem` or `lemma` declarations
(`grep -cE '^\s*(theorem|lemma)\b'` returns 0). Nothing outside the file
mentions the name: `grep -rn 'TM2ComputableInPolyTime' .`, excluding the file
itself, returns nothing. Its composition lemma is an unproved `proof_wanted` —
added 2025-05-09 (`0d2adda759`, PR #7172), moved into the new `Wanted/` tree
2026-08-11 (`e76467f1ca`, PR #42284), still unproved at `f5e9087`.

Two further facts about the same checkout, both verified here:

- **No file imports both `Mathlib.CategoryTheory` and `Mathlib.Computability`.**
- **Combinatorial game theory is gone from Mathlib.** No file matches `pgame`;
  there is no `Mathlib/SetTheory/Game` directory. Deprecated 2025-08-17
  (`8939bd7674`, PR #28063), removed 2026-02-20 (`08657ed7f8`, PR #35550). It
  now lives in `vihdzp/combinatorial-games`.

What Mathlib does have is a mature *computability* layer — `Primrec`, `Partrec`,
Gödel-numbered codes with the step-indexed `evaln`, Rice and halting, many-one
and Turing degrees, three deterministic machine models, and a separate
automata layer with `MyhillNerode.lean`. None of it carries a resource bound.

### Two downstream Lean 4 libraries do have one

Measured by the author from fresh clones, not by report:

**`PierreSenellart/descriptive-complexity`** — Apache-2.0, 728 files, 313,785
lines of Lean, zero `sorry`, zero `axiom` declarations, zero `native_decide`.
Pinned to Lean **v4.33.0** and Mathlib `v4.33.0`. Defines `PSPACE`, `EXPTIME`
and `EXPSPACE` as `ComplexityClass` values, and carries `QSAT_PSPACE_complete`
and `wideCorridor_EXPSPACE_complete`. The string `superko` appears zero times.

**`SamuelSchlesinger/complexitylib`** — Apache-2.0, 1,687 files, 475,427 lines,
zero `sorry`, zero `axiom` declarations. Pinned to Lean **v4.34.0-rc2**, this
project's exact toolchain. Works in the string model, `Language := Set (List Bool)`.
Defines `DTIME`, `DSPACE`, `P`, `NP`, `PSPACE`, `NPSPACE`, `EXP`, and carries
Savitch, Cook–Levin, and `MapReducesPoly` with its `mem_P`/`mem_NP` closure
lemmas.

Its `native_decide` discipline is worth recording, because it matches this
project's own. `native_decide` occurs in five modules, all named `Validation`,
all outside the public import graph, and **only inside anonymous `example`s**,
which create no persistent declaration. CI runs `scripts/AxiomGuard.lean`,
which audits every declaration originating in a `Complexitylib` module —
definitions and opaque declarations as well as proofs — against
`allowedAxioms := [propext, Classical.choice, Quot.sound]`. That is the same
three axioms this project records in `results/axioms.txt`.

Against both: each is a few months old, dominated by a single author, and
substantially machine-written. 87 files in `complexitylib` carry an
`Unreviewed` marker in their module docstring. Neither is peer-reviewed.

**`leanprover/cslib`** has a merged space measure and, at `0da3e0e`, no
complexity class and no many-one reduction.

### What it would cost to use either

Two comparable developments have been measured, and the ratio is what
transfers rather than the absolute size. In Balbach's Isabelle/AFP Cook–Levin
(56,514 lines) and Gäher–Kunze's Coq Cook–Levin (about 16,500 project-specific
lines on a pre-existing framework), the definitional layer — machine model,
classes, reductions — is between 1% and 8% of the whole. Everything else is
machine construction, simulation, and resource-bounded composition. A library
supplying the definitional percent leaves the rest.

Over `complexitylib`, proving that a trivial language is in P currently runs to
about 1,100 lines apiece: `Palindromes` 1,147, `AnBn` 1,142, `ZeroPrefix` 1,066,
`Balanced` 1,040. Its own `docs/N0-MachineAuthoring.md` records that no
high-level machine-authoring frontend has been adopted. Whether those figures
transfer to a superko-Go decider — board encoding, liberty and capture,
history comparison against an archive, bounded game-tree search — is an
inference, not a measurement, and it is the inference this experiment's verdict
rests on. Experiment 002 is the pre-registered attempt to measure it.

### Not checked

- Neither library was built or kernel-checked here. `sorry`-freedom and axiom
  cleanliness rest on source inspection plus each project's own green CI.
- No definition in either library was read closely enough to audit.
  `SOPFPDefinable`, `ComplexityClass.ofMem`, `Cfg.WithinDecisionSpace` and
  `DataEncode` would each need reading before adoption.
- The survey of what else exists is not exhaustive. GitHub code search was
  unavailable; the substitute matched package metadata on the Lean package
  index plus named repositories, not Lean source across the ecosystem.
- Whether a published source proves the bridge between the descriptive
  characterization of EXPSPACE and the string-encoded class was not confirmed.
  `descriptive-complexity`'s own README states that agreement with the usual
  presentations "is classical (Fagin; Immerman–Vardi) and is not formalized
  here – except for RE".

### Method defect, recorded because it changed the first answer

The first pass produced 321 findings and verified 60. A cap in the harness
selected those 60 in order, and the order put every Mathlib-source finding
first — so no finding about a downstream library reached adversarial
verification. The synthesis written on that subset stated "Row 1 of the table
is cleanly excluded" and selected row 2, having never seen the evidence for
row 1. A completeness pass caught it, and the external half was re-established
from primary sources in a second run. The Verdict below is the second answer.
The first would have recorded the ecosystem half of C-20 as unexamined while
asserting it.

## Verdict

**Row 1's condition fires. Row 1's consequence does not follow. The
pre-registration joined the two, and the join was wrong.**

*The condition fires.* Usable resource-bounded complexity infrastructure exists
in maintained downstream Lean 4 libraries. The hypothesis was written as though
it did not. It already did — `complexitylib` since 2026-03-06,
`descriptive-complexity` since 2026-07-22 — and the hypothesis simply did not
look. Any sentence asserting that no such infrastructure exists anywhere in the
Lean ecosystem is false as of 2026-09-09 and should not be written again.

*The consequence does not follow.* Not one of the nine `infra-gap` rows changes
its marking, and not one changes its status. Row 1 predicted that hardness and
membership results return to formalizable scope. They do not, because none of
the nine was blocked on the absence of a class definition.

The nine divide in two. Four are `cited` results whose formalization means
re-proving them — C-2, C-4, C-5, C-22 — at the scale the Isabelle and Coq
figures above indicate. Four are proved by nobody: C-3 and C-6 are `folklore`,
C-12 is `open`, C-14 is `conjecture`. A library that lets an unproved claim be
*stated* yields a Lean statement with no proof, which a reader must audit on top
of the prose. C-21 is the one row where `infra-gap` overstates the blocker:
the Fraenkel–Scheinerman–Ullman matching argument is graph theory, and whether
Mathlib's matching development reaches it is an untested judgment, not a
measured obstacle.

Two library-specific obstacles are sharper than the cost.

**`descriptive-complexity`'s classes are not the classes the Go literature
means.** They are descriptive — SO(TC) and SO(PFP) over finite relational
structures. A machine-checked `SUPERKO_GO ∈ EXPSPACE` there would not be the
sentence C-3 states, and the bridge between the two is unformalized in that
library and, on the evidence gathered here, unverified as `cited` anywhere.
The project would trade a `folklore` prose claim for a `proved` Lean claim
about a different object resting on an unproved bridge. Adopting it would also
pin this project to Lean v4.33.0, a downgrade.

**`complexitylib` is in the right model and has no PSPACE-complete problem.**
Its QBF syntax and semantics are complete; `ROADMAP.md` still carries "Prove
TQBF is PSPACE-complete" as an unchecked box, and there is no
`MapReducesPoly.mem_PSPACE` beside the `mem_P` and `mem_NP` analogues.
`PSPACEHard` would be three lines with nothing to instantiate it.

**Where the pre-registration went wrong.** All three rows tested for a
definitional layer and inferred formalizability from its presence or absence.
Formalizability of these nine rows turns on two other things: whether a proof
exists at all, and what it costs to build a machine and bound its resources.
The question the table should have asked is whether a library exists over which
this project could prove one of its three statements without constructing a
Turing machine and without first proving a hard source problem complete. The
answer is no, and it would have been no in 2025. Row 3's cost premise fails in
the other direction: a minimal definitional layer is far cheaper than a month,
which does not help, because a cheap definition nothing can be proved over is
the outcome row 3 already anticipated and declined.

**The decision.** Proceed as row 2's consequence directs: formalize the
combinatorial spine, cite the complexity scaffolding, and say so in
[`../../docs/trusted-base.md`](../../docs/trusted-base.md) and in the paper.
The nine `infra-gap` markings stand. Their recorded reason does not — it named
Mathlib, and the real blocker is machine construction and the absence of a
string-model hard source — and the ledger now records the corrected one.

`complexitylib` is the named fallback if Phase 5's C-2 reduction is ever built:
`MapReducesPoly` is off the shelf, the Lean toolchain matches exactly, and the
axiom guard admits exactly the three axioms already in `results/axioms.txt`.
Its Mathlib revision does not match this project's, and Lake resolves one
Mathlib per workspace, so adoption is not free even then.

**What would overturn this**, both checkable:

1. A Lean library with a string-model PSPACE-complete problem and proven
   machine-composition combinators.
2. A demonstration that the archive decider for C-3 can be built and
   space-bounded over `complexitylib` in weeks rather than months.

The second is the one this project can settle itself, and
[`../002-complexitylib-spike/`](../002-complexitylib-spike/) pre-registers it.

**Re-check in six months and re-pin the revisions.** Both libraries move
weekly. C-20 is `computed` against four pinned revisions on 2026-09-09, and it
decays.
