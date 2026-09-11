# First results

The work order. Stops at the point where the plan should be rewritten from what
was learned.

## Phase 0 — unblock

1. ~~**Install Lean.**~~ Done: Lean 4.34.0-rc2, Mathlib cached.
2. ~~**Compile `Defs.lean`.**~~ Done. It builds, and `Sanity.lean` exercises it
   with kernel-checked checks on geometry, chains, liberties, capture, suicide
   and area scoring.
3. ~~**Run experiment 001**~~ Done, and closed. The `infra-gap` markings all
   survive; the reason recorded for them did not. Mathlib has no complexity
   theory, two maintained downstream Lean 4 libraries do, and neither supplies
   what a superko claim needs. The shape of the project is unchanged: formalize
   the combinatorial spine, cite the complexity scaffolding. Experiment 002
   pre-registers the one measurement that could overturn that.

## Phase 1 — settle the definitions

The OPEN items in [`../formal-model.md`](../formal-model.md), in order of how
much rests on them:

1. **OPEN-4 / C-1, the input encoding.** Blocks every complexity claim. Partly
   historical — which encoding do Lichtenstein–Sipser and Robson use — and
   partly a decision this project makes and states.
2. ~~**OPEN-1 / C-18, passes and superko.**~~ Resolved from the held AGA and
   Tromp–Taylor texts: passes are exempt (C-18, `cited`).
3. ~~**OPEN-2 / C-19, pass stones.**~~ Resolved: AGA Rule 12 ignores prisoners
   under area counting (C-19, `cited`). ~~**OPEN-3, komi and ties.**~~
   Resolved: the tie half by theorem (C-27) and the quantification half by
   decision — komi is input, encoded exactly (C-31).

These are reading, not research, and they are the cheapest high-value work
available. C-1's residue is now the identification of the concrete encoding
with the literature's unstated one, plus OPEN-4.

## Phase 2 — prove termination

~~**C-13.**~~ Done. `Superko.C13_terminates`, with `C-26` the quantitative
companion: a game from a root position makes at most 4·3^(m·n) moves.

Three things were learned that the plan did not anticipate.

1. **The lexicographic measure was unnecessary.** Weighting the situation count
   by two flattens the order to a single natural number and one `omega` call per
   case discharges it. The cost is a factor of two in the length bound, which
   nothing downstream needs.
2. **Termination does not depend on C-18.** The proof runs through a named
   property, `Superko.ExcludesRepeats`, that constrains plays only; a run of
   passes is bounded by the pass counter rather than by the history. Whichever
   way the pass question falls, C-13 survives, and so does any variant rule that
   ends the game on two passes.
3. **The pipeline works and is cheap.** A claim moved from `conjecture` to
   `proved`, `check-ledger.sh` verified the theorem exists, `check-lean.sh`
   recorded its axioms. `Defs.lean` was not touched: the finiteness of
   `Situation` is an instance, and an instance is a derived notion.

Determinacy followed on 2026-09-11: C-28, by well-founded induction on the
relation `C13_terminates` supplies, with the exclusivity half needing no
hypothesis at all.

## Phase 3 — validate the definitions

Build `superko-rules` as the executable mirror of the now-settled `Defs.lean`,
then reproduce, in order of increasing cost:

1. legal-position counts `L(m, n)` for small boards (Tromp–Farnebäck);
2. **the 2×2 PSK game count, 386,356,909,593** (C-9) — the load-bearing
   validation;
3. the 1×n minimax score table under PSK, which settles C-11.

Every value lands in `results/` with a witness header.

**If (2) does not reproduce, stop.** The definitions are wrong and nothing built
on them is worth anything.

## Phase 4 — the first novel result

**C-17**, a minimal position whose value differs under PSK and SSK. Search in
`superko-solve`, emit the position and both winning strategies as a certificate,
check the certificate in Lean.

This is the first thing this project would publish: novel, self-contained,
machine-checkable end to end, needing none of the infrastructure experiment 001
is expected to find missing.

## Phase 5 — own the two known bounds

Both bounds on SUPERKO-GO are currently inherited rather than held. Writing
each out for *this* ruleset and this input encoding is what turns them from
citations into results this project can stand behind.

**The EXPSPACE archive algorithm (C-3).** Written out, and it did not move to
`proved`: the grounding decision ([`complexity-grounding.md`](complexity-grounding.md))
splits it into the decider's correctness (C-29, `proved`), the encoding's
honesty (C-31, `proved`), the komi normalization (C-27, `proved`), the
run-level configuration bound (C-30, `open`), and one sentence about Turing
machines that stays prose and is named C-32, `folklore`. The encoding question
did bite, in the opposite direction from the one anticipated: the argument is
insensitive to which of the three history conventions is chosen and fails
under a compressive encoding, which is why C-3 now depends on C-31's lower
length bound. What remains on this item is C-30, and the acquisition of
Robson 1984 and 1985, which may make the whole row `cited`.

**The PSPACE-hardness reduction (C-2).** Harder, and the reason `superko-reduce`
exists. Lichtenstein and Sipser have now been read: their game is a reduced
ruleset, their instances have White to move, and the transfer of their
construction to SUPERKO-GO is C-33, `open`, with no source. Under the §7
language the color to move is input, so White-to-move instances land in the
language directly; what has to be re-proved is the gadget correctness under
situational superko, mechanical area scoring, no suicide and komi. Building it
as a verified gadget is also how the reduction workbench gets built and tested
before it is pointed at anything novel.

## Rewrite this plan here

By the end of Phase 4 the project will know: whether Lean can carry the
complexity layer, whether the definitions are right, and what a formalized
result on this material costs to produce. All three are guesses today.

The attack on the main question — [`../open-questions.md`](../open-questions.md)
§4–6 — should be planned with those answers in hand, not before.
