# 2026-09-09 — C-13, and what termination did not need

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)

First claim to move to `proved`. The mathematics took an afternoon; three
things about it were worth more than the theorem.

## The lexicographic measure was not needed

[`docs/plans/first-results-plan.md`](../docs/plans/first-results-plan.md)
proposed a lexicographic measure on (unvisited situations, pass counter),
decreasing on every move. That works, and it is more machinery than the
problem needs.

Weight the situation count by two instead:

```
playMeasure st = 2 * (card (Situation m n) - st.seen.ncard) + (2 - st.passes)
```

A play spends at least one unit of the first summand — the history strictly
grows — and is handed back at most two units of the second when `passes` resets
to 0. Since `¬ Ended st` means `st.passes < 2`, at least one of those two units
was already spent at the source, so the play nets a strict drop. A pass leaves
the first summand weakly smaller and drops the second by exactly one. Both cases
are `omega` after the right inequalities are in hand, and the well-founded order
disappears: the measure is a natural number.

The cost is a factor of two in the length bound. The factor is charged on every
play though only one reset ever needs absorbing, so a lexicographic measure
would give something nearer `3^(m·n)` than `4·3^(m·n)`. Sharpening it is a
different proof rather than a tightening of this one, and nothing downstream
needs the constant — the exponential is the part that matters.

## Termination does not depend on OPEN-1

This is the finding, and it was not expected.

The ledger's C-13 entry read: *"Note its dependence on C-18: if passes were
subject to superko, the pass counter would be unnecessary but the argument
would change shape."* That is wrong, and the formalization is what showed it.

Nothing in the proof consults the repetition rule's treatment of passes. What
it needs is one property of *plays*:

```
ExcludesRepeats L : ∀ st p, L st (Move.play p) → st.now.after (Move.play p) ∉ st.seen
```

Both `SSK` and `PSK` have it — `SSK` by definition, `PSK` because a rule
forbidding a repeated position forbids a repeated situation a fortiori. A run of
passes is bounded by the pass counter, which is a field of `State` rather than
anything the rule sees. So `wellFounded_follows` is stated for an arbitrary
rule with that property, and C-13 and its PSK twin are one-line instantiations.

Whichever way the pass question falls, C-13 survives untouched, and so does any
future variant rule that ends the game on two passes and forbids repeated
plays. One less thing blocked on source acquisition.

The general lemma was not in the specification I wrote. It came out of one of
the parallel attempts and was grafted in.

## `Defs.lean` was not touched

The one addition termination needed to the core was finiteness of `Situation`,
and finiteness is an instance. `Fintype Color` and `instFintypeSituation` live
in `Basic.lean`, where the kernel checks them and no auditor has to read them.

Resist the temptation to put `Fintype` in `Color`'s `deriving` clause. It would
be one word, and it would enlarge the audit target and invalidate prior auditing
for nothing.

## Wrinkles, recorded so nobody rediscovers them

- **`playMeasure` must be `noncomputable`.** `State.seen` is a `Set`, so
  `Set.ncard` is `Nat.card` and does not compile. The consequence is real rather
  than cosmetic: no certificate checker can evaluate the measure, so any future
  `computed` claim about measure values needs a decidable counterpart and a
  bridging lemma, in the pattern `Basic.lean` already uses for `clear'` and
  `area'`.
- **`Set.ncard_insert_of_not_mem` no longer exists** under that name. The
  `not_mem` → `notMem` rename landed; it is `Set.ncard_insert_of_notMem`. Pass
  the finiteness argument explicitly as `Set.toFinite _` rather than leaving it
  to the autoparam — more robust across revisions.
- **`Follows` is oriented successor-first**, because `Acc r x` looks at every
  `y` with `r y x`. Written the other way round, `WellFounded (Follows L)` would
  assert the opposite of what is wanted and would be false. Worth stating in the
  file, since it is the thing an auditor is most likely to check backwards.
- **`decide` on `Fintype.card (Situation 2 2) = 162` exceeds `maxRecDepth`**
  when it has to compute the card from the instance. Proving `card_situation`
  in general and then rewriting makes both the general fact and the spot checks
  cheap — and a `decide` spot check would have been `computed`, not `proved`,
  in any case.

## Method

Five independent formalizations were produced in parallel against statements
fixed in advance, each then audited by a separate pass that re-elaborated the
file, compared every signature token by token against the specification, and —
the check that mattered — constructed a concrete legal move on a small board to
establish that `Follows` is inhabited and the theorems are not vacuously true.

All five compiled and proved the stated theorems. They differed only in
legibility and in how much of `Defs.lean`'s definitional shape they inlined into
proof bodies; the one chosen isolates every such dependence in a named one-line
lemma, so a change to `start` — which C-1 may well force — breaks a small named
site rather than the interior of a theorem.

Fixing the statements before delegating was the part that mattered. Every one of
the five hit the `noncomputable` problem and reported it rather than working
around it, which is the behavior the arrangement was designed to produce.

## What is not proved

Determinacy. `C13_terminates` supplies the well-founded relation a determinacy
proof would recurse on; that proof is not written, and the recursion has to
handle the `WinsFor` constructors, which is real work rather than a corollary.

That play *reaches* an ended state. Only that it cannot go on forever. Nothing
here says a legal move always exists.

Anything about Go. These are theorems about `Defs.lean`, whose definitions this
project records as plausible rather than validated. The independent-agreement
argument of [`docs/trusted-base.md`](../docs/trusted-base.md) — reproducing the
2×2 game count — has not been run, and until it is, a green checkmark here is a
green checkmark about a formal object that has not been shown to be Go.
