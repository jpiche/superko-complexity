# The grounding for complexity-theoretic claims

Decided 2026-09-11. This plan records what the project uses as the foundation
for every resource-bounded claim — over which definitions "SUPERKO-GO is in
EXPSPACE" is stated, which part of such a claim is machine-checked, which part
is prose, and what the prose cites. The evidence is summarized here at the
strength it supports; the full survey, its method and its dead ends are in
[`../../notebook/2026-09-11-complexity-grounding.md`](../../notebook/2026-09-11-complexity-grounding.md).

No effort or elapsed-time estimate appears in this document. The maintainer
ruled them out as measures on 2026-09-10, and the comparison is made on
trusted-base cost, fidelity to the literature's classes, concrete proof
obligations, reuse, and provenance — in that order, which is the order
[`../trusted-base.md`](../trusted-base.md) implies.

## Decision

**Prose classes over a held, in-field referent. The mathematics of the archive
argument in Lean, over `Defs.lean` and Mathlib alone. No complexity library in
`lean/`. One prose sentence, named as its own claim and marked `folklore`.**

1. **Referent.** PSPACE, EXPTIME, EXPSPACE = ⋃ₖ SPACE(2^(n^k)), polynomial-time
   many-one reducibility and X-hardness are cited to Hearn, *Games, Puzzles,
   and Computation*, MIT PhD thesis, 2006, Appendix A (A.1 for the machine,
   TIME and SPACE, P, PSPACE, EXPTIME, reducibility and hardness; A.4 for
   EXPSPACE), and each definition is written out in this project's own prose
   with the citation attached. Hearn's appendix transcribes Sipser's second
   edition, is freely available, and is by a coauthor of the survey that
   states this project's open problem, so a referee recognizes the sentence
   and can check the definition. Sipser, *Introduction to the Theory of
   Computation*, becomes the second, independent referent once a named edition
   is held — the independent-agreement defense of `trusted-base.md` applied
   to definitions rather than to numbers. Until then no Sipser theorem number
   is cited: every one the survey encountered was traceable only through
   lecture notes. Arora–Barak is not cited for EXPSPACE (the free 2007 draft
   defines no such class); Papadimitriou is not cited for Go.

   Two things Hearn asserts without citation come along with the referent and
   are marked as such wherever the prose leans on them: the equivalence of
   multitape and single-tape space classes at these bounds, and
   AP = PSPACE, APSPACE = EXPTIME.

2. **Lean.** The novel content of C-3 lands on the formalized side as derived
   notions, with `Defs.lean` unchanged: a fuel-indexed archive decider proved
   to decide `Superko.BlackWins` (C-29); a string encoding with an exact
   round trip and both length bounds, of which the lower bound
   `2·m·n ≤ |enc|` is what membership consumes (C-31); the fact that the
   verdict sees komi only through its floor, so every komi is equivalent to a
   half-integer (C-27); and determinacy (C-28). The run-level configuration
   bound (C-30) is stated and left `open`.

3. **No library.** No Lake dependency enters `lean/`; no Mathlib re-pin; no
   toolchain change; no new axiom; no `native_decide`.
   `SamuelSchlesinger/complexitylib` is retained as the subject of a
   pre-registered, non-load-bearing probe in a side workspace
   ([`../../experiments/003-complexitylib-window-probe/`](../../experiments/003-complexitylib-window-probe/)),
   never as a dependency of a headline theorem.

4. **The one prose sentence (C-32).** A deterministic Turing machine that
   decodes the input, iterates the decider's step from the initial
   configuration and reads off the verdict uses work space polynomial in the
   configuration size, uniformly in the board dimensions; with C-30 and C-31
   that places SUPERKO-GO in SPACE(2^O(|w|)) ⊆ EXPSPACE. It is its own ledger
   row, `folklore`, written out in [`../../proofs/C-32.md`](../../proofs/C-32.md),
   and its first line says it is not machine-checked and not proved by any
   source. C-3 stays `folklore`. It does not become `cited`, and no prose here
   may say it did.

5. **The language.** SUPERKO-GO is the language of
   [`../formal-model.md`](../formal-model.md) §7: an instance carries the
   dimensions, the position, the komi and the color to move, and the question
   is whether Black has a winning strategy. In Lean that is
   `Superko.Enc.goLang`, built over `BlackWinsFrom m n b komi c`, whose
   Black-to-move slice is `Superko.BlackWins` definitionally. Under this
   choice a hardness reduction whose instances have White to move — as
   Lichtenstein and Sipser's do — lands in the language with no complement
   step and no determinacy step. Komi is part of the input and is encoded
   exactly; OPEN-3 is resolved by that decision, which C-27 licenses.

## What the decision rests on

Facts about repositories and scratch artifacts below are `computed` at the
revisions named; the artifacts themselves were compiled from this project's
`lean/` at its own pins (Lean `v4.34.0-rc2`, Mathlib
`f5e908760f367cc0ad66f64fb3b3a689965bc7a1`) on 2026-09-11 and every theorem
reported a subset of `[propext, Classical.choice, Quot.sound]`.

### The novel content is provable without any library

The load-bearing question of experiment 001 — whether a decider for
SUPERKO-GO could be built and bounded — turned out to have a wrong premise.
Nothing about the game was computable at 2026-09-09: `Situation.after` goes
through the classical `resolve`, `State.seen` is a `Set`, and there was no
move enumeration. The survey built the computable layer as derived notions
and proved its correctness against `WinsFor` by fuel induction, with the fuel
exactly C-26's bound. The only obstruction met was Lean core's own: `Rat.sub`
is `@[irreducible]` at this toolchain, so a decider whose leaf computes
`Defs.lean`'s rational `margin` cannot reduce in the kernel at any komi, and
the leaf is an integer comparison at `⌊komi⌋` with a proved bridge to
`winner`. The decider reduces by `decide` on small boards — single queries
on 1×3, 1×4 and the empty 2×2 finish, an empty 2×3 did not, and the reach
depends on the komi as well as the board — and it is not a search engine: a
winner is not a game count, so no run of it bears on the 2×2 count.

The encoding was built the same way. Its lower length bound is not
bookkeeping: under a stone-list encoding the empty 4×4 board takes 24 bits
while there are 2·3^16 situations, so the archive does not fit in `2^|w|` and
the space budget of the archive argument fails at that instance
(`sparse_encoding_refuted`, by `decide`). The committed sentence that the
archive argument is insensitive to the input encoding was wrong in the half
that matters and is corrected in the ledger.

### Every library's bridge to the textbook class is asserted, not proved

Eleven candidate groundings were examined from primary sources and each
fact sheet was adversarially re-derived twice; two candidates the survey
missed were found by a completeness critic and examined afterwards. For every
Lean library, and for the Coq and Isabelle precedents, the identification of
the library's class with the textbook's is asserted in a docstring, cited to
a paper about a different model of computation, or deliberately not claimed:

| Candidate | Revision examined | Model | What it supplies | Why it is not the grounding |
|---|---|---|---|---|
| `SamuelSchlesinger/complexitylib` | `dev` 6c248df | multi-tape TM, string model | `DSPACE`, `PSPACE`, `EXP`, Karp reductions, Savitch, PSPACE ⊆ EXP, a generic window calculus that is axiom-clean at this project's Mathlib pin in a re-pinned copy | no EXPSPACE, no PSPACE-complete source, no closure of PSPACE under reductions; the space convention (head positions, input region free) cites no referent; the landing gear is in an `Internal` module; 87 files marked unreviewed; one author for 829 of 913 commits. Adoption puts `Cfg.WithinDecisionSpace`, `DecidesInSpace`, `DSPACE` and `BigO` into the audit |
| `PierreSenellart/descriptive-complexity` | `master` de212562 | SO(TC), SO(PFP) over finite structures | PSPACE, EXPTIME, EXPSPACE as logics; `QSAT_PSPACE_complete`; alternating machines; a DOI | its own docstrings say SO(PFP) = EXPSPACE is a definition with no capture theorem claimed; hardness is cofinal hardness under first-order reductions; a Go claim needs a second, structure-level definition of Go with an isomorphism-invariance proof; Lean `v4.33.0`, a downgrade |
| `leanprover/cslib` | `main` ec768ef | multi-tape TM with a cells-visited space measure | the best-documented machine header in the set; its multi-tape layer compiles unchanged at this project's Mathlib pin | no class and no reduction on `main`; the class draft is stale against a 2026-09-08 refactor; the composition line closed unmerged; no area maintainer covers complexity. The first re-check trigger, and a second attachment point for experiment 003 |
| Lax `lax-434930` (`EdouardBonnet/classical-complexity`) | 026a662 | three unbridged TM models over strings | PSPACE, NPSPACE, EXPTIME as definitions | the layer a consumer imports states its claims as Lean `axiom`s; drafts, not citable; Lean `v4.30.0` |
| `Shreyas4991/Algolean` | f64556d | single-tape TM with a time-and-writes cost pair | the strongest provenance in the set (six authors, CI, Reservoir) | cannot state membership of a language in a class at all: its class predicate has no input-length parameter and its cost bound is quantified so that an oracle-free problem is trivially in every class |
| `zksecurity/caliper` | b62f7c8 | imperative language with a peak-live-memory measure | a worked precedent for the *form* of a run-level space measure | no class, no string language, no machine, and no license file, so nothing may be copied |
| `DominikPeters/HardnessReductionsLean` | 280c51a | a combinator language with a blanket cost theorem | the round-trip discipline for encodings, which C-31 imitates | no license; no class; its polynomial-time claim is, by its own README, a design judgment deferred to a future machine model |
| `vihdzp/combinatorial-games` | a087fed | loopy games as a final coalgebra | nothing for complexity | declined for determinacy because the direct proof exists, is axiom-clean and mentions no library notion, and because its `master` renamed the measured API while moving to a Mathlib this project does not have |
| A bespoke layer over Mathlib's `Turing.TM2`/`TM0` | Mathlib f5e9087 | statements only | a draft layer typechecked | Mathlib has no space measure on any machine; scanned cells are not a function of a configuration; `TM2ComputableInPolyTime` has zero theorems and its composition is a `proof_wanted`. Stating is what experiment 001 already declined |
| An explicit-cost-model grounding alone | — | a size measure on Lean configurations | adopted as the *shape* of the Lean content | cannot define a class over Lean functions (a shallow model puts every language in the class, with `Classical.choice`); the class sentence is prose by necessity |
| A hypothesis-carrying (conditional) theorem | — | the unproved step as a named hypothesis | visibility to `#print axioms` | in the shallow form the hypothesis is vacuous or false; in the deep form it is the machine route with the machine's cost assumed instead of proved. Its naming discipline is adopted for C-32 and inside experiment 003 |
| Prose only, nothing new in Lean (the decision as previously recorded) | — | — | — | leaves the algorithm's correctness and the size arithmetic — the two things a kernel can check, and both now checked — trusted to the author |

Non-Lean precedents point the same way. Gäher and Kunze's Coq Cook–Levin
cites Forster, Kunze and Roth for the λ-calculus-to-machine transfer rather
than mechanizing it; Balbach's Isabelle Cook–Levin defines classes over its
own machine and proves no bridge; no proof assistant contains a space class
above logarithmic space in a string model with a PSPACE-complete problem
under polynomial-time reductions, and none contains any geography variant.
The negatives carry the survey's search scope, recorded in the notebook.

### The literature, read rather than recalled

Lichtenstein and Sipser 1980 was obtained and read in full during the survey.
Their hardness result is for a reduced ruleset — a capture order that removes
the mover's own surrounded groups after the opponent's, territory scoring
with judged dead stones, no komi, no tie rule, ko omitted — with White to move
in the constructed position, and they never name the reduction resource; the
notion they import for "Pspace-complete" is logspace reducibility via Meyer
and Stockmeyer 1973, of which polynomial-time many-one reducibility is the
sound weakening. The transfer of their construction to SUPERKO-GO as this
project defines it is therefore a claim with no source (C-33), not a
parenthetical on C-2. Saffidine, Teytaud and Yen 2015 label the EXPSPACE
bound a folklore result and word their superko positionally. Hearn 2006 cites
Robson 1984 for the membership and rejects history-as-position because
complexity results would then be taken relative to an exponentially larger
input. Robson 1984 and 1985 could not be obtained from this host; the 1985
abstract states the machine-level result as a case split whose branch for
situational superko is unidentified.

## What lands in Lean, what stays prose, and what is cited for each prose step

**Lean, over `Defs.lean` and Mathlib f5e9087 only.** The computable game layer
in [`../../lean/SuperkoComplexity/Decide.lean`](../../lean/SuperkoComplexity/Decide.lean);
the encoding in [`../../lean/SuperkoComplexity/Encoding.lean`](../../lean/SuperkoComplexity/Encoding.lean);
C-27, C-28 and C-29 under [`../../lean/SuperkoComplexity/Results/`](../../lean/SuperkoComplexity/Results/).
No class definition, no machine model, no `PSPACEHard`.

**Prose, and the citation each step carries.**

| Prose step | Status | Cited to |
|---|---|---|
| The class definitions and polynomial-time many-one reducibility | `cited` | Hearn 2006, App. A.1 and A.4; Sipser once a named edition is held |
| Multitape and single-tape space classes coincide at these bounds | `folklore` | Hearn asserts it uncited; avoided by stating bounds for multitape machines |
| C-32: the decider's step is Turing-implementable in space polynomial in the configuration size | `folklore` | Dershowitz and Falkovich-Derzhavetz 2015, Theorem 2, for algorithm to arithmetic RAM; RAM space to Turing-machine space covered by no source read; Saffidine–Teytaud–Yen 2015, Theorem 1, is this sentence and calls itself folklore |
| C-2: Lichtenstein and Sipser's game is PSPACE-hard under ≤ₚ | `cited` | Lichtenstein–Sipser 1980, with ≤log imported via Meyer–Stockmeyer 1973 and weakened to ≤ₚ |
| Transfer of that construction to SUPERKO-GO as stated in §7 | `open` (C-33) | no source |
| Encoding (C) is the literature's convention; `enc` is its encoding up to polynomial recoding | `cited` for the convention, asserted for the recoding; C-1 stays `open` | Lichtenstein–Sipser; Stockmeyer–Chandra 1979 (abstract only); Saffidine–Teytaud–Yen; Hearn 2006 |
| Polynomial-time computability of any reduction this project builds | `folklore` | settled by inspection throughout the literature |

**Trusted-base delta.** Item 3 of `trusted-base.md` is unchanged. Item 4 grows
from seven marked bridge statements to seventeen — seven in `Decide.lean`
tying the computable game layer to the core, three in `Encoding.lean` — plus
one statement of a new kind, the decider's correctness theorem, whose
right-hand side is the already-audited `BlackWins`; a wrong twin among the
bridges would make that root unprovable rather than misleading. The marking
convention is the repository's existing one, applied uniformly: every
statement tying a twin to a `Defs.lean` notion is marked. Eighteen items
against a threshold of twenty; growth from here is a decision, not
absorption.

## Effect on the ledger and the experiments

- New rows: C-27, C-28, C-29, C-31 `proved`; C-30, C-33, C-35 `open`; C-32
  `folklore`.
- C-3 stays `folklore` and now depends on C-29, C-30, C-31 and C-32; its
  detail names C-32 as the residue and corrects the encoding sentence.
- C-2 is narrowed to the Lichtenstein–Sipser ruleset under ≤ₚ; the
  parenthetical moves to C-33.
- C-20 is re-dated and narrowed: what no library supplies is a PSPACE-complete
  source problem under polynomial-time many-one reductions over a
  string-encoded machine class; the machine half of the old sentence was too
  strong.
- C-1, C-5, C-14 and C-21 gain detail; C-6 gains its dependency on C-21.
- Experiment 002 is retired unrun with its premise defect recorded;
  experiment 003 is pre-registered with threshold-free falsifiers.

## What would overturn this

Each is checkable, and none is a duration.

1. **Robson 1985 or 1984, read in full, covers situational superko on the
   time-exponential branch.** C-3 becomes `cited`; the Lean rows become
   corroboration; the decision stands.
2. **A Lean library on this toolchain gains a string-model PSPACE-complete
   source under polynomial-time many-one reductions together with proven,
   public machine-composition combinators.** A hardness statement could then
   be formalized end to end. cslib's class draft rebased and merged with a
   composition API is the specific form.
3. **Experiment 003's first outcome fires.** A machine-checked membership in a
   disclosed side base, with C-32's residue reduced to a textbook arithmetic
   sentence, is the trigger to amend the boundary section again.
4. **C-30 cannot be proved as stated.** A finding about the decider's design,
   not the grounding; the space account falls back to the per-frame bound of
   the big-step decider, still inside EXPSPACE.
5. **C-9 fails to reproduce.** Every row above is about `Defs.lean` rather
   than about Go. Stop before anything else.
6. **The maintainer weights the boundary rule above the trusted-base count
   and the pin discipline.** The strongest dissent, preserved in the notebook,
   reads C-32 as the arrangement the boundary rule forbids and would take the
   complexitylib side base now. That is a legitimate reading; this document
   treats C-32 as the rule's own named exception.

## Re-check triggers

- C-20's expiry: re-run the audit against fresh revisions of complexitylib,
  cslib, descriptive-complexity, the Lax records, Algolean and Caliper before
  leaning on the row. Every one of them moved within the survey's own window.
- complexitylib defines EXPSPACE, moves its window calculus out of `Internal`,
  or cites a referent for its space convention.
- A Mathlib bump: re-verify experiment 003's starting point, and note that
  the per-instance floor in C-29's spot checks relies on Mathlib's `norm_num`
  extension for `Int.floor`.
- A Lean core release that un-marks `Rat.sub` as irreducible: the integer
  leaf is a workaround tied to this toolchain, documented as such.
- Sipser held, edition named: add the second referent and check every number
  before it enters the ledger.
- Slot and van Emde Boas, or van Emde Boas 1990, obtained: if the RAM-space
  to machine-space step is proved there, C-32's residue narrows to the
  instantiation alone.
- The bridge count: any growth beyond the policy recorded in the notebook is
  a conscious decision, not absorbed.

## How this was produced

A multi-agent survey on 2026-09-10 and 2026-09-11: eleven readers, one per
candidate, each adversarially verified by two independent agents; five
designers proposing a grounding from five angles, including one written to
defend the previous decision; three judges; a synthesis; a completeness critic
whose four follow-up briefs were run and verified. Verification, design and
judging ran on Claude Opus 5; the syntheses ran on Claude Fable 5.1; the
maintainer's session re-compiled every Lean artifact this document relies on
and read the headline statements. The judgments are inputs, not evidence: the
facts above were re-derived or are marked at the strength of a survey report
in the notebook.
