# State of the art

What the literature establishes, by whom, and where it stops. Each result is
tied to its ledger row; the ledger, not this document, records how much each
one may be leaned on.

Every source named below is listed in
[`../references/README.md`](../references/README.md) with its acquisition
status. Several are now `held`: Lichtenstein–Sipser 1980, Hearn 2006, the
Saffidine–Teytaud–Yen draft and Demaine–Hearn were read in full during the
grounding survey of 2026-09-10. Robson's papers remain unread, and a claim
resting on a detail of an unread source says so in its ledger row.

## The bracket

Generalized Go under superko is **PSPACE-hard** (C-2) and **in EXPSPACE**
(C-3). Neither bound has moved since 1980. The three candidate answers —
PSPACE-complete, EXPTIME-complete, EXPSPACE-complete — are all live, and the
people who have looked do not agree.

The status of the two bounds is not symmetric. C-2 is `cited`, for a narrower
game than the row used to say: Lichtenstein and Sipser prove PSPACE-hardness
for a reduced ruleset — ko omitted, a capture order under which suicide is
effectively legal, territory scoring with judged dead stones, no komi — with
White to move, and they never name the reduction resource. Whether their
construction carries to SUPERKO-GO as this project defines it is C-33,
`open`; the claim that it does is Demaine and Hearn's, asserted without
argument, and it concerns kos rather than superko-forbidden repetition, which
is strictly stronger. C-3 is `folklore`: the Saffidine–Teytaud–Yen survey
states it as a theorem whose proof is one sentence — extend the state with an
exponential archive of visited situations — and labels it folklore itself.
This project has written the argument out and split it: the decider's
correctness, the encoding's honesty and the komi normalization are
machine-checked (C-29, C-31, C-27), the configuration bound is open (C-30),
and the one remaining sentence — that a Turing machine iterating the decider
runs in space polynomial in its configuration — is named as C-32 and stays
prose, because it is the invariance thesis instantiated to this project's own
functions and no source proves it. The grounding for that arrangement is in
[`plans/complexity-grounding.md`](plans/complexity-grounding.md).

## Why the Japanese-rules result does not transfer

Robson proved Go under Japanese rules EXPTIME-complete in 1983 (C-4). **Both
halves of that proof depend on properties superko removes**, which is the
entire reason the superko question is open:

- *Membership.* Under Japanese rules the relevant state is polynomial — the
  board plus a bounded ko history — so the exponential game tree yields to
  exhaustive search. Under superko, legality depends on the whole history, and
  only the exponential archive is known to suffice.
- *Hardness.* The construction encodes computation in controlled ko cycles,
  which the basic ko rule permits and superko curtails.

## Why the EXPSPACE meta-theorem does not transfer either

Robson's 1984 no-repeat meta-theorem (C-5) is the one rigorous result tying a
superko-like rule to a complexity jump: adding a no-repeat rule to certain
EXPTIME-complete games yields EXPSPACE-complete games. It gives
EXPSPACE-completeness for superko chess and superko checkers.

It requires that the *entire dynamical state* of the underlying game live in
the global position, so that the no-repeat constraint binds and forces
exponential history tracking. Chess and checkers supply this. Go, in Robson's
construction, does not: the dynamical state lives in kos, which are local and
reversible. Under superko such a game becomes undirected vertex geography,
solvable in time polynomial in the graph size — and the graph is exponential,
so the construction stays in EXPTIME (C-6).

**C-6 is the strongest reason to suspect EXPTIME, and it is folklore about one
construction.** It is not a statement about Go. The gap between "Robson's
gadget behaves this way" and "every Go position behaves this way" is the
research problem.

## What has appeared since

Restricted and variant Go, not the main question:

| Result | Status |
|---|---|
| Ladders are PSPACE-complete — Crasmaru–Tromp 2000 | cited |
| A class of tsume-go problems is NP-complete — Crasmaru 1998 | cited |
| Crasmaru's master's thesis re-derives Robson's EXPTIME-hardness using ladders in place of his "pipes" | partial — described by the author on the computer-go list, thesis not held |
| Go endgames are PSPACE-hard — Wolfe 2002 | cited |
| Atari-Go is PSPACE-complete; hardness carries to Phantom Go — Saffidine–Teytaud–Yen 2015 | cited |
| Kill-all Go is PSPACE-hard under Chinese rules, EXPTIME-complete under Japanese — Zhang 2019 | cited |

Zhang's concluding section poses exactly this project's question, and reaches
the same trichotomy: Kill-all Go under Chinese rules "might be PSPACE-complete,
EXPTIME-complete or even EXPSPACE-complete".

No paper in any venue improves either bound for PSK or SSK Go. That is a
confirmed absence as of a 2026-09-09 search of arXiv, DBLP and the Computers
and Games, ACG, Games of No Chance and BIRS proceedings, and it should be
re-checked before anything is written up.

## The nearest related work in print

Chung's MIT master's thesis (February 2026) classifies the Chandra–Stockmeyer
formula games and their variants, and its future directions name superko
formula games explicitly.

Two things follow, both about the literature rather than about anyone's plans.
It is a 2026 source that presents the superko variants in print as unsettled
and worth investigating, which is evidence the question is live rather than
merely unclosed. And it is the nearest published point of contact with this
project: formula games under a no-repeat rule are the object Robson 1984
settles (C-5), and whether that transfers to Go is C-14.

Superko formula games and superko Go remain different problems, and no
published work connects them. Future work in this line is worth watching;
[`../references/README.md`](../references/README.md) records who publishes on
the material.

## The structural results this project builds on

These, rather than the complexity results, are where the formalizable content
is:

- **Tromp–Farnebäck 2006** (C-7, C-8). Exact legal-position counts; the base of
  liberties ≈ 2.9757341920433572493; and Lemma 2 — under PSK, games correspond
  one-to-one with *simple paths* from the empty position in the situation
  graph, while under SSK the paths need not be simple. That lemma is the
  cleanest published structural distinction between PSK and SSK and nobody has
  asked what it implies for complexity.
- **Walraet–Tromp 2016** (C-10). A googolplex of Go games, via a Gray-code
  construction that realizes each sign change as a capture sequence. It
  establishes that exponentially long repetition-free games exist, so **game
  length is not what blocks a hardness proof**. Whether such sequences can be
  made to encode computation appears to be unposed in print.
- **Geography variants** (C-21, C-22). Undirected vertex geography is
  polynomial (Fraenkel–Scheinerman–Ullman 1993); directed vertex, directed edge
  and undirected edge geography are PSPACE-complete. Under superko a Go game is
  a self-avoiding walk in situation space, which makes these the right
  analogues, and the directed/undirected line is precisely what separates
  polynomial from PSPACE-hard for the reachability core. C-21 is what C-6 rests
  on.

## The counting numbers

Context for how large the objects are, all from Tromp–Farnebäck. The game
counts are read from Table 7 of the held 2016 revision; the rest are still
transcribed — see
[`../test_data/literature/README.md`](../test_data/literature/README.md):

| Quantity | Value |
|---|---|
| Legal positions, 19×19 | L(19,19) ≈ 2.082 × 10^170 |
| Base of liberties | lim L(m,n)^(1/mn) ≈ 2.9757341920433572493 |
| Games on 19×19 | 10^(10^48) < N(19) < 10^(10^171), stated as an open problem |
| Games on 1×n | at least 2^(2^(n−1)) |
| Exact game counts under PSK | 1×1: 1, 1×2: 9, 1×3: 907, 1×4: 2,098,407,841, 2×2: 386,356,909,593 (C-9, C-23) |

**Note the notation clash.** Tromp–Farnebäck's `N(n)` counts *games*. This
project writes `H(n)` for the history-congruence index (C-15) to keep the two
apart.

## Reusable tooling

- **Tromp's 2×2 solver** and the exact count 386,356,909,593 under PSK (C-9) —
  a fully enumerated history space on the smallest nontrivial board, and this
  project's primary definitional validation.
- **Weninger–Hayward, Positional Linear Go** (ACG 2017). Solves 1×n under PSK
  to n = 9. Their state is explicitly (player to move, position, set of earlier
  positions, whether the previous move was a pass) — a history-carrying state,
  which is precisely the object this project wants to compress. Public codebase.
- **van der Werf's MIGOS**. Solved 5×5, and all rectangular boards up to 30
  intersections under Chinese rules. Handles the graph-history-interaction
  problem by putting situational properties in the hash; under superko a long
  cycle can block a proof, handled heuristically.

## Confirmed absences

Things that appear not to exist. Each is a reason this project has room to
work, and each is the kind of assertion that decays — all were true of a
2026-09-09 literature search and should be re-checked before anything is
written up.

- **No complexity treatment isolates a repetition rule** as the driver of a
  class change for chess, shogi or xiangqi (C-25), though all three have
  history-dependent legality or outcome. The standard EXPTIME-completeness
  proofs handle repetition with a turn or move counter instead. Robson's
  no-repeat meta-theorem (C-5) is the one rigorous result tying a no-repeat
  rule to a complexity jump, which is why it is the tool one would try to
  adapt.
- **No prior formalization of a history congruence** for games with
  history-dependent legality. The nearest formal object in the Go literature is
  Tromp–Farnebäck's border-state automaton, which is a Myhill–Nerode-style
  compression of the *static legality predicate*, not of outcomes over
  histories. The `H(n)` index and the fooling-set direction (C-15) appear to be
  new.
- **No dedicated literature on "trail games" or "self-avoiding walk games"**
  beyond the Geography family, under those names.
- **No published PSK/SSK complexity separation or equivalence** (C-12). Both
  are lumped together as "superko".
- **No paper improves either bound** for PSK or SSK Go.

## Where the sources are thin

Flagged because building on them without checking would be a mistake:

- **Robson's 1981/1982 ANU technical reports are not available online.** Some
  structural details of the no-repeat reduction — including whether it strictly
  requires a pass move — are reconstructed from secondary sources.
  [`../references/README.md`](../references/README.md) tracks acquisition.
- **"Chinese rules" and "superko" are not synonyms**, and published results
  usually say "superko" without saying which. Every citation must be read for
  which variant it actually covers.
- **The 1×9 minimax score under PSK is reported inconsistently** — 0 in one
  source, 4 in an earlier draft (C-11). Unresolved, and settling it is an early
  target.
