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
of OPEN-4. The rules questions about passes (C-18, C-19) are unresolved and
are reading, not research.

The grounding for every complexity claim is settled:
[`plans/complexity-grounding.md`](plans/complexity-grounding.md).

## 2. Does the machinery work at all? (C-13, C-9, C-11)

Termination is proved (C-13), with the game-length bound of C-26 alongside it;
so are determinacy (C-28), the correctness of a fuel-indexed archive decider
against `WinsFor` (C-29), the komi normalization (C-27) and the string
encoding with its length bounds (C-31). What remains is the acceptance suite.
C-9 validates the definitions against a number Tromp computed independently;
the decider of C-29 cannot help there — a winner is not a game count, and
kernel evaluation of it reaches only small boards — so Rust remains the only
candidate. C-11
settles a disagreement between two published sources about the 1×9 minimax
score, which is a small real contribution and a sharp test of the kernel.

C-13 and its successors show the formalization pipeline works. They say
nothing about whether the definitions describe Go, which is what C-9 is for —
and until C-9 reproduces, every theorem here is a theorem about `Defs.lean`
rather than about Go.

**Closable in weeks**, and closing it is what makes any later claim credible.

## 3. Positional versus situational superko (C-12, C-17, C-8)

No published complexity separation or equivalence exists. Tromp–Farnebäck's
Lemma 2 gives the structural distinction — PSK games are simple paths in the
situation graph, SSK games need not be — and nobody appears to have asked
whether it has complexity consequences.

Two sub-questions, of very different difficulty:

- **A minimal separating position** (C-17). Real PSK/SSK divergences are
  documented in KGS games, but no minimal case with computed values is
  published. This is a search problem with a certificate: Rust finds the
  position and the two winning strategies, Lean checks both. Fully formalizable,
  and publishable on its own.
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
outcomes over histories.

The decision threshold, fixed in advance:

> If H(n) grows like 2^poly(n) and the classes look computable in exponential
> time, the EXPTIME upper bound is the target. If a fooling set forces
> double-exponential growth, that is evidence against EXPTIME and the effort
> moves to hardness.

Small-board data is suggestive and not decisive — 2×2 and 2×3 may be far too
small to show the asymptotics. A fooling-set *construction*, unlike a
measurement, would be a theorem.

## 5. The upper bound (C-14, C-6)

Generalize Demaine–Hearn beyond Robson's construction: show that Go's live
dynamical state under superko always reduces to something
undirected-geography-like.

The obstacle is stated precisely enough to attack: ko toggles are locally
reversible, and captures in general are not. **Characterize exactly when a Go
move creates a directed edge in situation space.** If the irreversible moves
can be bounded or factored out, the folklore argument might extend; if they
can encode computation, they are the route to hardness instead.

Two things to keep in view. The archive decider of C-29 is doubly exponential
in time (C-34), so C-3's argument lends this conjecture nothing. And by
APSPACE = EXPTIME the conjecture is equivalent to the existence of a
polynomial-space alternating machine deciding the game — the crisp form of the
question, and only an interpretation of it, since such a machine may decide
the game by any characterization at all.

`superko-geography` exists so this is testable rather than quotable: build the
situation graph of a small position and ask whether it is undirected.

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
