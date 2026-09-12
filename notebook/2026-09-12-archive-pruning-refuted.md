# 2026-09-12 — the archive prunes to nothing

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)

The upper-bound thread's first real attempt, and it is a refutation. The
question put to it was how to put numbers to the compressibility of superko.
The answer that came back is that the phrase hides three questions, that one of
them now has an answer, and that the answer is no.

## What was asked

The standing intuition: the EXPSPACE archive of C-3 carries every visited
situation, `2 · 3^(m·n)` of them, and surely most of it cannot matter to the
value. If the part that matters is small, C-47's backward induction gives
C-14 and the upper bound moves for the first time since 1980.

Three inequivalent readings of "the part that matters", which had been running
together in [`../docs/open-questions.md`](../docs/open-questions.md) §§4–5:

1. **How many archives are value-distinguishable** — C-15's index.
2. **How few bits suffice for a summary** that fixes legality and the value and
   updates cheaply — the alternating reading, by C-48.
3. **Is such a summary a subset of the archive** — the reading every concrete
   pruner assumes without saying so.

The third is the one that looked easiest and it is the one that fell.

## The theorem came out first, and it is exact

[`../lean/SuperkoComplexity/Compress.lean`](../lean/SuperkoComplexity/Compress.lean).
`winsFor_transfer` (C-41) is the schema everything else factors through: a
relation preserving the situation, the pass counter and legality, and surviving
`step`, preserves who wins — parametric in the repetition rule, so the
positional twin is proved rather than asserted. From it, `winsFor_cone_congr`
and `winsFor_seen_inter_cone` (C-42): the value reads the archive only inside
the forward cone, so anything outside may be dropped and anything unreachable
may be added.

Two things this cost that the plan did not anticipate.

**The obvious invariant is false.** The first proof attempt carried
`t.seen = s.seen ∩ Cone s.now` through `step`. It is not preserved: the cone
*shrinks* under a move while the intersection does not, so the two come apart
after one step. The invariant has to be weakened to *agreement on the cone*,
which is what `winsFor_cone_congr` takes as its hypothesis. The hand argument
had this wrong and the elaborator caught it, which is the whole reason to write
these down in Lean rather than in prose.

**C-29 is stated at the root only.** Every mid-game statement needs the decider
at an arbitrary archive, and `C29_decideWins_iff_blackWins` does not give it.
`decideWins_iff_winsFor` (C-43) is the generalization; it costs only the fuel
arithmetic, and it enlarges what a reader of the trusted base audits from the
special root to the general state. Recorded here rather than absorbed.

## Then the census said it was worth nothing

`superko-graph::scc` builds the situation graph — vertices are situations,
edges are `Compress.SitStep`, a pass always and a play the board permits, with
no repetition rule in it — and runs Tarjan.

| board | legal situations | components | largest | outside largest | depth |
|---|---|---|---|---|---|
| 1×1 | 2 | 1 | 2 | 0 | 1 |
| 1×2 | 10 | 3 | 4 | 6 | 2 |
| 1×3 | 30 | 2 | 28 | 2 | 2 |
| 2×2 | 114 | 2 | 112 | 2 | 2 |
| 3×3 | 25 350 | 2 | 25 348 | 2 | 2 |
| 3×4 | 643 378 | 2 | 643 376 | 2 | 2 |

Two components on every board from 1×3 to 3×4: the empty board's pass 2-cycle,
and one component holding every other situation. So the cone of any non-empty
situation is the whole of that giant component, and C-42's prune removes at
most two entries from an archive of up to `2 · 3^(m·n)`. That is C-45 and C-46,
both `computed`, and it refutes the whole subset-of-the-archive family — cone,
strongly connected component, reachability ball — not just the one instance
proved.

1×2 is the exception and is kept. There the giant component splits in two:
from `X.` only White can move and only to `.O`, from which only Black can move
and only back, and the mirrored pair is a second 2-cycle the first never
reaches. 1×1 has one component because no stone can ever be played.

## The one place the structure was not an accident

The two components are two rather than one because of a theorem, not a
measurement. `resolve` clears `c.other` only, so the played stone survives
(`resolve_self`), so no play produces the empty board, so the empty situation
has in-degree zero under play edges — on every board, for every rule (C-44).

The census corroborates it in the sharpest way available. Under the
suicide-permitting convention — Tromp–Farnebäck's, with no counterpart in
`Defs.lean`, where a self-capture *can* take the played stone back off — the
empty board acquires an incoming edge and the graph collapses to a **single**
component on every board censused, where the prune removes nothing at all. The
hypothesis of the theorem and the shape of the data move together, which is
about as much definitional corroboration as a structural claim gets.

This was not the prediction. The route into the census was that irreversibility
should give a progress measure: Go adds stones, captures are rare, so the
condensation should be deep and each component small, and the archive should
only ever need the current component. The condensation is two deep and one
component holds everything. The prediction was wrong by as much as it could be
and the error was in the same direction throughout — every step of the informal
argument was locally plausible.

## Two corrections to text that was already in the repository

**"Under superko a Go game is a self-avoiding walk in situation space"** —
[`../docs/state-of-the-art.md`](../docs/state-of-the-art.md) and
`superko-geography`'s module doc. That is C-7, and it is the *positional*
statement. Under SSK a pass is exempt from the repetition rule (C-18) and
re-enters a situation already seen, so the walk avoids itself on play edges
only (C-8). Both sites are fixed.

**"A fooling set forcing double-exponential growth is evidence against
EXPTIME"** — `open-questions.md` §4's threshold, fixed in advance and wrong. An
EXPTIME algorithm need not enumerate states; the input is a root position alone
(C-1), so nothing obliges an algorithm to summarize an interior history; an
alternating machine may guess a summary and verify it universally; and among
PSPACE ⊆ EXPTIME ⊆ EXPSPACE only the outer inclusion is known strict. A large
index kills an approach, which is a different and much smaller thing. The
threshold now says so.

The same section now carries the two asymmetries that govern every number this
thread will produce: measurement runs from the empty root while the theorem
quantifies over all `3^(m·n)` colorings, so an encouraging number licenses
nothing and only a discouraging one is evidence; and a degree-(K−1) polynomial
interpolates K points, so no series over the handful of areas a history-carrying
enumeration reaches can reject "polynomial" whatever it returns.

## Two rows that were being spent without one

C-47, backward induction — bounded effective state implies EXPTIME — is the
second half of every "the history compresses, therefore C-14" argument, and it
appeared nowhere in `docs/`, `proofs/`, `notebook/` or `lean/` before today
while the shape of the argument appeared repeatedly. C-48 is APSPACE = EXPTIME,
which §5 has been quoting from Hearn's uncited restatement. Both are `folklore`
here rather than `cited` because the sources exist and this project has not
read them; obtaining Chandra–Kozen–Stockmeyer moves C-48 and costs an
afternoon.

C-49 is the third: C-21, which C-6 rests on, is a **normal-play** theorem —
the player unable to move loses, and the algorithm is a maximum matching. A
superko Go game never becomes immobile, because a pass is always legal, and it
ends by area scoring. The theorem that would carry C-6 to arbitrary Go does not
exist, and until today neither did the row saying so. This matters more than
the other two, because C-6 is the entire reason to believe C-14.

## What is still live

Not C-42, which is exact and stays. The *subset* form of compression, and only
that. A summary that is not a subset of the archive — a hash, a quotient
representative, an automaton state, a matching certificate — is untouched by
any of this, and C-15's index is still the question: how many archives are
distinguishable, not which subset of one suffices.

The next question is the right form of §5's, and the census does not answer it:
**how many edges inside the giant component are one-way?** C-6's mechanism
needs that number small, not the component small, and the crude question this
repository used to pose — is the graph undirected — answers no immediately and
uninformatively, since any stone the opponent cannot capture is a one-way edge.

## Not an experiment

There is no directory under `experiments/` for this and there should not be.
The census was run inside a survey, against a prediction that was never
registered, and writing the falsification criterion afterwards — when the answer
was already on the screen — would be the exact defect
[`../experiments/TEMPLATE.md`](../experiments/TEMPLATE.md) exists to prevent.
It is a `computed` ledger row with a witness command and a notebook entry, and
that is the honest shape for it.
