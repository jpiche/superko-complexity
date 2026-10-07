# The OpenAI mathematics release, read as a whole

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`), Claude Opus 5.5 (`claude-opus-5-5`), Claude Sonnet 5.5 (`claude-sonnet-5-5`). Fable orchestrated, read the Lean and deep-dive reports, and wrote this entry. Opus did the reading: 130 skim readers over all 372 families, 6 surveyors and 4 deep readers over the Lean library, 8 readers of the proof-discovery documents, 17 subject synthesizers, and 4 full-paper readers with 8 adversarial refuters for the companion entry. Sonnet read the two discovery documents Opus's safety classifier refused, as mathematics-only outlines, and one more as a fallback.

A survey of `~/code/external/math`, OpenAI's October 2026 release of 722
manuscripts in 372 result families, produced by an unreleased model. This
entry says what the release contains, how its proofs go, what its Lean side
is, and what a reader should check first. The companion entry,
[`2026-10-07-openai-math-trove-applicable.md`](2026-10-07-openai-math-trove-applicable.md),
says what bears on this project. The working records are under
`data/singularity/` (gitignored); its `README.md` indexes them.

**Status convention for this entry.** Every mathematical statement attributed
to the release is a *claim* by an unreviewed source. Nothing below is `proved`,
`cited` or `computed` in this project's sense. Where the release supplies a
Lean statement, what exists is a machine-checkable *statement* whose
faithfulness to the paper nobody has audited and whose proof this project did
not build. The release's own `formalization.yaml` records `review: unchecked`.

## How it was read, and how deep

- Every family's catalog summary, and every paper's abstract, section list
  and front matter through its second section heading, were read by an Opus
  reader scoring two things: relevance to this project on a 0–3 rubric, and
  general interest. Readers could open one further section when the method
  was unclear; 82 of 372 did.
- Seventeen Opus readers then synthesized the records of one subject each,
  instructed to discount the skim readers' generosity.
- Ten Lean readers surveyed the library's `Computability` and `GameTheory`
  subtrees and its repository hygiene, four of them reading specific files in
  full.
- The ten "reasoning summaries" OpenAI released were read in full.
- Four papers were read in full for the companion entry.

What was not done: no Lean was built and no `#print axioms` was run; no paper
outside those four was read past its introduction except by section; no
mathematics was checked. This is a map, not a review.

## The release in numbers

| Quantity | Value |
|---|---|
| Manuscripts / families | 722 / 372, in 17 subjects |
| Dating | all but 7 papers dated 2026-09-22 to 2026-10-06; 370 on September 23–24 alone, 112 on October 5 |
| Length | median 16,085 words per paper; 83 papers under 6,000 words, 49 over 50,000 |
| Families with a Lean document | 235 of 372 (63 %); 405 comparator challenge statements |
| Lean library | 121,734 files, 1.7 GB; zero `sorry`, zero `axiom`, zero `native_decide`, zero `set_option`; 1.35 M uses of `decide +kernel` |
| Toolchain | Lean v4.34.1, Mathlib `d13f23b` (this project: v4.34.0-rc2, `f5e9087`) |
| Families the skim readers tied to a named open problem | 369 of 372 |
| Counterexample claims, per the subject syntheses | 92 entries |
| Results resting on computer certificates outside Lean | 49 entries |
| Substantive red flags, per the subject syntheses | 152, of which 31 name a dependency on another unreviewed paper in the same release |

The release's README says the model was posed about 4,000 problems at roughly
three hours of compute each, and that the families are what survived
aggregation at "an appropriate level of significance." The dates say the
writing happened in two bursts a fortnight apart.

## What is claimed

Almost every family claims to settle a named problem, and many of the names
are the best known in their fields. Theoretical computer science alone claims
the Unique Games Conjecture (102), L = RL = BPL (103), ω ≤ 9/4 for matrix
multiplication (107), integer multiplication below n log n (109), an FPRAS for
perfect matchings in general graphs (113), and Subset Sum in O(2^0.49n)
(138). Number theory claims a zero-free half-plane Re s > 7/8 for every
Dirichlet L-function (003), Hilbert's tenth problem over ℚ (004), the
irrationality exponent of π equal to 2 (017), and BSD in Selmer coranks 0 and
1 (002). Combinatorics claims Hadwiger's conjecture false (157), Borsuk false
from dimension 9 (156), the plane not 5-colorable (158), and the Erdős
reciprocal-sum conjecture for all k (159). Group theory claims Thompson's F
nonamenable (248) and Cannon's conjecture (246). Operator algebras claims
L(F₂) ≅ L(F₃) (287). Topology claims Hilbert–Smith (304). Differential
geometry claims the aspherical positive-scalar-curvature conjecture in all
dimensions (335, 336). Mathematical physics claims the Haldane gap (268) and
Polyakov's mass gap (215). Each subject synthesis under
`data/singularity/subjects/` lists its own headline claims.

About a quarter of the entries are counterexamples rather than proofs, and
the subject synthesizers rated several of the explicit ones as the most
checkable items in the release: polynomial counterexamples to cancellation
(047), a 66-edge counterexample to Sidorenko (161), finite-dimensional
counterexamples to Auslander–Reiten (199), a 4,000-word counterexample to
Wall's D(2) problem (321).

Two structural facts matter more than any single claim.

**The claims are not independent.** Thirty-one red flags record a headline
resting on another unreviewed paper in the same release. The sharpest case is
the zero-free half-plane of family 003: family 029 (Artin's primitive root
conjecture), the unconditional half of 142 (polynomial factorization over
F_p) and family 182 cite it, and since the Dedekind zeta function of
ℚ(ζ₁₂) has ζ as a factor, any of those is at least as strong as a
quasi-Riemann hypothesis. The birational-geometry cluster (033, 034, 036,
056, 057, 058) is a chain; the 4-manifold pair 305 and 320 shares two
companion inputs; four operator-algebra headlines (288, 289, 295, 296) rest
on one Popa-freeness estimate; the 3D harmonic-analysis cluster (074, 077,
078, 079) shares one entropy-capacity engine. One gap would take several
headlines with it.

**Priority is sometimes external.** The release itself credits humans or
earlier AI-produced work with the main theorem or an earlier proof in
families 119, 213, 235, 290, 297 and 366, and the model's own prior output
in 132 and 172. "Claims to resolve" and "gives a second proof of" are
different statements, and the catalog does not always distinguish them.

## How the proofs go

The skim readers classed 216 families as "new idea" and 115 as "leap." The
subject synthesizers, told to discount that, converged on a narrower
description that recurs across all seventeen subjects:

> A known framework, plus one new lemma that carries the whole proof, plus a
> long verification.

In 102 the lemma is a "latent alphabet gadget" set on top of the
Barak–Kothari–Steurer test, the Khot–Minzer–Safra inverse theorem and
Dinur–Steurer repetition. In 107 one discrete inequality carries the drop
from ω < 2.371 to 9/4, in a 5,000-word paper that appeared eight days after
a 34,000-word paper reaching 2.258. In 256 one subadditivity estimate in a
group von Neumann algebra carries the Kervaire conjecture. The synthesizers'
phrase for this was that correctness "depends on whether the single new
lemma survives scrutiny," and they say it of most subjects.

Three profiles sit beside that one.

- **Short conceptual papers on old problems.** Eighty-three papers are under
  6,000 words. Several claim decades-old problems through a single identity:
  a skew-form linear-algebra bound for crossing numbers (165), an alternating
  character product for circulant Hadamard matrices (179), a parity
  mechanism for Kaplansky's zero-divisor conjecture (196), affine transport
  of dyadic intervals for Thompson's F (248). The combinatorics synthesizer
  observes that this is "exactly the profile of past false proofs," and that
  179 itself lists five failed predecessors. These are the fastest claims to
  check, and the ones most in need of it.
- **Long quantitative grinds.** Forty-nine papers exceed 50,000 words; 159
  (Erdős–Turán) is 100,000, 143 (Hilbert's 16th, second part) 80,000, 286
  (lattice rigidity) 89,000. Their correctness rests on parameter bookkeeping
  nobody has audited, and the skim readers read none of it.
- **Thin margins.** Several results beat a barrier by an amount that a single
  slipped constant would erase: κ = 2^−182 in 109, exponent 1 − 10^−13 in
  130, 0.49 against 0.5 in 138, polynomial degree 150,020 in 124. A thin
  margin is not evidence of error. It is evidence that the proof is where the
  result is.

Recurring techniques, by subject: PCP gadgets over inverse theorems
(102, 105, 106, 117, 118); annealed Markov chains on enlarged state spaces
with Lorentzian-polynomial mixing (113, 114, 115); finite "rank-deficit" wins
amplified recursively (107, 109, 130); monoid and algebraic encodings of
lower bounds (108, 116, 129); Popa ultrapower freeness with Haagerup's
inequality (288, 289, 295, 296); Fargues–Fontaine methods reused across the
chromatic cluster (311, 313, 318); entropy-capacity contradictions in
harmonic analysis (074, 077, 078, 079); minimal counterexample plus Euler
characteristic (197, 173). The pattern of one engine applied repeatedly
explains much of the volume.

Computer assistance is rarer than the volume suggests: 49 entries across
all subjects, mostly exact-rational or interval certificates for scalar
inequalities checked in Python or C++ rather than Lean (107, 119, 125, 138,
087, 101, 367, 371), and a few kernel-checked certificates (187, 189, 266).
One headline is explicitly a computation: 266's "exactly three mutually
unbiased bases in dimension six" rests on a binary64 run, and its own Lean
document says it does not establish the bound of three.

## The process evidence: ten proof-discovery documents

OpenAI released, for ten results, a summary of how the proof was found. All
ten were read in full. Readers classified eight as idea-driven, one as
mixed, and none as computation-driven; no computer search appears in any of
the ten, and several documents note that the prompt discouraged one. On the
maintainer's hypothesis that the model makes leaps rather than brute-forcing,
this is the evidence, and it points one way. The same documents also say, in
their own hedged wording, that the decisive steps are "asserted, not shown":
π's Liouville stress test is recorded as stalled; the arithmetic-progressions
document leaves a circularity concern unresolved in §26 and presents the
argument anyway in §27; the Vlasov–Maxwell document ends conditionally on
three estimates its cited continuation theorem does not supply.

The habits the documents display are consistent and transferable:

1. Reduce the question to an equivalent structural statement before hunting
   (the Dykema–Rădulescu dichotomy; a continued-fraction criterion for
   Flint–Hills).
2. Catalog every known approach and kill each with one named obstruction,
   often dozens of them in a page ("volume," "odd-prime cost," "equivalent
   to the thing you want, so circular").
3. Keep an exact equality model and an adversarial zoo, and run every
   candidate inequality against it before investing (orthant and simplex
   plus ball, crosspolytope, Lorentz, PSD and Vinberg cones in 087).
4. Ask "does it prove too much?" against a class where the conclusion must
   fail (Liouville numbers; characteristic zero in 197).
5. Write the resource recurrence explicitly and find the compounding term
   (P_{j+1} = P_j + (2+P_j)^E versus d → d + d₀ in 159).
6. Separate budgets: structure used inside an existence proof is not charged
   to the output object ("warm" and "cold" in 159).
7. Prove termination by a weighted potential that strictly drops at each
   failure event.

Two documents could not be summarized at the process level. Both are titled
"Summarized chain of thought" and narrate "the assistant examined … the
assistant next explored"; every reader asked to summarize them, on either
model, was stopped by Anthropic's safety classifier at the output stage with
the label `reasoning_extraction`, although the documents are public PDFs. A
mathematics-only outline of each was obtained instead. The incident is
recorded because it bounds what this review could read, not because it says
anything about the mathematics.

## The Lean side

The library is large, clean by grep, and organized for external checking.
Each of 405 headline statements has a comparator challenge: a statement file
importing only Mathlib and ending in `sorry`, a JSON naming the solution
module and the permitted axioms (always `propext`, `Quot.sound`,
`Classical.choice`), and a flag for a second kernel. Twenty-three third-party
dependencies are patched for the toolchain; Mathlib is not patched, and the
patches add no `sorry` or axiom by grep. Nothing here was built.

Three caveats recur in every subject synthesis. The Lean scope is often
narrower than the headline: only the planar case of Falconer (073), only
finite entropy where zero entropy is claimed (152), only N(6) ≤ 5 where
N(6) = 3 is claimed (266), only the known side of Bernoulli flatness (367),
only an inaccessible-cardinal version of a ZF consistency claim (244). Some
comparator files define key notions themselves (`cohomologicalDimension` in
249, CAT(0) and filling in 257, `FullCPT` in 243), so the theorem means what
the file's definition means. And the largest claims in several subjects have
no Lean at all: 335, 336, 338, 339, 345, 346, 355 in differential geometry;
the 3D harmonic-analysis cluster; Hadwiger (157); Thompson's F has a
statement but 258 and 259 have none.

The complexity-theoretic infrastructure, which is what this project cares
about, is described in the companion entry. In one line: the release has
genuine P, NP and EXPTIME over small paper-local machines and over Mathlib's
`Turing.TM2`, proved composition, one file with a space measure, and no
PSPACE or EXPSPACE anywhere.

## What a reader should check first

The subject synthesizers were asked for the families an expert in the subject would read
first. Their lists favor claims that are short, that rest on one identifiable
step, and that have a Lean statement against Mathlib's own definitions:

- 003, zeta zero-free in Re s > 7/8: stated against Mathlib's `riemannZeta`
  with standard axioms. Building this comparator is the single most
  informative check in the release, and it would also decide 029, 142 and
  182.
- 107, ω ≤ 9/4: a 5,000-word paper whose weight sits on one inequality.
- 248, Thompson's F nonamenable: 4,000 words, Lean-stated, on a problem with
  withdrawn proofs in both directions.
- 321, Wall's D(2): 4,000 words and one explicit presentation.
- 287, L(F₂) ≅ L(F₃): 8,700 words, Lean-stated, against decades of
  expectation.
- 165, crossing numbers: a short skew-form argument against a Lean statement.
- 047, 161, 199: explicit counterexamples whose objects can be examined
  directly.
- 017, μ(π) = 2: a Lean statement of both the exponent and Flint–Hills
  summability, and a discovery document that records its own unresolved
  stress test.

## Verdict on the hypothesis

The maintainer's premise was that this model makes leaps rather than
brute-forcing, and that the approaches may be worth more than the results.
The evidence read here supports the first half with a qualification and the
second half as stated.

The proofs are not brute force. Computer search is rare, the ten discovery
documents show idea-driven attack with systematic dead-end pruning, and the
recurring proof shape is a single new lemma on a known frame. But "leap" is
too generous for most of the release. The synthesizers' sober description is
one new lemma plus a long verification, with the lemma usually sitting
exactly where earlier attempts failed, and with the release's own documents
conceding that the lemma is asserted rather than shown. Whether the leaps are
right is a question this review cannot answer and did not try to.

What is worth more than the results is the discipline around them: the
obstruction catalogs, the adversarial zoo, the prove-too-much test, the
explicit resource recurrence, and a formalization pipeline that confines the
trusted surface to a short statement file and an axiom whitelist. Those
transfer regardless of which headlines survive. The companion entry says
how.
