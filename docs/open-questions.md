# Open questions

The live fronts, each pointing at the claims that would close it. Ordered by
what this project can plausibly reach, not by importance.

## 1. What is the problem? (C-1, C-18, C-19)

Narrower than it was. The komi question is resolved (OPEN-3: komi is input,
encoded exactly, licensed by C-27), the bit-level encoding is concrete
(C-31), and the literature's convention for the input — position as root,
no history — is now read from the sources rather than assumed. What keeps
C-1 open is the identification of that encoding with the literature's
unstated one, which is asserted by inspection, and the reachability question
of OPEN-4. The rules questions about passes are settled by the texts, now
held: passes are exempt from repetition (C-18) and pass stones cannot enter
an area count (C-19). The root situation counting as a previous position is
the reading both texts support.

The grounding for every complexity claim is settled:
[`plans/complexity-grounding.md`](plans/complexity-grounding.md).

## 2. Does the machinery work at all? (C-13, C-9, C-11)

Termination is proved (C-13), with the game-length bound of C-26 alongside it;
so are determinacy (C-28), the correctness of a fuel-indexed archive decider
against `WinsFor` (C-29), the komi normalization (C-27) and the string
encoding with its length bounds (C-31). The first row of the acceptance suite
is in: the Rust mirror reproduces Tromp's 2×2 count under `Defs.lean`'s
rules (C-39, `computed`), and along the way found the suicide convention
observable at 1×4 (C-36). The decider of C-29 had no part in it — a winner
is not a game count, and kernel evaluation of it reaches only small boards.
C-11 settles a disagreement between two published sources about the 1×9
minimax score, which is a small real contribution and a sharp test of the
kernel, and is the row still open.

C-13 and its successors show the formalization pipeline works. C-39 is the
evidence that the definitions describe Go, and it is `computed` evidence on
one board about one quantity: it sees neither komi nor scoring, and no
kernel has checked a count. Every theorem here is still a theorem about
`Defs.lean`; what changed is that `Defs.lean` now agrees with Tromp where
the two can be compared.

The 1×n values reproduce for n ≤ 6 (C-24); 1×7 and 1×8 do not resolve within
the solver test's budget of 4 × 10⁷ nodes, and C-11's 1×9 is not attempted.
The 2×2 row is closed.

## 3. Positional versus situational superko (C-12, C-17, C-8)

No published complexity separation or equivalence exists. Tromp–Farnebäck's
Lemma 2 gives the structural distinction — PSK games are simple paths in the
situation graph, SSK games need not be — and nobody appears to have asked
whether it has complexity consequences.

Two sub-questions, of very different difficulty:

- **A minimal separating position** (C-17). Real PSK/SSK divergences are
  documented in KGS games, but no minimal case with computed values is
  published. Under `Defs.lean`'s rules none exists on any board of at most five
  points (C-53, `computed`), although its searches make plays SSK permits and
  PSK refuses on some of those boards; under the suicide-removing convention
  the least is `X.` on 1×2 (C-54, `computed`). Under that convention separation
  is not monotone in the board; under `Defs.lean`'s rules nothing is known
  either way, so the absence at five points bounds nothing at six, and the
  boards of six points are unswept. The shape of a separating play is settled (C-50 and C-51,
  `proved`; C-52, `proved` by hand): it closes an odd walk of at least three
  plays with no pass at the recurring board, so a value separation needs a
  position where closing that walk is worth a player's while. A witness, once
  found, would be certified by kernel evaluation of `Superko.decideWins` under
  both rules at one komi, if that evaluation reaches its board. Fully formalizable, and publishable on its own.
- **A complexity separation** (C-12). Much harder, and `infra-gap`.

**C-17 is the best short-term target in the project**: novel, self-contained,
machine-checkable end to end, and it needs no complexity-theory infrastructure.

## 4. The history congruence (C-15)

Define the Myhill–Nerode congruence on histories — when do two histories leave
the same options open, in the sense that no continuation distinguishes them —
and measure its index H(n).

`H`, not `N`: Tromp–Farnebäck's `N(n)` counts *games*, and this project needs
both quantities in the same sentence often enough that reusing the letter would
be a defect.

No prior formalization of a history congruence for games with history-dependent
legality appears in the literature. The nearest object is Tromp–Farnebäck's
border-state automaton, which compresses the static legality predicate, not
outcomes over histories. **Read that absence with the caveat
[`state-of-the-art.md`](state-of-the-art.md) now carries**: the
graph-history-interaction line — Kishimoto and Müller on Go under superko above
all — was never examined, and it is about exactly when two histories at one
position may be identified. A search heuristic is not a bound on the index, but
that is a reason to read them rather than to keep asserting the absence.

The decision threshold, fixed in advance, and **corrected**:

> If H(n) is proved to grow like 2^poly(n), uniformly in the board and over
> every root, then with C-47 the EXPTIME upper bound follows and is the target.
> If a fooling set forces double-exponential growth, that refutes the
> history-summary route and the effort moves to hardness.

The second clause used to read "that is evidence against EXPTIME". It is not,
and the difference is the one this project can least afford to blur. An EXPTIME
algorithm need not enumerate states: the decision problem takes a root position
alone (C-1, encoding (C)), so no algorithm is obliged to summarize an interior
history at all; an alternating machine may guess a summary and verify it
universally, and a deterministic index bounds neither; and among
PSPACE ⊆ EXPTIME ⊆ EXPSPACE only the outer inclusion is known strict, so
nothing short of a major separation places this game outside EXPTIME. A large
index kills an approach. The first clause is sound only because of its second
conjunct, which is C-47 and is now a row.

Two asymmetries govern every number this section will ever produce, and they
are stated here once rather than rediscovered per measurement.

**Measurement runs from the empty root; the theorem quantifies over every
root.** `Position m n` ranges over all `3^(m·n)` colorings. An empty-root
measurement is therefore a *lower* bound on the quantity an upper-bound
argument consumes: an encouraging number licenses nothing, and only a
discouraging one is valid evidence.

**Small boards cannot separate the shapes.** A degree-(K−1) polynomial
interpolates K points exactly, so no series over the four or five areas a
history-carrying enumeration reaches can reject "polynomial" whatever it
returns. A fooling-set *construction*, unlike a measurement, would be a
theorem, and is the only thing in this section that could move a row.

The first attempt on this front is recorded and it failed: pruning the archive
to a subset of itself is refuted (C-46). See §5.

## 5. The upper bound (C-14, C-6)

Generalize Demaine–Hearn beyond Robson's construction: show that Go's live
dynamical state under superko always reduces to something
undirected-geography-like.

### What "the history compresses" has to mean

The loose phrase hides three questions with different targets, and only the
first two can yield an upper bound.

- **How many archives are value-distinguishable?** Target `2^poly(m·n)`. This
  is C-15's index H, and it is what C-47's backward induction consumes. The
  names may be `2^O(n)` bits in the input length: an EXPTIME budget affords a
  full archive bit-vector as a canonical name.
- **How few bits suffice for a summary** that fixes legality, fixes the value
  and updates in polynomial time? Target `poly(m·n)`. This is the alternating
  reading, by C-48.
- **Is such a summary a subset of the archive?** **Answered, and the answer is
  no** — C-46.

The second implies the first and not conversely, so the alternating form is the
*stronger* hypothesis; this section used to state only it and call it the crisp
form of the question. It is the crisp form of the equivalence C-48 asserts, not
of what membership needs, and it is not the form C-6's own mechanism uses — a
maximum matching on an exponentially large graph is polynomial in the graph and
nowhere near polynomial in the board.

### What is closed

Pruning the archive to a subset of itself. `Compress.winsFor_seen_inter_cone`
(C-42, `proved`) says the value reads the archive only inside the forward cone,
which is exact and which bounds every scheme of this shape — cone, strongly
connected component, reachability ball. The census says the bound is worthless:
above the empty board the situation graph is a single mutually reachable
component on every board through `m·n = 12` (C-45), so the prune removes at
most two entries from an archive of up to `2 · 3^(m·n)` (C-46). A summary that
is not a subset — a hash, a quotient representative, an automaton state — is
untouched, and is now the only live form.

### What is open

The obstacle is stated precisely enough to attack: ko toggles are locally
reversible, and captures in general are not. **Characterize exactly when a Go
move creates a directed edge in situation space.** Ask it *inside* the giant
component, not about the graph as a whole: "is the situation graph undirected"
answers no immediately and uninformatively, because any stone the opponent
cannot capture is a one-way edge. The quantity C-6's mechanism needs small is
the number of one-way edges within the component, not the component.

Three things to keep in view.

1. The archive decider of C-29 is doubly exponential in time (C-34), so C-3's
   argument lends C-14 nothing.
2. C-21, which C-6 rests on, is a **normal-play** theorem — the player unable
   to move loses, and the algorithm is a maximum matching. A superko Go game
   never becomes immobile, because a pass is always legal (C-18), and it ends
   by area scoring. The theorem that would carry C-6 to arbitrary Go is C-49,
   `open`, and no source has it.
3. No result in this section, in any branch, bears on C-14. A bound on one
   summary scheme bounds no complexity class; see §4's corrected threshold.

## 6. The lower bound (C-5, C-10)

Combine the Walraet–Tromp Gray-code construction with a Robson-style formula
game encoding, making the Gray-code walk *steerable* by the players so it
simulates the no-repeat formula game.

The Gray code establishes that exponentially long repetition-free games exist,
so length is not what blocks a hardness proof. The question nobody appears to
have posed in print is whether such long sequences can be made *programmable*.
That framing alone is worth writing down.

Note the asymmetry that makes this the harder direction. A hardness result here
would be prose with a formalized combinatorial core rather than a machine-checked
theorem end to end — not because no Lean library defines the classes (C-20
records that two do) but because none supplies a hard source problem in the
right model, and building the reduction machine is the whole cost. The
checkable content is gadget correctness, which is a statement about Go
positions and needs no complexity vocabulary at all.

## What would change course

- **A Lean library on this toolchain gains a string-model PSPACE-complete
  problem under polynomial-time reductions together with proven, public
  machine-composition combinators.** The grounding decision
  ([`plans/complexity-grounding.md`](plans/complexity-grounding.md)) keeps every
  complexity library out of `lean/` because none proves that its class is the
  textbook's; a hard source with composition would let a hardness statement be
  formalized end to end and is the specific trigger to revisit that. The
  companion trigger is experiment 003 firing on its first outcome: a
  machine-checked membership for the archive decider in a disclosed side base.
- **The Demaine group publishes on superko formula games.** Chung's 2026 thesis
  lists it as future work. If they reach Go first, this project's contribution
  is the small formalized results, and it should be packaged as such quickly.
- **C-9 fails to reproduce.** The definitions are wrong. Stop and fix them
  before anything else.
