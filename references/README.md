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
| Lichtenstein & Sipser, *GO Is Polynomial-Space Hard*, JACM 27(2), 1980 | held | the PSPACE-hardness reduction (C-2), for a reduced ruleset with ko omitted; read in full 2026-09-10 (see Convention); the transfer to superko is C-33 |
| Robson, *The Complexity of Go*, IFIP 1983 | sought | EXPTIME-completeness under Japanese rules (C-4); borrowable on archive.org (lending item, account required); reprints reportedly available from the author |
| Robson, *Combinatorial games with exponential space complete decision problems*, MFCS 1984, LNCS 176 | sought | the no-repeat meta-theorem (C-5) — the tool this project would most like to adapt; Hearn cites it for the EXPSPACE membership (C-3); Springer serves only a challenge page from this host |
| Robson, *Alternation with restrictions on looping*, Information and Control 67, 1985 | partial | abstract held (via CORE and zbMATH): no-repeat alternating machines model games such as "the Chinese version of Go", and restricted space is equivalent either to time or to space exponential in it, depending on the restriction; indexed as open access but refused from this host; the likely journal form of the 1984 paper |
| Robson, ANU TR-CS-81-02 and TR-CS-82-02 | absent | groundwork for the above; not retrievable online |
| Tromp & Farnebäck, *Combinatorics of Go*, CG 2006 (rev. 2016) | sought | Lemma 2, the PSK/SSK structural distinction (C-7, C-8) |
| Walraet & Tromp, *A Googolplex of Go Games*, CG 2016 | sought | the Gray-code construction (C-10); preprint public |
| Saffidine, Teytaud & Yen, *Go Complexities*, ACG 2015 | held | the survey; states the EXPSPACE bound as folklore (C-3) and words superko positionally; the HAL author draft, read in full 2026-09-10, not the Springer print |
| Demaine & Hearn, *Playing Games with Algorithms*, arXiv:cs/0106019 | held | the UVG observation (C-6) and the one-sentence transfer claim now recorded under C-33; arXiv v2, read in part 2026-09-10 |
| Weninger & Hayward, *Exploring Positional Linear Go*, ACG 2017 | sought | 1×n solver and score table (C-11); public codebase |
| van der Werf et al., *Solving Go on Small Boards*, ICGA 2003; *…for Rectangular Boards*, ICGA 2009 | sought | MIGOS; GHI handling under superko |
| Chung, *Complexity of Unbounded Boolean Formula Games*, MIT M.Sc., Feb 2026 | held | names superko formula games as future work; read in part 2026-09-10 |
| Zhang, *A Note on Computational Complexity of Kill-all Go*, arXiv:1911.11405 | sought | poses this project's question for a Go variant |
| Fraenkel, Scheinerman & Ullman, undirected vertex geography in polynomial time, 1993 | sought | C-21 — what the folklore EXPTIME argument rests on |
| *On the Computational Complexities of Various Geography Variants*, arXiv:2108.09367 | sought | C-22; resolves several cases |
| *Undirected edge geography games on grids*, arXiv:2504.12148 | sought | C-22, grid case |
| Crasmaru, master's thesis (ladders in place of Robson's pipes) | partial | a self-described incomplete draft (missing the introduction and one chapter) obtained from the author's public drive 2026-09-10; gives an explicit 2n²-bit position encoding; not vendored — the draft reserves reproduction rights |
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
| Combinatorial Game Theory blog, January 2011 | sought | Hearn's record of Robson's reported belief that superko Go is in EXPTIME — reported opinion, second-hand, in the same paragraph as an opposite speculation |
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

## Textbooks and definitional referents

The grounding decision ([`../docs/plans/complexity-grounding.md`](../docs/plans/complexity-grounding.md))
cites the complexity classes and the reduction notion in prose to a held text.
These rows are what that citation rests on.

| Work | Status | Note |
|---|---|---|
| Hearn, *Games, Puzzles, and Computation*, MIT PhD thesis, 2006 | held | **the primary definitional referent**: Appendix A.1 (single-tape machine, TIME and SPACE, P, PSPACE, EXPTIME, polynomial-time many-one reducibility, hardness) and A.4 (EXPSPACE = ⋃ₖ SPACE(2^(n^k))), transcribing Sipser's second edition; asserts the multitape equivalence and AP = PSPACE without citation; read in full 2026-09-10 |
| Hearn & Demaine, *Games, Puzzles, and Computation*, A K Peters 2009 | sought | the book form of the thesis; not obtained |
| Sipser, *Introduction to the Theory of Computation* (edition to be named) | sought | the second, independent referent once held; every Sipser definition and theorem number encountered so far is traceable only through lecture notes and is not cited |
| Arora & Barak, *Computational Complexity: A Modern Approach*, 2009; 2007 web draft | partial | the free 2007 draft was read in part; it defines no EXPSPACE class and is not cited for it; the published edition's numbering is unverified |
| Papadimitriou, *Computational Complexity*, 1994 | sought | not cited for Go: Hearn reports its Go treatment is of a bounded modification |
| Meyer & Stockmeyer, *Word problems requiring exponential time*, STOC 1973 | held | the logspace-reducibility notion Lichtenstein and Sipser import for "Pspace-complete", and its remark that it implies polynomial-time reducibility; read 2026-09-11 |
| Stockmeyer & Chandra, *Provably difficult combinatorial games*, SIAM J. Comput. 1979 | partial | abstract only: exponential-time completeness under logspace reductions, input the starting position; which exponential class is meant is not determinable from the abstract |
| Chandra, Kozen & Stockmeyer, *Alternation*, JACM 1981 | sought | APSPACE = EXPTIME, the equivalence C-14's alternating restatement rests on; held here only through secondary restatements |
| Dershowitz & Falkovich-Derzhavetz, *A formalization and proof of the extended Church–Turing thesis*, LMCS 2015 | held | Theorem 2, the space invariance theorem: a sequential algorithm runs on an arithmetic RAM in space linear in its configuration size; the citation for the algorithm-to-RAM half of C-32; read 2026-09-10 |
| Cook & Reckhow, *Time bounded random access machines*, JCSS 1973 | held | Theorem 2 is time-only; the word "space" does not occur; not a citation for the RAM-to-machine space step; read 2026-09-10 |
| Slot & van Emde Boas, *On tape versus core*, STOC 1984 / Inf. Comput. 1988; van Emde Boas, *Machine models and simulations*, Handbook 1990 | sought | the invariance thesis as a published statement about space; unread, so the RAM-space to machine-space step of C-32 is covered by no source read |
| Forster, Kunze & Roth, *The weak call-by-value λ-calculus is reasonable for both time and space*, POPL 2020 | held | the one mechanized invariance result, for a λ-calculus; cited as precedent, not as a theorem about this project's functions; read 2026-09-10 |

## Rules sources

Needed for the OPEN items in [`../docs/formal-model.md`](../docs/formal-model.md),
which are rules questions rather than mathematical ones.

| Work | Status | Note |
|---|---|---|
| AGA rules, official text | held | `data/AGA_Rules_of_Go.pdf` (gitignored), the Rules Committee text dated 1991-09-01 as amended — its Rule 3 carries the 7½ komi of 2004; read 2026-09-11. Rule 2 and 6 settle C-18, Rule 12 settles C-19, Rules 9 and 10 are the mechanism C-16 argues from |
| Tromp–Taylor rules | held | <https://tromp.github.io/go.html>, read 2026-09-11, text saved as `data/tromp-taylor.txt`; Rule 6 exempts passes and is positional superko by its own Comment 6; Rule 5 starts from the empty grid, so the initial coloring is an earlier one |
| Chinese rules, official text | sought | for the PSK/SSK comparison |

## Convention

A `partial` or `absent` source may be cited, and the ledger row must say which.
A claim resting on a reconstruction is not `cited`.

A `held` source obtained during a model-assisted session was read by the
models named in the corresponding notebook entry, under the author's
accountability, before the author read it; the note says when it was read.
The copies obtained on 2026-09-10 sit under `data/grounding/scratch/`, which
is gitignored, and nothing is vendored into this repository without its
license being settled first.

## Provenance

The author has worked on Go rules and scoring elsewhere, in private code. That
experience informs this project, and nothing is carried over from it: the
definitions here are written fresh against the primary sources listed above.

Should any of that prior work turn out to be worth reusing, it is imported by
an explicit decision recorded here, with its licensing settled first — not by
quietly reproducing it. The same applies to any third-party source: this
repository is Apache-2.0 and everything in it must be distributable under
that.
