# First results

The work order. Stops at the point where the plan should be rewritten from what
was learned.

## Phase 0 — unblock

1. ~~**Install Lean.**~~ Done: Lean 4.34.0-rc2, Mathlib cached.
2. ~~**Compile `Defs.lean`.**~~ Done. It builds, and `Sanity.lean` exercises it
   with kernel-checked checks on geometry, chains, liberties, capture, suicide
   and area scoring.
3. **Run experiment 001** (the Mathlib complexity audit). Still the next thing.
   Its outcome decides how much of this project can be machine-checked and
   whether the `infra-gap` markings in the ledger survive.

## Phase 1 — settle the definitions

The OPEN items in [`../formal-model.md`](../formal-model.md), in order of how
much rests on them:

1. **OPEN-4 / C-1, the input encoding.** Blocks every complexity claim. Partly
   historical — which encoding do Lichtenstein–Sipser and Robson use — and
   partly a decision this project makes and states.
2. **OPEN-1 / C-18, passes and superko.** A rules question. Read AGA Rule 6
   against Tromp–Taylor. The answer changes the shape of the termination
   argument and the PSK/SSK distinction.
3. **OPEN-2 / C-19, pass stones**, and **OPEN-3, komi and ties.** Smaller.

These are reading, not research, and they are the cheapest high-value work
available.

## Phase 2 — prove termination

**C-13.** Every play strictly enlarges `State.seen`, bounded by the finite set of
situations; passes leave it alone but advance a counter that ends the game at
two. A lexicographic measure on (unvisited situations, pass counter) decreases
on every move.

Load-bearing — determinacy and the definability of the game value both rest on
it — and elementary, which makes it the right place to find out what
formalizing on this material actually costs. It also exercises the whole
pipeline once: a claim moves from `conjecture` to `proved`, the ledger's
formalization column names a theorem, `check-ledger.sh` verifies it exists,
`check-lean.sh` records its axioms.

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

**The EXPSPACE archive algorithm (C-3).** Cheap, and can be done as soon as the
definitions settle — it needs no reduction machinery. The literature states it
in one sentence and no source proves it, which is why the ledger marks it
`folklore`. Writing it out is what moves it to `proved`, and it is the first
chance to find out whether the encoding question (C-1) actually bites: the
argument is insensitive to encoding, and confirming that is worth something.

**The PSPACE-hardness reduction (C-2).** Harder, and the reason `superko-reduce`
exists. Lichtenstein–Sipser's construction builds no kos at all, so it should
carry to SSK unchanged — but "should" is the word the ledger exists to
eliminate. Building it as a verified gadget, against a reduction already known
to be correct, is also how the reduction workbench gets built and tested before
it is pointed at anything novel.

Do the archive algorithm early, out of order, if Phase 1 stalls on source
acquisition. It is the one piece of real mathematics here that depends on
nothing else.

## Rewrite this plan here

By the end of Phase 4 the project will know: whether Lean can carry the
complexity layer, whether the definitions are right, and what a formalized
result on this material costs to produce. All three are guesses today.

The attack on the main question — [`../open-questions.md`](../open-questions.md)
§4–6 — should be planned with those answers in hand, not before.
