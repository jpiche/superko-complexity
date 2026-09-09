# References

The bibliography, and the acquisition status of each primary source.

The second column is the point. This literature has a documented problem:
Robson's 1981/1982 ANU technical reports are not available online, and some
structural details of the no-repeat reduction are reconstructed from secondary
sources. Building on a reconstruction without recording that it is one is how
an error propagates.

| Status | Meaning |
|---|---|
| `held` | full text in hand and read |
| `partial` | abstract, excerpts, or a secondary account only |
| `sought` | identified, not yet obtained; the attempt is recorded |
| `absent` | confirmed not retrievable by the means tried |

## Primary sources

| Work | Status | Note |
|---|---|---|
| Lichtenstein & Sipser, *GO Is Polynomial-Space Hard*, JACM 27(2), 1980 | sought | the PSPACE-hardness reduction (C-2); ko-independent |
| Robson, *The Complexity of Go*, IFIP 1983 | sought | EXPTIME-completeness under Japanese rules (C-4); reprints reportedly available from the author |
| Robson, *Combinatorial games with exponential space complete decision problems*, MFCS 1984, LNCS 176 | sought | the no-repeat meta-theorem (C-5) — the tool this project would most like to adapt |
| Robson, ANU TR-CS-81-02 and TR-CS-82-02 | absent | groundwork for the above; not retrievable online |
| Tromp & Farnebäck, *Combinatorics of Go*, CG 2006 (rev. 2016) | sought | Lemma 2, the PSK/SSK structural distinction (C-7, C-8) |
| Walraet & Tromp, *A Googolplex of Go Games*, CG 2016 | sought | the Gray-code construction (C-10); preprint public |
| Saffidine, Teytaud & Yen, *Go Complexities*, ACG 2015 | sought | the survey; states the EXPSPACE bound as folklore (C-3); open draft on HAL |
| Demaine & Hearn, *Playing Games with Algorithms*, arXiv:cs/0106019 | sought | the UVG observation (C-6) |
| Weninger & Hayward, *Exploring Positional Linear Go*, ACG 2017 | sought | 1×n solver and score table (C-11); public codebase |
| van der Werf et al., *Solving Go on Small Boards*, ICGA 2003; *…for Rectangular Boards*, ICGA 2009 | sought | MIGOS; GHI handling under superko |
| Chung, *Complexity of Unbounded Boolean Formula Games*, MIT M.Sc., Feb 2026 | sought | names superko formula games as future work |
| Zhang, *A Note on Computational Complexity of Kill-all Go*, arXiv:1911.11405 | sought | poses this project's question for a Go variant |
| Fraenkel, Scheinerman & Ullman, undirected vertex geography in polynomial time, 1993 | sought | C-21 — what the folklore EXPTIME argument rests on |
| *On the Computational Complexities of Various Geography Variants*, arXiv:2108.09367 | sought | C-22; resolves several cases |
| *Undirected edge geography games on grids*, arXiv:2504.12148 | sought | C-22, grid case |
| Crasmaru, master's thesis (ladders in place of Robson's pipes) | absent | described by the author on the computer-go list; thesis not located |
| Crasmaru & Tromp, *Ladders are PSPACE-complete*, CG 2000 | sought | follows Lichtenstein–Sipser, avoids pipes and crossovers |
| Wolfe, *Go endgames are PSPACE-hard*, More Games of No Chance, 2002 | sought | sums of small endgames |
| Crasmaru, *On the complexity of Tsume-Go*, CG 1998 | sought | a restricted life-and-death class is NP-complete |

## Open-problem lists and folklore

Where the problem is stated as open, and where the folklore is recorded. These
matter for establishing that the question really is open, and for dating that
claim.

| Source | Status | Note |
|---|---|---|
| Demaine & Hearn, *Playing Games with Algorithms* / *Algorithmic Combinatorial Game Theory* | sought | the canonical statement of the open problem |
| Eppstein, *Computational Complexity of Games and Puzzles* (ics.uci.edu) | sought | lists Go; notes the Chinese/US-rules case as apparently still open |
| Combinatorial Game Theory blog, January 2011 | sought | Hearn's record of Robson's reported belief that superko Go is in EXPTIME — reported opinion, not a proof |
| Wikipedia *Go and mathematics* / HandWiki | sought | states the PSPACE-hard / EXPSPACE bracket as an open problem |
| computer-go mailing list archive | sought | where much of the folklore is recorded, including Crasmaru's own account of his thesis |
| Sensei's Library, *Robson's Proof that GO is EXP-time Hard* | sought | an improved exposition of the 1983 proof |

## People

Who has published on the material this project uses. The list exists for two
narrow purposes: to know whose future publications are worth watching, and to
know who might — if a result here ever warrants it — be interested to hear
about it.

**No one named here is involved in this project, has reviewed it, or is
answerable for any part of it.** Each entry describes published work and
nothing more. This section asserts nothing about what anyone is currently
doing, intends to do, or would want. Where a source records someone's stated
view, the claim is attributed in the ledger and marked as reported belief
rather than as fact — C-14 is the example.

- **Erik Demaine** and the MIT Algorithmic Lower Bounds group — algorithmic
  lower bounds for combinatorial games. Chung's thesis, above, names superko
  formula games as future work.
- **Robert Hearn** — co-author of the survey that states the open problem.
- **John Tromp** — counting, rules, small-board solving.
- **Abdallah Saffidine**, **Olivier Teytaud**, **Shi-Jim Yen** — the *Go
  Complexities* survey.
- **Ryan Hayward**, **Noah Weninger** — Positional Linear Go.
- **Erik van der Werf** — MIGOS.
- **Marcel Crasmaru**, **David Wolfe** — restricted Go.
- **J. M. Robson** — originated the EXPTIME/EXPSPACE line.

## Rules sources

Needed for the OPEN items in [`../docs/formal-model.md`](../docs/formal-model.md),
which are rules questions rather than mathematical ones.

| Work | Status | Note |
|---|---|---|
| AGA rules, official text | sought | Rule 6 (situational superko) and Rule 7 (pass stones) — C-18, C-19 |
| Tromp–Taylor rules | sought | the phrasing that exempts passes from repetition (C-18) |
| Chinese rules, official text | sought | for the PSK/SSK comparison |

## Convention

A `partial` or `absent` source may be cited, and the ledger row must say which.
A claim resting on a reconstruction is not `cited`.

## Provenance

The author has worked on Go rules and scoring elsewhere, in private code. That
experience informs this project, and nothing is carried over from it: the
definitions here are written fresh against the primary sources listed above.

Should any of that prior work turn out to be worth reusing, it is imported by
an explicit decision recorded here, with its licensing settled first — not by
quietly reproducing it. The same applies to any third-party source: this
repository is Apache-2.0 and everything in it must be distributable under
that.
