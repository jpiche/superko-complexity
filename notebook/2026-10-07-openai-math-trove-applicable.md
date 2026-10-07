# What the OpenAI mathematics release offers this project

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`), Claude Opus 5.5 (`claude-opus-5-5`), Claude Sonnet 5.5 (`claude-sonnet-5-5`). Fable orchestrated and wrote this entry. Opus did the reading: the 130-reader skim that scored every family for relevance, six surveyors and four deep readers over the release's Lean library, and four full-paper readers each followed by two adversarial refuters. Sonnet read two proof-discovery documents as outlines. The companion entry [`2026-10-07-openai-math-trove.md`](2026-10-07-openai-math-trove.md) describes the release as a whole and the method of the review.

**Status convention.** No claim in this project changes status because of
anything below. Every statement about the release is a claim by an
unreviewed source; every Lean fact is from reading, not building; every
proposal is `conjecture` until someone writes it in `proofs/` or runs it as an
experiment.

## The short answer

Nothing in the release bears on which complexity class SUPERKO-GO is in. Of
372 families, 308 scored zero for relevance, 39 scored one, and the 25 that
scored two or three did so, with one exception, for Lean infrastructure
rather than mathematics. Four families whose technique looked like it might
transfer were read in full and attacked by two refuters each; three were
refuted on both lenses and survive at most as a sentence here. The fourth,
family 129, was refuted as a transfer too, but reading it produced one
observation about C-15 that belongs to this project and is worth writing up.

What the release does offer is on the Lean side: a correction to a factual
sentence in the grounding plan, a worked statement shape for an EXPTIME claim
and for a hardness reduction, a template for the run-level bound C-30 asks
for, and a certificate discipline that would carry C-56 from `computed` to
`proved` without touching `Defs.lean`. Each is a pattern to copy, not a
dependency to import. The release is on Lean v4.34.1 and Mathlib `d13f23b`;
this project pins v4.34.0-rc2 and `f5e9087`.

## What does not transfer, and why

**Mean-payoff and parity games (104).** The quasipolynomial algorithms beat
an exponential in the numerical width of the weights; front 1's exponential
is the archive, and Go's payoff width is already polynomial. The arena is
explicit, legality is history-free, and the objectives are prefix-independent,
each of which superko violates. A quasipolynomial solver on an explicit
2^poly(mn) arena would run in 2^poly(mn) time, but that is C-47 restated, and
the arena is exactly what C-15 lacks. Two remarks survive at the weakest
honest strength: on the situation graph every cycle is even and who closes it
depends on the entry point, so the "closer loses" payoff is not
rotation-invariant and the first-cycle-game route to positional strategies
(Ehrenfeucht–Mycielski 1979; Aminof–Rubin, cited by the readers, unverified)
cannot be invoked as is; and because a pass is exempt from repetition (C-18)
and archives both of a board's situations (C-50), SUPERKO-GO is not
literally a first-cycle game.

**Quasilinear PCP for PPAD (136).** The "macrograph" makes a long local walk
simple by making the dynamics a partial injection from a predecessor-free
source. Read as Go, that is C-10 in other words. The construction has no
adversary, the situation graph is one strongly connected component on every
censused board (C-45), and the clock ticks the paper leaves uncounted are the
whole cost in Go. A clocked simulation that makes superko vacuous on honest
play is the move-counter technique C-25 records for chess and shogi; it does
not engage the repetition rule.

**Self-simulating hierarchies robust to forged states (247).** The robustness
is a safety invariant over a symmetric rewriting system, proved by reversing
tours. Under superko, returning to a previous board is exactly what is
illegal, captures are one-way, and the deviator is a strategic alternating
player. Survives: one design rule any front-2 gadget must meet, that no gadget
may carry hidden scratch state an adversary can use to make old boards fresh
again, which follows from the definition of superko and needs no paper.

**Everything else.** No family touches positional versus situational superko
(front 3). No family supplies a hard source problem in a string model with
a repetition rule. The release's game-theory content is Maker–Breaker
achievement games, which are monotone; Go is not.

## Family 129 and the index H(n) of C-15

The paper claims exponential state lower bounds for two-way automata by
mapping what a reader retains about a segment into a diagram monoid and
showing that correctness forces a surjection onto the full relation monoid,
whose degree is then bounded by rank-loss amplification. The reader
established two things about carrying this to Go histories.

*The monoid method cannot apply.* Every legal game from a root lasts at most
4·3^(mn) moves (C-26, `proved`). So in the syntactic monoid of legal superko
histories, every nonempty move word u satisfies u^N ≡ 0 for N above that
bound: every non-identity element is nilpotent and every subgroup is trivial.
The paper's amplification needs unit lifts of permutations of a growing set,
and no aperiodic monoid surjects onto one containing a symmetric group. The
argument is two lines from a proved claim; it is not written in `proofs/`,
so its status here is `conjecture` until it is.

*The elementary half of the method may apply.* The paper's step (2) is a
Myhill–Nerode fooling set by singleton contexts. The reader sketched a Go
instance: k + 1 independent simple kos walled in by unconditionally alive
groups, so the board is a bit vector and a capture flips one bit; a backbone
path through 2^Ω(k) disjoint six-cycles in the middle levels of the cube,
each offering two arcs, so 2^(2^Ω(k)) histories end at one situation with
distinct archives; and a guard bit that lets a continuation walk through a
never-visited copy of the cube to a chosen vertex and test, by one capture,
whether that vertex is archived. If the gadget is Go-legal and the
combinatorial lemma holds, a congruence that fixes legality has index at
least 2^(2^Ω(mn)), which refutes the 2^poly(mn) form of C-15 as the ledger
states it. A congruence that fixes only the value is untouched, because the
test capture would also have to be the only winning move. Status:
`conjecture`, a sketch nobody has checked; the suicide rule, the capture
order and the pass semantics all need checking against `Defs.lean`.

Two refuters attacked the proposal, and both refuted the *transfer*: the
fooling-set device is classical, the paper uses it only to make a surjection
well defined and says itself that counting alone gives a weak bound, and
open-questions §4 already names a fooling-set construction as the one thing
that could move C-15. The bead-and-corridor construction uses nothing from
the paper. If it works it is this project's own, and so is its risk: a
gadget legal under `Defs.lean`, a lemma that the middle levels of the
hypercube hold 2^Ω(k) disjoint six-cycles threaded by one path, and the
parity bookkeeping for PSK and SSK are all unproved. Under §4's corrected
threshold, even a proved doubly exponential index kills one approach and
says nothing about EXPTIME or EXPSPACE.

The reader's recommendation, which this entry endorses as a question rather
than a decision: decide whether C-15 is meant to measure a congruence on
legality and value, which the sketch targets, or on value alone, which is
what the alternating reading in open-questions §4 needs and which the sketch
leaves open. The ledger row currently says both.

## The Lean side, claim by claim

**C-20 and the grounding plan.** Two sentences in
`docs/plans/complexity-grounding.md` are overtaken.

- The table row for a bespoke layer over Mathlib's `Turing.TM2` says that
  `TM2ComputableInPolyTime` "has zero theorems and its composition is a
  `proof_wanted`." At this project's pin there is no `proof_wanted` under
  `Mathlib/Computability`; `Computable.lean` has identity witnesses only
  (`idComputableInPolyTime`) and no composition. "Zero theorems" stands;
  "composition is a `proof_wanted`" is not supported.
- The release proves, by reading, what the row said was missing: sequential
  composition of `TM2ComputableInPolyTime` witnesses with an explicit
  polynomial (`MachineSequential.compose`, in `OAI/Computability/BinPacking/
  Computation/MachineFiniteAlphabet.lean` and three other copies), a
  unary-counted loop, and a time-bounded compilation of a stack language
  through Mathlib's own `TM2to1`, `TM1to1` and `TM1to0` to a `TM0` over
  `Fin 4` with quadratic overhead (`raw_compilation`, in
  `OAI/Computability/RationalHitting/Compiler.lean`). Every one of these adds
  a `FiniteAlphabet` side condition, because Mathlib's `FinTM2` requires
  finiteness only of the input alphabet.
- The headline "every library's bridge to the textbook class is asserted,
  not proved" should be qualified, not reversed: the release has proved,
  time-only cost bridges between machine models, and still no proved
  identification of any class with the textbook's. That `TM0` is two-way
  infinite, moves or writes but not both, and halts by an undefined
  transition; its step count is the textbook's time by a modeling
  identification, not a theorem.

The release contains two genuine EXPTIME classes with completeness theorems,
both over small paper-local machines rather than Mathlib's:
`OAI/Computability/WLIdentification/Model.lean` (single-tape, about sixty
definition lines) and `OAI/Combinatorics/VariableWL/Model.lean` (multitape). It contains P and NP over Mathlib's `TM2`
(`BinPacking/Model.lean`, `KMedian/Hardness/Model.lean`), a machine-level
Cook–Levin, and NP-hardness for every `InNP` language (`DirectedFeedback`).
It contains no PSPACE, no EXPSPACE, no alternation, and no space class on any
Mathlib machine. The plan's "what would change course" trigger, a library on
this toolchain gaining a string-model PSPACE-complete problem with proved
machine-composition combinators, is not met: wrong toolchain, EXPTIME rather
than PSPACE, and composition at the level of a word-program DSL rather than
of machines. It is the closest candidate seen in spirit and deserves a
dated note under C-20 with a re-check trigger.

**C-32, C-30, C-34.** One unlinked file,
`OAI/Computability/Logspace/Deterministic.lean` (585 lines, Mathlib only,
cited by no document in the release), holds the only space measure in the
library and a mechanized configuration-counting chain: injective encoding of
configurations into a finite code type, exact cardinality, pigeonhole halting.
It runs from machine space to machine time. C-32 needs the opposite
direction, from an algorithm to machine space, so it does not touch C-32's
status. What it offers C-30 is a
statement shape: "every configuration the decider reaches fits in B bits" as
an injective map into a counted finite type, carried by an invariant that
state outside a bounded window is trivial, with the space measure taken
cumulatively over the run prefix so that it is monotone, which answers the
ledger's note that a measure shrinking on a stack pop is not. Its space convention cites no referent, the defect the plan
holds against `complexitylib`.

**C-14 and C-47.** `inEXPTIME_of_budget` in `WLIdentification/
ExponentialComputation.lean` is the shape a machine-checked EXPTIME
membership would take: a polynomial-time word program run against a unary
budget word of length 2^(C(n+1)^d), compiled to the paper-local machine with
a proved step bound. The archive decider cannot be dropped in, because it is
a Lean function rather than a first-order program and its design is doubly
exponential (C-34). C-47, backward induction over 2^poly states with a
polynomial successor, is the one folklore row that fits the pattern and could
become `proved` relative to that library's own class definition, in a side
workspace, after a port of about 4,400 lines that the plan's no-re-pin rule
currently forbids.

**C-33.** The release's hardness statements share one shape: a `Reduction`
record bundling `construct`, a `TM2ComputableInPolyTime` certificate,
`FiniteAlphabet`, and correctness split into a yes and a no lemma, with the
mathematical reduction proved on an abstract finite type and transported
through a numbering equivalence. A board printer exists as a bounded
arithmetic DSL compiled to `TM2` (`tableComputer`, in
`OAI/Computability/MaxCut/Machines/Printer.lean`). Five of six hardness
families leave "3SAT is NP-hard" implicit; a Go reduction in this style
would likewise leave "TQBF is PSPACE-hard" `cited`. The gadget mathematics
is untouched.

**C-56, C-57, C-24: the certificate route.** The release's Snaky development
(`OAI/GameTheory/SnakyTwentyOne`) is the cleanest large instance seen of this
project's own rule, untrusted search emits a witness and the kernel checks it:
one generated lemma per strategy node, side conditions by `decide +kernel`,
one proved soundness rule, zero `native_decide` over about 12 MB of
generated Lean, and 1.35 million `decide +kernel` library-wide. The reader's
adaptation, which touches nothing in `Defs.lean`:

- a `Cert` tree (leaf, mover with one move, waiter with all legal replies)
  and a reflective `checkCert` over `Decide.lean`'s computable layer;
- `checkCert_sound`, a near copy of `decideWins_sound`, concluding `WinsFor`
  from a checked certificate through the existing bridges `step'_toState`,
  `ended'_iff` and `winnerZ_eq_winner`;
- witness theorems: for `X.X.X.` on 1×6 a Black certificate under SSK at
  floor 1 and a White certificate under PSK, the latter turned into
  ¬`BlackWinsPSK` by `C28_not_both`; the same pair for `OOO/.X.` on 2×3;
  for the 1×n values with n ≤ 6, verdict pairs "Black wins at floor v−1,
  White wins at floor v," which certify verdicts and not the score claim
  C-24 makes, since C-55 is open;
- a recording mode in `superko-solve`'s verdict search emitting the proof
  DAG with mirrored moves off, since Lean recomputes legality and a skipped
  waiter child would be rejected;
- splitting at the top few plies into separate theorems, the one Snaky
  habit worth keeping; per-node generated lemmas for the whole tree are not,
  at 43 generated lines per node.

A Lean-checked witness would no longer rest on the unlicensed divergences
the witness files name. The blockers are measurable and unmeasured: the
proof-tree size, for which the recorded search counts of 8.0 and 12.8
million nodes are a loose upper bound, and the kernel cost per guided node,
where the unguided decider already fails to finish on the empty 2×3. Snaky's
envelope trick depends on monotonicity and does not port; on six points it
is also unnecessary.

**Comparator pattern.** Each of the release's 405 headline theorems has a
statement file importing only Mathlib and ending in `sorry`, a JSON naming
the solution module and the permitted axioms, and an external checker
(`leanprover/comparator`). This mechanizes what `results/axioms.txt` and
`tools/check-lean.sh` do by hand and forces each headline statement to be
re-read in isolation, this project's `Defs.lean` audit boundary in another
form. Worth adopting when there are more headline theorems.

## Habits worth adopting

The ten proof-discovery documents, described in the companion entry, show
four habits that map onto this project's practice: an exhaustive obstruction
catalog, one named obstruction per abandoned approach, which §5 of
open-questions already keeps for history summaries; an adversarial zoo of
small positions and histories with known values and separators, run against
any proposed summary or gadget before investment; a prove-too-much test, a
variant rule under which a proposed upper-bound argument would wrongly
collapse a known separation, such as superko chess (C-5); and the resource
recurrence written out for any candidate summary, looking for the term that
compounds.

## Leads outside the release

Cited by readers from the papers' bibliographies, not verified here:
Ehrenfeucht and Mycielski 1979 and Aminof and Rubin on first-cycle games,
the right references for why the "closer loses" payoff resists positional
arguments; Sipser 1980 and Lange, McKenzie and Tapp 2000 on reversible
versus deterministic space, adjacent to the undirected-geography reading
behind C-6 and C-21; Sakoda and Sipser 1978 for the fooling-set technique in
its home setting.

## Recommended next actions

1. Record under C-20 a dated note, `computed` by reading, on the release's
   EXPTIME classes and composition theorems, and correct the `proof_wanted`
   sentence in the grounding plan. No status changes.
2. Pre-register an experiment for the certificate route on `X.X.X.`:
   proof-tree size from the Rust emitter first, then the kernel cost of one
   subtree; kill condition, a subtree the kernel cannot check without
   `native_decide` or raised limits.
3. Write the nilpotency remark and the fooling-set sketch as a proposal
   against C-15's legality form, and decide what C-15 measures.
4. Acquire Ehrenfeucht–Mycielski and Aminof–Rubin for `references/`.
5. Adopt nothing from the release's Lean as a dependency; port patterns in a
   side workspace under the existing plan.
