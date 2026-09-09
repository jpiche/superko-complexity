# 2026-09-09 — the complexity audit, and a search that capped itself

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)

Experiment 001 is closed. The finding is in
[`../experiments/001-mathlib-complexity-audit/`](../experiments/001-mathlib-complexity-audit/)
and C-20 carries it. What belongs here is how the audit nearly got the wrong
answer, and three things it turned up that nobody was looking for.

## The search capped itself, and the cap was invisible

The audit ran as a fan-out: eleven parallel readers, then an adversarial pass
that re-derives each finding independently, then a synthesis that picks a branch
of the pre-registered falsification table.

The verification stage took the first sixty of three hundred and twenty-one
findings. The order put every Mathlib-source finding ahead of every
ecosystem finding, so the sixty that were checked were all about Mathlib and
none was about the rest of the Lean world. The synthesis then wrote:

> Row 1 of the table is cleanly excluded.

Row 1 reads "usable infrastructure exists in Mathlib **or a maintained
downstream library**". Half its condition had not been tested. The synthesis
was not wrong about what it had seen; it was confident about what it had not.

The completeness pass caught it, and the external half was re-established from
primary sources in a second run, which reversed the branch. The lesson is
cheap to state and was expensive to find: **a cap that is not reported reads as
coverage.** The truncation was mine, in the harness, and nothing in the output
distinguished "we checked and found nothing" from "we did not check".

The same failure has a research-side shape and it is the one to watch for. A
negative finding inherits the scope of the search that produced it, and a
negative stated without its scope is indistinguishable from a stronger negative.
C-25 already carries its search date for this reason. C-20 now carries its four
pinned revisions and its date for the same reason.

## What the audit found that nobody expected

**Lean has complexity theory. It appeared this year, and not in Mathlib.**
The hypothesis was written as though nothing existed anywhere. Two maintained
libraries already did: `SamuelSchlesinger/complexitylib` since 2026-03-06,
`PierreSenellart/descriptive-complexity` since 2026-07-22. The first proves
Savitch and Cook–Levin in the string model on this project's exact toolchain.
The second defines EXPSPACE and proves complete problems for it. Both are
Apache-2.0 and sorry-free.

Neither is adopted, and the reason is not the one anyone would guess. It is not
that the libraries are bad. It is that a superko claim needs a Turing machine
for Go and a hard source problem in the right model, and no library supplies
either — so adopting one buys a vocabulary rather than a proof. That argument
is in `docs/trusted-base.md` now, because it is a statement about the trusted
base and not about a plan.

`complexitylib` deserves a note for a reason unrelated to complexity. Its
`native_decide` discipline is exactly this project's: the tactic appears only
inside anonymous `example`s, in modules outside the public import graph, and CI
audits every declaration originating in the library against
`[propext, Classical.choice, Quot.sound]`. Someone else arrived at the same
answer independently, which is the first outside evidence that the rule in
`CLAUDE.md` is the normal one rather than an idiosyncrasy.

**Combinatorial game theory was deleted from Mathlib.** Deprecated 2025-08-17,
removed 2026-02-20. No `PGame`, no `SetTheory/Game`. It lives in
`vihdzp/combinatorial-games` now, which is Apache-2.0 and more actively watched
than either complexity library.

That repository is where the category-theory thread reappears, from a direction
nobody planned. It formalizes loopy — cyclic — games as a final coalgebra,
`LGame := QPF.Cofix GameFunctor`, with win, loss and draw outcomes, surviving
strategies and stopper-determinacy. A superko game *is* a loopy game made
finite by the repetition rule, so the fit is real rather than an analogy. What
it would buy over the existing `WinsFor` inductive is not established and
should not be assumed: `WinsFor` is a plain `Prop`-valued inductive over states
and C-13 now gives it a well-founded relation, which may be all this project
ever needs. Recorded as a lead, not a plan.

**The project has been repeating something its source does not say.** C-2's
parenthetical — "the reduction builds no kos" — is not Lichtenstein and
Sipser's. They say they omit the ko rule. The claim that the construction
therefore transfers unchanged to PSK and SSK is Demaine and Hearn's, asserted
in a sentence, and it is about kos rather than about superko-forbidden
repetition, which is strictly stronger. Phase 5 of the plan rested on it as a
premise. It is open work now.

This one is the most useful thing the audit produced, and it came from a slice
that was only in the sweep because the encoding question was worth a look. It
is also the failure mode `docs/style.md` was written against, surviving in the
project's own prose for as long as the prose has existed.

## What the verdict rests on, and does not

The branch decision turns on a cost: that building a machine and bounding its
resources over a downstream library takes months. That number is an inference
from four files in someone else's repository — four trivial languages proved to
be in P, each around eleven hundred lines. Whether it transfers to a Go decider
is untested.

The honest response is to measure it rather than argue about it, so
[`../experiments/002-complexitylib-spike/`](../experiments/002-complexitylib-spike/)
pre-registers the measurement with a ten-day box and a line-count threshold
fixed before it runs. If the spike comes in under the threshold, 001's verdict
is wrong at that scale and the question re-opens. That is the shape a cost
argument should have had in the first place.
