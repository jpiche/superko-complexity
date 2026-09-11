# 2026-09-11 — settling the grounding for complexity claims

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`) designed the survey, wrote the syntheses and this entry, and re-compiled every Lean artifact cited; Claude Opus 5 (`claude-opus-5`) ran the verification, design, judging, critique and follow-up stages and produced the decider merge and the encoding

The outcome is in [`docs/plans/complexity-grounding.md`](../docs/plans/complexity-grounding.md).
This entry is the history: what was asked, how it was run, what nobody
expected, the two decisions that are policies rather than facts, and the
dissent.

## What was asked

Experiment 001 closed with a branch decision — prose classes, cite the
scaffolding, complexitylib as a named fallback — that rested on a cost
inference, and pre-registered experiment 002 with a ten-day box and a line
threshold. The maintainer's instruction on 2026-09-10: settle the grounding
first, on merits; do not reinvent the wheel; ignore every duration in the
repository, which was scaffolding. So the question became: over what
definitions does this project state, and where it can prove, resource-bounded
claims — and the answer had to come from primary sources without a time
estimate anywhere in it.

## How it was run

A workflow of fifty-two agents over two days. Eleven readers, one per candidate
grounding: complexitylib, descriptive-complexity, cslib, a bespoke layer over
Mathlib's own machines, HardnessReductionsLean, the explicit-cost-model
approach and its precedents, combinatorial-games for the game side, the small
Lean repositories a fresh GitHub sweep turned up, the Coq and Isabelle
precedents, the literature's own conventions (which meant obtaining and
reading the papers), and the ecosystem's direction. Every fact sheet was then
attacked by two verifiers with different lenses — re-read the source; refute
the fitness assessment — and every one came back partly confirmed with a list
of corrections, which is what the lens was for. Five designers proposed a
grounding from five angles, one of them written to defend the previous
decision. Three judges scored them. A synthesis. A completeness critic, whose
four follow-up briefs were run and verified: two missed candidates, the
merge of two half-finished deciders, the acquisition of Robson, and the
encoding.

The run was interrupted once by the session limit with eighteen agents done
and twenty-five unstarted; it resumed from cache with the bounded stages moved
to Opus, which is now the standing practice for this project. The judgments
are inputs, not evidence: the first synthesis called their agreement evidence,
and the critic pointed out that a reader of the decision cannot check a
judgment. Every fact the decision uses was re-derived or is marked at the
strength of a survey report.

Two probes in the survey went beyond reading. One built complexitylib against
this project's own Mathlib revision and elaborated a Go encoding over its
classes. One compiled cslib's multi-tape layer, unmodified, against the same
revision. Both are measurements, and neither is a reason to adopt anything.

## What nobody expected

**The novel content was provable without any library, and it got proved.**
Experiment 002's premise was that the mathematical input was done and only
machine engineering remained. It was the reverse. At 2026-09-09 nothing about
the game was computable — `Situation.after` goes through the classical
`resolve`, `State.seen` is a `Set`, there was no move enumeration — and the
first thing any grounding needed was a decider with a bridge to `WinsFor`.
Two designers built one independently in scratch, one proved correct with a
rational leaf, one kernel-reducible with an integer leaf, and neither both.
The critic caught that the synthesis had presented the merge as a two-site
edit on an unverified conjunction. The merge was then carried out as a
follow-up, and it was exactly the two-site edit: the resulting file has
fourteen theorems, every one on the three admitted axioms, and its headline
carries `⌊komi⌋` on the left. The encoding was written the same afternoon,
seven hundred lines, with the lower length bound as a theorem. I compiled all
of it from `lean/` myself before writing a word of the decision.

**`Rat.sub` is irreducible.** Every decider whose leaf computes `Defs.lean`'s
rational `margin` is stuck in the kernel at every komi, because Lean core
marks rational subtraction `@[irreducible]` at this toolchain. The judge who
first found the stall diagnosed the decidability instance; the follow-up found
the real cause. The integer leaf `winnerZ b ⌊komi⌋`, with a proved bridge to
`winner`, is the only kernel-reducible leaf, and the per-instance floor is one
`norm_num` call — Mathlib ships an extension for `Int.floor` that a follow-up
agent had reported absent. This is a workaround tied to a toolchain and is
documented as one.

**A committed ledger sentence was refuted by a kernel-checked instance.** C-3's
detail said the archive argument is insensitive to the input encoding and so
depends on nothing else. Under a stone-list encoding the empty 4×4 board takes
24 bits while there are 2·3^16 situations; `decide` proves the archive does not
fit in `2^|w|`. The argument is insensitive to which history convention is
chosen and sensitive to compression, and it consumes the lower length bound.
That is why C-3 now depends on C-31. Whether the language lies outside
EXPSPACE under a compressive encoding is not established, and the encoding
file's own docstrings, which said "false rather than loose", were rewritten to
say only what is proved.

**Lichtenstein and Sipser's game is not this project's.** Read in full for the
first time: ko omitted, a capture order that makes suicide effectively legal,
territory scoring with judged dead stones, no komi, no tie rule, White to move
in the constructed positions, no reduction resource named anywhere, and no
outcome stated for the infinite plays their rules admit. C-2 is narrowed to
their ruleset; the transfer is C-33 with no source. This is the status rule
working late: the parenthetical the ledger carried since the first commit was
never theirs.

**Every bridge is asserted.** Not one library — Lean, Coq or Isabelle — proves
that its class is the textbook's. complexitylib cites Arora–Barak in
docstrings and says in its README that it sets its own conventions;
descriptive-complexity's docstrings say its EXPSPACE is a definition with no
capture theorem claimed, while its citation file says the opposite; cslib's
machine header lists its deviations from Papadimitriou and asserts each is
equivalent; the Coq Cook–Levin cites a paper for the λ-calculus-to-machine
transfer. The prose grounding's whole advantage — a referent a reader can open
— is the one property no library has, and the survey found no exception.

**Three candidates were missed by the first sweep** and found by the critic
with GitHub queries no reader had run: Algolean (six authors, a time-and-writes
cost pair on a single-tape machine, listed on Reservoir before experiment 001
ran), Caliper (a peak-live-memory measure over an imperative language, no
license file), and an alternating-machine framing of the question. None
changed the decision. Two of them refuted a negative the survey had written —
that no Lean precedent exists for a space measure over algorithms — and that
sentence is withdrawn. The lesson is the one experiment 001 already recorded
about caps: a negative inherits the scope of the search that produced it.

**The hypothesis-carrying grounding is vacuous in the form that avoids a
machine.** State the class membership as a theorem whose one unproved step is
a named hypothesis over Lean functions, and with `Classical.choice` any
language has a one-step decider from a constant-size configuration; the
hypothesis puts everything in the class. Making it non-vacuous requires a
deep-embedded program, which is the machine route with the cost assumed
instead of proved. Its naming discipline survives: C-32 is one named sentence,
and inside experiment 003 a missing library lemma is carried as a named
hypothesis rather than worked around.

**Robson is the load-bearing unread source.** Neither 1984 nor 1985 could be
obtained from this host by any route tried, and every route and its status is
logged. The 1985 abstract states the machine-level result as a case split —
restricted space is equivalent either to time or to space exponential in it —
and the arithmetic inverts the critic's conditional: the space-exponential
branch would give 2-EXPTIME, weaker than the archive bound. Only the
time-exponential branch would make C-3 `cited`, and only if Robson's
"Chinese version of Go" is situational rather than positional, which the
secondary accounts suggest it is not.

## Two decisions that are policies

**The bridge-marking policy.** `docs/trusted-base.md` item 4 counts the bridge
statements a reader must audit. The five proposals reported five different
counts (8, 12, 14, 16 and "7→13") because they counted different things, and
the first synthesis wrote one of them as a fact; the revised synthesis
recommended marking only the decider's root correctness statement and
treating its intermediate bridges as internal. The promoted files do not do
that: they follow the repository's existing convention and mark every
statement that ties a computable twin to a `Defs.lean` notion — seven in
`Decide.lean`, three in `Encoding.lean` — and the reviewer caught the prose
saying otherwise. The convention is the policy, applied uniformly: seventeen
marked statements plus the root, eighteen items against the threshold of
twenty set on 2026-09-09. That is close, and it is meant to be felt: the next
bridge is a decision recorded here, not absorbed.

**The language.** `docs/formal-model.md` §7 makes the color to move an input
and asks whether Black wins; `Defs.lean`'s `BlackWins` fixes Black to move;
the scratch encoding defined a mover-wins language; Lichtenstein and Sipser's
hard instances have White to move. The first synthesis would have amended §7
to Black-to-move. The revised one keeps §7's language, which needs no change
to `Defs.lean` (`WinsFor` already takes the starting color), lands
White-to-move instances directly, and needs neither a complement nor
determinacy for hardness. `BlackWinsFrom` is that predicate; `BlackWins` is
its Black-to-move slice by `Iff.rfl`. Komi is quantified over and encoded
exactly, licensed by C-27; "closes OPEN-3 by theorem", which the first
synthesis wrote, is withdrawn in favor of "resolved by decision, licensed by
theorem".

## The dissent, preserved

The strongest case against the decision is the maximal-machine-checked
proposal's, and its factual base is measured: in a copy of complexitylib
re-pinned to this project's Mathlib, `decidesInSpace_of_keepsWindow` and
`loopTM_keepsWindow_indexed` report only the three admitted axioms, a
four-line exponential specialization elaborates, and a file importing both
`SuperkoComplexity.Defs` and `Complexitylib.Classes.Space` elaborates a
`goLang` membership statement. Its argument: the boundary rule says every
novel claim lands on the formalized side and every unformalized claim is one
the project cites, and C-32 — a sentence about the project's own Lean
functions that no source proves — is novel content on the unformalized side
with nothing citing it, which is exactly the arrangement the rule forbids.
The library route would shrink the residue to a sentence about arithmetic
(that complexitylib's head-position convention is the textbook's up to a
constant), which a held textbook could settle.

I take C-32 to be the rule's own named exception: it is the same sentence at
which every published game-membership proof stops, and putting a library's
class definitions into the trusted base to avoid it trades a sentence a reader
can check against a book for definitions a reader can check against nothing.
A maintainer who reads the rule as inviolable should take the dissent's route,
and experiment 003 runs either way; every outcome of it is informative and
nothing load-bearing depends on it.

## What is not established

Nothing here is about Go rather than about `Defs.lean`; C-9 has not
reproduced, and the decider cannot help: a winner is not a game count, and
kernel evaluation reaches only small boards (single queries on 1×3, 1×4 and
the empty 2×2 finish; an empty 2×3 did not; the reach depends on the komi as
well as the board, because the search short-circuits). C-30, the run-level
configuration bound, is open, and the promoted decider is big-step and
carries the archive by value, so the honest space account today is
per-frame. The color-general corollary of C-29 that §7's language wants went
through during promotion with the headline's proof. The RAM-space to
machine-space step of C-32 is covered by no source read. Every Sipser number
is unverified. Robson is unread. And the judgments that ranked the proposals
are not artifacts anyone can check; the decision does not rest on them.

## Wrinkles, so nobody rediscovers them

- In the harness's shell, `grep -r` rooted at the repository root silently
  returns nothing for files under `data/`. One follow-up agent asserted that
  no encoding existed anywhere in the repository while a 729-line one sat
  under `data/grounding/scratch/`. Use `command grep`, or `rg`.
- descriptive-complexity has a stray `main` branch holding an unrelated
  library; raw-file fetches from `main` return the wrong code with HTTP 200.
  Pin `master` or a tag.
- complexitylib's Mathlib pin is 343 commits behind this project's; Lake
  resolves one Mathlib per workspace, so any probe against it means a
  re-pinned copy plus a `lake update` for two transitive dependencies before
  `lake exe cache get` will run. The copy under `data/` is what experiment 003
  starts from.
- `WellFounded.fix` definitions do not reduce; the decider is fuel-indexed
  for that reason, with the fuel discharged by C-26.
- A `decide` on a rational literal written with `/` or as a decimal is itself
  stuck (`Rat.inv`, `Rat.ofScientific`); the integer handed to the decider has
  to come from a proof.
