# Claim ledger

Every claim this project makes or relies on, with its status and its evidence.
This file is the spine of the project: `docs/` prose may state a claim only if
the claim is here, and may state it only at the strength recorded here.

[`../tools/check-ledger.sh`](../tools/check-ledger.sh) parses the table below
and enforces two invariants:

1. **No `proved` claim depends on a claim that is not `proved` or `cited`.**
   A proof resting on folklore, on a computation, or on a conjecture is not a
   proof, and the gate treats it as a build break.
2. **Every `formalized:` claim names a theorem that exists** in the Lean build
   with a clean axiom list.

## Statuses

| Status | Meaning | May a proof depend on it? |
|---|---|---|
| `proved` | proved here; machine-checked unless the formalization column says otherwise | yes |
| `cited` | proved in the literature; the source gives an actual proof | yes |
| `folklore` | asserted in the literature without proof, or with a sketch too thin to check | **no** |
| `computed` | verified exhaustively on finite instances; evidence about the general case, not proof of it | **no** |
| `conjecture` | believed, with a reason recorded | **no** |
| `open` | genuinely unsettled, including by us | **no** |
| `refuted` | disproved; kept with the counterexample | n/a |

`folklore` is not a slight. The EXPSPACE archive argument is almost certainly
correct. It is `folklore` because no source proves it, and a project whose
whole claim to attention is checkability cannot afford to inherit an unchecked
step.

## Formalization

| Value | Meaning |
|---|---|
| `formalized:Name` | machine-checked; `Name` is the Lean theorem |
| `formalizable` | within reach of current Mathlib; not yet done |
| `infra-gap` | blocked on complexity-theory infrastructure Mathlib lacks (see C-20) |
| `prose-only` | a claim about the literature or the rules; not a mathematical statement |
| `n/a` | not applicable |

## The table

Format is fixed and parsed mechanically: six pipe-delimited columns, ids
matching `C-<digits>`, `depends-on` comma-separated or `-`.

| id | statement | status | formalization | depends-on | witness |
|---|---|---|---|---|---|
| C-1 | Encoding (C) — position as root, empty history — is the faithful formalization of SUPERKO-GO | open | prose-only | - | docs/formal-model.md §5 |
| C-2 | Generalized Go is PSPACE-hard, under every ruleset (the reduction builds no kos) | cited | infra-gap | - | Lichtenstein–Sipser 1980 |
| C-3 | SUPERKO-GO is in EXPSPACE, by carrying an archive of visited situations | folklore | infra-gap | - | Saffidine–Teytaud–Yen 2015, Thm 1 |
| C-4 | Go under Japanese rules is EXPTIME-complete | cited | infra-gap | - | Robson 1983 |
| C-5 | The no-repeat formula game is EXPSPACE-complete | cited | infra-gap | - | Robson 1984 |
| C-6 | Robson's EXPTIME construction, played under superko, stays in EXPTIME via undirected vertex geography | folklore | infra-gap | - | Demaine–Hearn, Playing Games with Algorithms |
| C-7 | Under PSK, games correspond one-to-one with simple paths from the empty position in the situation graph | cited | formalizable | - | Tromp–Farnebäck 2006, Lemma 2 |
| C-8 | Under SSK the corresponding paths need not be simple; a position may be visited twice | cited | formalizable | C-7 | Tromp–Farnebäck 2006, Lemma 2 remark |
| C-9 | The number of 2x2 games under PSK is 386356909593 | cited | formalizable | - | test_data/literature/game-counts.toml (transcribed, unverified) |
| C-10 | Repetition-free games exist whose length is exponential in the board size | cited | formalizable | - | Walraet–Tromp 2016 |
| C-11 | The 1x9 empty-board minimax score under PSK is 0 | open | formalizable | - | disputed: 0 vs 4 across sources |
| C-12 | SUPERKO-GO under PSK and under SSK lie in the same complexity class | open | infra-gap | C-1 | no published separation or equivalence |
| C-13 | Every play sequence under SSK from any start state is finite | proved | formalized:C13_terminates | - | lean/SuperkoComplexity/Results/C13_Termination.lean |
| C-14 | SUPERKO-GO is in EXPTIME | conjecture | infra-gap | C-1 | Robson reported belief; no proof |
| C-15 | The history-congruence index H(n) grows as 2^poly(n) | open | formalizable | C-13 | - |
| C-16 | Mechanical area scoring agrees with AGA agreed scoring under optimal play | conjecture | formalizable | - | docs/formal-model.md §4 |
| C-17 | A minimal position exists whose game value differs under PSK and SSK | open | formalizable | - | KGS anecdotes; no published minimal case |
| C-18 | Under AGA rules a pass is exempt from the superko restriction | open | prose-only | - | AGA Rule 6 vs Tromp–Taylor; unresolved |
| C-19 | Pass stones do not affect the outcome under area scoring | conjecture | formalizable | C-18 | docs/formal-model.md §4 |
| C-20 | Mathlib has no usable resource-bounded complexity infrastructure | open | prose-only | - | experiments/001-mathlib-complexity-audit |
| C-21 | Undirected vertex geography is solvable in time polynomial in the graph size | cited | infra-gap | - | Fraenkel-Scheinerman-Ullman 1993 |
| C-22 | Directed vertex, directed edge and undirected edge geography are PSPACE-complete | cited | infra-gap | - | Geography literature; arXiv:2108.09367 |
| C-23 | The PSK game counts on 1x1, 1x2, 1x3, 1x4 are 1, 9, 907, 2098407841 | cited | formalizable | - | test_data/literature/game-counts.toml (transcribed, unverified) |
| C-24 | The 1xn empty-board PSK minimax scores for n <= 8 are 0, 0, 3, 4, 0, 1, 2, 3 | cited | formalizable | - | test_data/literature/linear-go-scores.toml (transcribed, unverified) |
| C-25 | No published complexity result isolates a repetition rule as the driver of a class change for chess, shogi or xiangqi | open | prose-only | - | confirmed absence; literature search 2026-09-09 |
| C-26 | A game begun from a position as the root of play lasts at most 4·3^(m·n) moves under either superko rule | proved | formalized:C13_length_bound_explicit | C-13 | lean/SuperkoComplexity/Results/C13_Termination.lean |

## Detail

Claims needing more than a row.

### C-1 — the encoding question

Blocks the ledger. Every complexity claim below inherits whichever encoding is
chosen, and the three candidates in [`formal-model.md`](formal-model.md) §5 are
not obviously equivalent. Until this is settled, `C-3`, `C-6`, `C-12` and
`C-14` are claims about an object that is not yet pinned down.

Settling it is partly historical — which encoding do Lichtenstein–Sipser and
Robson actually use — and partly a decision this project makes and states.

### C-3 — the archive bound is inherited, not held

`folklore` because the Saffidine–Teytaud–Yen survey states it as a theorem
whose proof is a single sentence: extend the state with an exponential archive
of visited situations. The argument is almost certainly right.

It stays `folklore` until this project writes it out for its own ruleset and
encoding, which is Phase 5 of
[`plans/first-results-plan.md`](plans/first-results-plan.md) and is the
cheapest real mathematics available here — it depends on nothing else, not even
on C-1 settling, since the archive argument is insensitive to the input
encoding.

### C-13 — termination

Proved, and machine-checked: `Superko.C13_terminates` in
[`../lean/SuperkoComplexity/Results/C13_Termination.lean`](../lean/SuperkoComplexity/Results/C13_Termination.lean),
axioms `propext, Classical.choice, Quot.sound`. The statement is that the play
relation `Superko.Follows SSK` is well-founded, so no infinite sequence of
legal moves exists. `C13_terminates_psk` is the same under positional superko.

The measure is a single natural number, `2 · (unvisited situations) + (passes
remaining)`, not the lexicographic pair this project first proposed. The weight
of two is what absorbs a play's reset of the pass counter, and flattening the
order removes the well-founded-order plumbing at the cost of a factor of two in
C-26.

Termination turns out not to depend on C-18 after all. The proof runs through
`Superko.ExcludesRepeats` — the property that a legal *play* may not recreate a
seen situation — and takes no position on passes at all: what bounds a run of
passes is the pass counter, not the history. `Superko.wellFounded_follows`
states it in that generality, so any variant rule that ends the game on two
passes and forbids repeated plays terminates with no further work. Whichever
way OPEN-1 falls, this claim survives.

What it does not establish: determinacy. `C13_terminates` supplies the
well-founded relation a determinacy proof would recurse on, and is not that
proof. Nor does it show that play must *reach* an ended state — only that it
cannot go on forever.

### C-26 — how long a game can be

`Superko.C13_length_bound_explicit`. A game begun from a position as the root
of play — encoding (C) of [`formal-model.md`](formal-model.md) §5 — makes at
most `4 · 3^(m·n)` moves, because there are `2 · 3^(m·n)` situations
(`Superko.card_situation`) and the measure starts at twice that.

The constant is loose and the exponential is not. Two units of measure are
charged on every play though only one reset ever needs absorbing, so a
lexicographic measure would give something nearer `3^(m·n)`; sharpening it means
a different proof, not a tightening of this one. Nothing downstream needs the
constant.

The exponential is the quantity the complexity question consumes. It is the
reason a machine playing a game out needs exponentially many moves, and it is
where the archive argument of C-3 would get its space bound. This claim does
not make that argument, and says nothing about whether the space is necessary.

### C-14 — the headline conjecture

The reason to believe it is C-6, and C-6 is folklore about a *single
construction*. It says nothing about arbitrary Go positions, and the gap
between them is the research problem. Recorded as `conjecture` rather than
`open` only because a specific person with standing is reported to hold it;
that is a reason to investigate, not evidence.

The route to attacking it is in [`open-questions.md`](open-questions.md).

### C-21, C-22 — the geography results

C-21 is what makes C-6 more than an analogy: if undirected vertex geography
were not polynomial, the folklore reason to believe superko Go is in EXPTIME
would evaporate. The ledger records C-6 as depending on it.

C-22 is the other half of the picture, and the reason
[`open-questions.md`](open-questions.md) §5 poses the question it does: the
directed/undirected distinction is exactly what separates polynomial from
PSPACE-hard for the reachability core, so *characterizing when a Go move
creates a directed edge in situation space* is the concrete form of the
upper-bound attack.

### C-23, C-24 — transcribed, not verified

Both rest on values copied from secondary accounts rather than read from the
papers — the game counts from Tromp's 2×2 solver page and the *Combinatorics of
Go* abstract, the 1×n scores from Hayward's course table — and every source
involved is still `sought` in
[`../references/README.md`](../references/README.md). They are `cited` because
the literature does establish them, but a failure to reproduce them has three
possible causes rather than two — see
[`../test_data/literature/README.md`](../test_data/literature/README.md).

Promote the witness to a real citation once a primary source is `held`.

### C-25 — a confirmed absence

Chess threefold repetition, shogi sennichite and xiangqi perpetual-check rules
all make legality or outcome history-dependent, and the standard
EXPTIME-completeness proofs for those games handle repetition with a turn or
move counter rather than by analyzing the repetition rule itself. No result
appears to isolate the repetition rule as the thing that moves the complexity
class.

Recorded as a claim because it is the novelty argument for
[`open-questions.md`](open-questions.md) §4, and because an absence is exactly
the kind of assertion that decays: it was true of a 2026-09-09 literature
search and should be re-checked before anything is written up.

### C-20 — the infrastructure finding

Not mathematics, but it determines how much of this project can be
machine-checked, so it is tracked like a claim. `infra-gap` in the
formalization column above is an assertion of C-20 and every such row is
provisional until the audit is done.
