# Claim ledger

Every claim this project makes or relies on, with its status and its evidence.
This file is the spine of the project: `docs/` prose may state a claim only if
the claim is here, and may state it only at the strength recorded here.

[`../tools/check-ledger.sh`](../tools/check-ledger.sh) parses the table below
and enforces two invariants:

1. **No `proved` claim depends on a claim that is not `proved` or `cited`.**
   A proof resting on folklore, on a computation, or on a conjecture is not a
   proof, and the gate treats it as a build break.
2. **Every `formalized:` claim names a theorem that exists** in the Lean build
   with a clean axiom list.

## Statuses

| Status | Meaning | May a proof depend on it? |
|---|---|---|
| `proved` | proved here; machine-checked unless the formalization column says otherwise | yes |
| `cited` | proved in the literature; the source gives an actual proof | yes |
| `folklore` | asserted in the literature without proof, or with a sketch too thin to check | **no** |
| `computed` | verified exhaustively on finite instances; evidence about the general case, not proof of it | **no** |
| `conjecture` | believed, with a reason recorded | **no** |
| `open` | genuinely unsettled, including by us | **no** |
| `refuted` | disproved; kept with the counterexample | n/a |

`folklore` is not a slight. The EXPSPACE archive argument is almost certainly
correct. It is `folklore` because no source proves it, and a project whose
whole claim to attention is checkability cannot afford to inherit an unchecked
step.

## Formalization

| Value | Meaning |
|---|---|
| `formalized:Name` | machine-checked; `Name` is the Lean theorem |
| `formalizable` | within reach of current Mathlib; not yet done |
| `infra-gap` | blocked on complexity-theory formalization out of this project's reach: for a `cited` row, formalizing means re-proving the source; for an unproved row, there is no proof to formalize (C-20) |
| `prose-only` | a claim about the literature or the rules; not a mathematical statement |
| `n/a` | not applicable |

## The table

Format is fixed and parsed mechanically: six pipe-delimited columns, ids
matching `C-<digits>`, `depends-on` comma-separated or `-`.

| id | statement | status | formalization | depends-on | witness |
|---|---|---|---|---|---|
| C-1 | Encoding (C) — position as root, empty history — is the faithful formalization of SUPERKO-GO | open | prose-only | - | docs/formal-model.md §5 |
| C-2 | Generalized Go under the Lichtenstein–Sipser ruleset is PSPACE-hard under polynomial-time many-one reductions | cited | infra-gap | - | Lichtenstein–Sipser 1980, read in full 2026-09-10; the reduction notion is imported via Meyer–Stockmeyer 1973 |
| C-3 | SUPERKO-GO is in EXPSPACE, by carrying an archive of visited situations | folklore | infra-gap | C-29, C-30, C-31, C-32 | Saffidine–Teytaud–Yen 2015, Thm 1; the prose residue is C-32, proofs/C-32.md |
| C-4 | Go under Japanese rules is EXPTIME-complete | cited | infra-gap | - | Robson 1983 |
| C-5 | The no-repeat formula game is EXPSPACE-complete | cited | infra-gap | - | Robson 1984, body unread by this project; reported consistently by Hearn 2006 §6.3, Zhang 2019 and Chung 2026 |
| C-6 | Robson's EXPTIME construction, played under superko, stays in EXPTIME via undirected vertex geography | folklore | infra-gap | C-21 | Demaine–Hearn, Playing Games with Algorithms |
| C-7 | Under PSK, games correspond one-to-one with simple paths from the empty position in the situation graph | cited | formalizable | - | Tromp–Farnebäck 2006, Lemma 2 |
| C-8 | Under SSK the corresponding paths need not be simple; a position may be visited twice | cited | formalizable | C-7 | Tromp–Farnebäck 2006, Lemma 2 remark |
| C-9 | The number of 2x2 games under PSK from the empty board is 386356909593, under Tromp–Farnebäck's rules | computed | formalizable | - | Tromp–Farnebäck 2016 Table 7, held, whose evidence is enumeration by Tromp's `2x2.c`; reproduced exactly by the Rust mirror under the paper's suicide convention, results/count-games-2x2-psk-remove-own.txt (experiment 004) |
| C-10 | Repetition-free games exist whose length is exponential in the board size | cited | formalizable | - | Walraet–Tromp 2016 |
| C-11 | The 1x9 empty-board minimax score under PSK is 0 | open | formalizable | - | disputed: 0 vs 4 across sources |
| C-12 | SUPERKO-GO under PSK and under SSK lie in the same complexity class | open | infra-gap | C-1 | no published separation or equivalence |
| C-13 | Every play sequence under SSK from any start state is finite | proved | formalized:C13_terminates | - | lean/SuperkoComplexity/Results/C13_Termination.lean |
| C-14 | SUPERKO-GO is in EXPTIME | conjecture | infra-gap | C-1 | Robson's belief as reported second-hand in a 2011 blog comment and in Hearn 2006; no proof |
| C-15 | The history-congruence index H(n) grows as 2^poly(n) | open | formalizable | C-13 | - |
| C-16 | Mechanical area scoring agrees with AGA agreed scoring under optimal play | conjecture | formalizable | - | docs/formal-model.md §4; the resumption mechanism the argument uses is AGA Rules 9 and 10, held |
| C-17 | A minimal position exists whose game value differs under PSK and SSK | open | formalizable | - | KGS anecdotes; no published minimal case |
| C-18 | Under AGA rules a pass is exempt from the superko restriction | cited | prose-only | - | AGA Rules 2, 6 and 7 (held): Rule 6 restricts playing, Rule 2 says a pass is always legal; Tromp–Taylor Rule 6 (held) makes a turn either a pass or a non-repeating move |
| C-19 | Pass stones do not affect the outcome under area scoring | cited | prose-only | - | AGA Rule 12 (held): under area counting prisoners are ignored, and a pass stone is a prisoner; Rule 11's extra White pass changes no point of the board |
| C-20 | Mathlib has no resource-bounded complexity class; the downstream Lean 4 libraries surveyed supply classes, a generic space-membership lemma that is axiom-clean at this project's Mathlib pin, and space measures over machines and programs, and none supplies a PSPACE-complete source problem under polynomial-time many-one reductions over a string-encoded Turing-machine class | computed | prose-only | - | experiments/001-mathlib-complexity-audit and notebook/2026-09-11-complexity-grounding.md; mathlib f5e9087 checked 2026-09-09; descriptive-complexity de212562, complexitylib 6c248df and EdouardBonnet/classical-complexity 026a662 checked 2026-09-10; cslib ec768ef, Shreyas4991/Algolean f64556d and zksecurity/caliper b62f7c8b checked 2026-09-11 |
| C-21 | Undirected vertex geography is solvable in time polynomial in the graph size | cited | infra-gap | - | Fraenkel-Scheinerman-Ullman 1993 |
| C-22 | Directed vertex, directed edge and undirected edge geography are PSPACE-complete | cited | infra-gap | - | Schaefer 1978 and Lichtenstein–Sipser 1980 Thm 2 for the directed variants; Fraenkel–Scheinerman–Ullman 1993 for undirected edge geography, via secondary reading; arXiv:2108.09367 |
| C-23 | The PSK game counts on 1x1, 1x2, 1x3, 1x4 from the empty board under Tromp–Farnebäck's suicide-permitting rules are 1, 9, 907, 2098407841 | cited | formalizable | - | Tromp–Farnebäck 2016 Table 7, held; reproduced by the Rust mirror under remove-own suicide (computed): results/count-games-1x1-psk-remove-own.txt through results/count-games-1x4-psk-remove-own.txt; the first three also hold under Defs.lean's rule, the fourth does not (C-36) |
| C-24 | The 1xn empty-board PSK minimax scores for n <= 8 are 0, 0, 3, 4, 0, 1, 2, 3 | cited | formalizable | - | test_data/literature/linear-go-scores.toml (transcribed, unverified) |
| C-25 | No published complexity result isolates a repetition rule as the driver of a class change for chess, shogi or xiangqi | open | prose-only | - | confirmed absence; literature search 2026-09-09 |
| C-26 | A game begun from a position as the root of play lasts at most 4·3^(m·n) moves under either superko rule | proved | formalized:C13_length_bound_explicit | C-13 | lean/SuperkoComplexity/Results/C13_Termination.lean |
| C-27 | The verdict depends on komi only through its floor: every komi is equivalent for BlackWins to the half-integer floor(komi) + 1/2, so the tie-to-White convention is unobservable for BlackWins at a fixed color to move | proved | formalized:C27_blackWins_iff_halfInteger | C-29 | lean/SuperkoComplexity/Results/C27_Komi.lean |
| C-28 | From every state exactly one color has a winning strategy: existence under any repetition rule that excludes repeats, exclusivity under any rule | proved | formalized:C28_determined | C-13 | lean/SuperkoComplexity/Results/C28_Determinacy.lean |
| C-29 | The fuel-indexed archive decider, run at floor(komi) with fuel 4·3^(m·n)+1 from a position as the root of play, returns true exactly when BlackWins holds, and likewise under positional superko | proved | formalized:C29_decideWins_iff_blackWins | C-13, C-26 | lean/SuperkoComplexity/Results/C29_DeciderCorrect.lean |
| C-30 | Every configuration the archive decider reaches from a root position fits in 2^O(m·n) bits, with the archive shared along the search path | open | formalizable | C-26 | a static bound on one configuration is proved in scratch; the run-level theorem is not written |
| C-31 | The encoding enc is injective and decodes exactly, and 2·m·n ≤ length(enc) ≤ 2·m·n plus terms logarithmic in the dimensions and the komi | proved | formalized:C31_enc_length_lower | - | lean/SuperkoComplexity/Encoding.lean |
| C-32 | A deterministic Turing machine that decodes the input, iterates the decider's step and reads off the verdict uses work space polynomial in the configuration size, uniformly in the board; with C-30 and C-31 this places SUPERKO-GO in EXPSPACE | folklore | prose-only | C-29, C-30, C-31 | proofs/C-32.md; Saffidine–Teytaud–Yen 2015 Thm 1; Dershowitz–Falkovich-Derzhavetz 2015 Thm 2 for the algorithm-to-RAM half |
| C-33 | The Lichtenstein–Sipser construction is a reduction to SUPERKO-GO as stated in formal-model.md §7: situational superko, mechanical area scoring, no suicide, komi, color to move carried | open | formalizable | C-2, C-28 | no source; five ruleset divergences and an undefined outcome for infinite play in the source |
| C-34 | The archive decider explores at most (m·n+1)^(4·3^(m·n)) nodes | open | formalizable | C-26 | not attempted; records why the archive argument lends C-14 no support |
| C-35 | BlackWins depends on komi only through floor(komi) clamped to the interval from −(m·n)−1 to m·n | open | formalizable | C-29 | winner-level lemmas proved in lean/SuperkoComplexity/Results/C27_Komi.lean; the lift through WinsFor is not attempted |
| C-36 | From the empty 1x4 board under PSK, Defs.lean's rules give 719178893 games, not the published 2098407841; the suicide convention is observable in the count from four points on a line | computed | formalizable | - | results/count-games-1x4-psk-forbid.txt against results/count-games-1x4-psk-remove-own.txt, experiment 004; the witness play is Black at the end of `.OX.`, a two-stone self-capture to `.O..` |
| C-37 | From the empty 1x4 board under SSK, Defs.lean's rules give 1359471437 games | computed | formalizable | - | results/count-games-1x4-ssk-forbid.txt; no independent source exists for any SSK count |
| C-38 | From the empty 2x2 board under PSK the game count is the same under both suicide conventions, because every position-changing self-capture on 2x2 returns to the empty board, which the root archives | computed | formalizable | - | results/count-games-2x2-psk-forbid.txt and results/count-games-2x2-psk-remove-own.txt agree on every field, and the suicide refusals of the first equal the extra repetition refusals of the second; the census of the eight plays is in notebook/2026-09-11-experiment-004.md |
| C-39 | From the empty 2x2 board under PSK, Defs.lean's rules give 386356909593 games, the number Tromp computed under his own rules from his own definitions | computed | formalizable | - | results/count-games-2x2-psk-forbid.txt, experiment 004; the independent-agreement validation of docs/trusted-base.md, at computed and not above |
| C-40 | From the empty 2x2 board under SSK, Defs.lean's rules give 1391718029753 games, 3.6 times the PSK count | computed | formalizable | - | results/count-games-2x2-ssk-forbid.txt, experiment 004; no SSK count has been published at any size, so nothing independent checks it |

## Detail

Claims needing more than a row.

### C-1 — the encoding question

Narrowed, not closed. The literature's convention is encoding (C): Lichtenstein
and Sipser ask about "an arbitrary GO position on an n × n board"; Saffidine,
Teytaud and Yen call a position with no history of forbidden states "the
classical considered setting"; Hearn rejects history-as-position because
complexity results would then be taken relative to an exponentially larger
input; Stockmeyer and Chandra speak of "the size of the starting position", on
the strength of their abstract alone. `Defs.lean` commits to (C), and `start`
seeds the history with the root situation, so the accurate phrase is "history
= the root situation alone", not "history empty". The rules texts, now held,
support that seeding: AGA Rule 6 forbids recreating "a previous board
position from the game", and Tromp–Taylor's Rule 6 forbids repeating "an
earlier grid coloring" in a game that Rule 5 starts from the empty grid, so
the position a game starts from counts in both. Extending that to a game
begun from an arbitrary position is this project's decision.

The bit-level encoding is now concrete — `Superko.Enc.enc` (C-31) — and no
source fixes one, so the identification of `enc` with the literature's
unstated encoding is asserted by inspection. That, and OPEN-4's reachability
question, is the residue. `Position m n` ranges over all colorings, including
ones no play produces; Lichtenstein and Sipser do not argue reachability for
their constructions either. Whether any position's value differs between
"history empty" and "history = the root situation" is not settled, and
nothing rests on it now that the reading is fixed from the texts.

### C-2 — what Lichtenstein and Sipser prove, and for which game

Read in full on 2026-10 by the grounding survey
([`../notebook/2026-09-11-complexity-grounding.md`](../notebook/2026-09-11-complexity-grounding.md)).
The row is narrowed to what the source establishes.

Their game is a reduced ruleset: passes at any time, the game ends when both
pass, a capture order that removes the mover's own surrounded groups after the
opponent's (so suicide is effectively legal — an inference from the capture
order, since the paper never mentions suicide), territory scoring with judged
dead stones, no komi, no tie rule, and ko omitted. Their constructed
positions have White to move and ask whether Black wins. Because ko is
omitted and the game ends only on two passes, infinite plays exist and the
paper states no outcome for them. They never name the reduction resource; the
notion they import for "Pspace-complete" is logspace reducibility from Meyer
and Stockmeyer 1973, of which polynomial-time many-one reducibility is the
sound weakening, and that is what the row now says.

Propositions 4 to 6 carry one-word proofs and Proposition 1's case analysis
is omitted with the planar drawing deferred to a memo; the row keeps `cited`
because the construction and its correctness argument are given, and records
that the source sits nearer the `folklore` boundary than a bare citation
suggests.

The parenthetical this row used to carry — "under every ruleset (the
reduction builds no kos)" — is not the source's. It is Demaine and Hearn's,
asserted in one sentence, and it concerns kos rather than superko-forbidden
repetition. The transfer to this project's object is C-33, `open`: no source
addresses it, and the transfer must supply a winning convention Lichtenstein
and Sipser never state and re-verify the gadgets under a rule that can forbid
the punishing move.

### C-3 — the archive bound, and where exactly it stops being proved

`folklore` because the Saffidine–Teytaud–Yen survey states it as a theorem
whose proof is a single sentence, labels it folklore itself, and words its
superko positionally where this project's object is situational (the archive
argument is insensitive to that difference). Hearn 2006 cites Robson 1984 for
the membership; that paper's body is unread here, and the abstract of Robson
1985 states the machine-level result as a case split whose branch for
situational superko is unidentified. Obtaining Robson is the first action;
if it covers situational superko on the time-exponential branch, this row
becomes `cited` and everything below is corroboration.

The argument's content is now split across rows so that the ledger names the
step that is not proved. Proved over `Defs.lean`: the decider decides the
game (C-29), the input is honest with the board linear in its length (C-31),
komi enters only through its floor (C-27). Open on the proved side: the
run-level bound on the decider's configuration (C-30). Prose: the one
sentence that a Turing machine iterating the decider's step uses space
polynomial in the configuration size (C-32), written out in
[`../proofs/C-32.md`](../proofs/C-32.md) with its first line saying it is not
machine-checked. The residue of this row is C-32, not "the archive argument".

**The earlier detail here was wrong in the half that matters.** It said the
argument is insensitive to the input encoding and so depends on nothing else,
not even C-1. It is insensitive to which of encodings (A), (B) and (C) is
chosen, because each carries the board explicitly; it fails under a
compressive encoding. `Superko.Enc.sparse_encoding_refuted` is the
kernel-checked instance: under a stone-list encoding the empty 4×4 board takes
24 bits while there are 2·3^16 situations, so the archive does not fit in
`2^|w|`. The argument consumes the lower length bound `2·m·n ≤ |enc|` of
C-31, and the row now depends on it. Whether the language lies outside
EXPSPACE under a compressive encoding is not established; the asymptotic
reading is an inference.

### C-13 — termination

Proved, and machine-checked: `Superko.C13_terminates` in
[`../lean/SuperkoComplexity/Results/C13_Termination.lean`](../lean/SuperkoComplexity/Results/C13_Termination.lean),
axioms `propext, Classical.choice, Quot.sound`. The statement is that the play
relation `Superko.Follows SSK` is well-founded, so no infinite sequence of
legal moves exists. `C13_terminates_psk` is the same under positional superko.

The measure is a single natural number, `2 · (unvisited situations) + (passes
remaining)`, not the lexicographic pair this project first proposed. The weight
of two is what absorbs a play's reset of the pass counter, and flattening the
order removes the well-founded-order plumbing at the cost of a factor of two in
C-26.

Termination turns out not to depend on C-18 after all. The proof runs through
`Superko.ExcludesRepeats` — the property that a legal *play* may not recreate a
seen situation — and takes no position on passes at all: what bounds a run of
passes is the pass counter, not the history. `Superko.wellFounded_follows`
states it in that generality, so any variant rule that ends the game on two
passes and forbids repeated plays terminates with no further work. Whichever
way OPEN-1 falls, this claim survives.

What it does not establish by itself: determinacy, which is C-28 and recurses
on the well-founded relation this claim supplies. Nor does it show that play
must *reach* an ended state — only that it cannot go on forever.

### C-26 — how long a game can be

`Superko.C13_length_bound_explicit`. A game begun from a position as the root
of play — encoding (C) of [`formal-model.md`](formal-model.md) §5 — makes at
most `4 · 3^(m·n)` moves, because there are `2 · 3^(m·n)` situations
(`Superko.card_situation`) and the measure starts at twice that.

The constant is loose and the exponential is not. Two units of measure are
charged on every play though only one reset ever needs absorbing, so a
lexicographic measure would give something nearer `3^(m·n)`; sharpening it means
a different proof, not a tightening of this one. Nothing downstream needs the
constant.

The exponential is the quantity the complexity question consumes. It is the
reason a machine playing a game out needs exponentially many moves, and it is
where the archive argument of C-3 would get its space bound. This claim does
not make that argument, and says nothing about whether the space is necessary.

### C-14 — the headline conjecture

The reason to believe it is C-6, and C-6 is folklore about a *single
construction*. It says nothing about arbitrary Go positions, and the gap
between them is the research problem. Recorded as `conjecture` rather than
`open` only because a specific person with standing is reported to hold it;
that is a reason to investigate, not evidence.

The route to attacking it is in [`open-questions.md`](open-questions.md).

The witness is thinner than the row once admitted: the reported belief is
second-hand, recorded in a 2011 blog comment in the same paragraph as an
opposite speculation by the same person, and Hearn 2006 records it as belief.

Two things the grounding survey adds. The archive decider of C-29 explores up
to (m·n+1)^(4·3^(m·n)) nodes (C-34), doubly exponential in time, so C-3's
argument lends this conjecture nothing. And by APSPACE = EXPTIME (Chandra,
Kozen and Stockmeyer 1981, held only through secondary restatements), the
conjecture is equivalent to the existence of a polynomial-space alternating
machine deciding the game — the crisp form of the open problem, marked as an
interpretation, since such a machine may decide the game by any
characterization and "the superko history compresses into polynomial space"
narrows the problem by an unproved step.

### C-21, C-22 — the geography results

C-21 is what makes C-6 more than an analogy: if undirected vertex geography
were not polynomial, the folklore reason to believe superko Go is in EXPTIME
would evaporate. The ledger records C-6 as depending on it.

C-22 is the other half of the picture, and the reason
[`open-questions.md`](open-questions.md) §5 poses the question it does: the
directed/undirected distinction is exactly what separates polynomial from
PSPACE-hard for the reachability core, so *characterizing when a Go move
creates a directed edge in situation space* is the concrete form of the
upper-bound attack.

### C-9, C-23 — read from the paper and partly reproduced; C-24 — transcribed, not verified

The game counts are read from Table 7 of the held Tromp–Farnebäck revision,
and Tromp's 2×2 program is held and read
([`../references/README.md`](../references/README.md)). Both rows stay
`cited` on the ledger's vocabulary, with a caveat the vocabulary has no word
for: the source's evidence is brute-force enumeration, not a proof, so the
rows are computation done by others. A game in Table 7 is, by Lemma 2, a
simple path from the empty position with its forced passes, which is every
legal alternating sequence ending at the second consecutive pass — the
notion `Defs.lean` induces.

The paper's ruleset permits multi-stone suicide where `Defs.lean` forbids
it, and experiment 004 found the difference observable at four points: the
Rust mirror reproduces all four 1×n counts exactly under the paper's
convention and gives a different 1×4 count under `Defs.lean`'s (C-36,
`computed`). C-23 is therefore stated with its ruleset. On 2×2 the
difference vanishes, as the census predicted: every position-changing
self-capture there returns to the archived empty board, and the two 2×2
runs agree on every field (C-38, `computed`). The 2×2 count under
`Defs.lean`'s own rules is C-39, and it is the definitional validation the
trusted base asks for, at `computed`: the mirror is not trusted, the kernel
has checked no count, and the number says the definitions agree with
Tromp's on the one quantity both can compute, not that they describe Go.

C-24's 1×n scores are still copied from Hayward's course table, with
Weninger–Hayward `sought`, so a failure to reproduce them has three possible
causes rather than two — see
[`../test_data/literature/README.md`](../test_data/literature/README.md).


### C-25 — a confirmed absence

Chess threefold repetition, shogi sennichite and xiangqi perpetual-check rules
all make legality or outcome history-dependent, and the standard
EXPTIME-completeness proofs for those games handle repetition with a turn or
move counter rather than by analyzing the repetition rule itself. No result
appears to isolate the repetition rule as the thing that moves the complexity
class.

Recorded as a claim because it is the novelty argument for
[`open-questions.md`](open-questions.md) §4, and because an absence is exactly
the kind of assertion that decays: it was true of a 2026-09-09 literature
search and should be re-checked before anything is written up.

### C-20 — the infrastructure finding

Not mathematics, but it determines how much of this project can be
machine-checked, so it is tracked like a claim. The audit is
[`../experiments/001-mathlib-complexity-audit/`](../experiments/001-mathlib-complexity-audit/);
the row was re-examined and re-dated by the grounding survey of 2026-09-10
and 2026-09-11 ([`../notebook/2026-09-11-complexity-grounding.md`](../notebook/2026-09-11-complexity-grounding.md)),
which added three repositories the audit had missed.

Mathlib has nothing: no class, no space measure on any machine model, and one
polynomial-time predicate with zero theorems whose composition is a
`proof_wanted`. Downstream, the picture is richer than the row used to say
and the blocker is narrower. complexitylib defines the classes over multi-tape
machines in the string model and its window calculus for space membership is
generic in the bound and axiom-clean at this project's own Mathlib pin, in a
re-pinned copy; cslib has a multi-tape machine with a cells-visited space
measure and no class on `main`; descriptive-complexity defines EXPSPACE as a
logic and proves PSPACE-complete problems under first-order reductions;
Caliper carries a peak-live-memory measure over an imperative program;
Algolean a time-and-writes cost pair over a single-tape machine. What none of
them supplies is a PSPACE-complete source problem under polynomial-time
many-one reductions over a string-encoded Turing-machine class, and none
proves that its class is the textbook's. The consequence for the `infra-gap`
markings is unchanged, and the reason the project adopts no library is
recorded in [`trusted-base.md`](trusted-base.md), not here.

**This row is terminal at `computed`.** The ledger bars `computed` from
becoming `proved` without a Lean-checked certificate, and no certificate can
exist for a claim about other people's repositories. Its only lifecycle is
re-checking and re-dating. Every candidate moved within the survey's own
window; treat each date in the witness as an expiry, and add any repository
examined before re-dating the row again.

### C-27 — komi

`Superko.C27_blackWins_iff_halfInteger`: `BlackWins m n b komi ↔ BlackWins m n b (⌊komi⌋ + 1/2)`.
Obtained from C-29 by rewriting — because the decider's left-hand side sees
komi only through its floor — rather than by induction on `WinsFor`, which is
why the row depends on C-29. Four `winner`-level lemmas accompany it,
including that `winner` is constant once the floor leaves the interval the
board's area allows.

What it settles: the tie half of OPEN-3, by theorem. What it does not: the
tie convention is observable under a color swap at margin zero, so the word
"unobservable" is qualified in the row; the clamp of the floor to the board's
range is proved for `winner` and not lifted through `WinsFor` (C-35); the
positional-superko twins are not written, so the row is situational only.

### C-28 — determinacy

`Superko.C28_determined`: under any repetition rule that excludes repeats,
from every state Black or White has a winning strategy; by well-founded
induction on the relation C-13 supplies, with no library.
`Superko.C28_not_both`: the two cannot both hold, under any rule, by
induction on the Black derivation — no hypothesis at all. Together they
discharge the promise in `Defs.lean`'s `WinsFor` docstring and the caveat in
C-13's detail, and they close a live inconsistency:
[`formal-model.md`](formal-model.md) §6 asserted determinacy while the ledger
said it was not established.

### C-29 — the decider

`Superko.C29_decideWins_iff_blackWins`:

    decideWins ⌊komi⌋ Color.black SSK' (4 * 3 ^ (m * n) + 1) (start' b Color.black) = true
      ↔ BlackWins m n b komi

The right-hand side is `Defs.lean`'s `BlackWins`, quantified over every
position and every rational komi. The left is a derived layer
([`../lean/SuperkoComplexity/Decide.lean`](../lean/SuperkoComplexity/Decide.lean)):
a state whose history is a `Finset`, a Boolean rule, and a fuel-indexed
depth-first search over the game tree that carries the archive. Soundness and
completeness are separate theorems, parametric in any Boolean rule faithful to
a `Repetition` that excludes repeats, so the positional twin is proved, not
asserted. The fuel is exactly C-26's bound plus one: tightening C-26 changes
this statement.

Three facts about the statement that a reader should know. The floor is
there because Lean core marks `Rat.sub` irreducible at this toolchain, so a
decider whose leaf computes `Defs.lean`'s rational `margin` reduces in the
kernel at no komi; the integer leaf with its proved bridge to `winner` is the
only kernel-reducible one, and the per-instance floor is discharged by
`norm_num`. Kernel evaluation by `decide` reaches small boards — single
queries on 1×3, 1×4 and the empty 2×2 finish, an empty 2×3 did not, and the
reach depends on the komi as well as the board because the search
short-circuits (`computed`, 2026-09-11); the decider is not a search engine,
and since a winner is not a game count no run of it bears on the 2×2 count of
C-9. And the statement is an algorithm-correctness statement, the first of
its kind in the trusted base's item 4; the seven bridge lemmas it rests on are
marked and counted there, and a wrong twin among them would make the headline
unprovable rather than misleading, which is why the root statement is the one
to read for vacuity.

### C-30 — the configuration bound

Open. A static bound on one configuration of a decider state — the archive at
most 2·3^(m·n) situations of 2·m·n+1 bits, the stack at most 4·3^(m·n)
frames of O(log(m·n)) bits — is proved in scratch over a state type that
differs from the promoted decider's. The theorem C-32 consumes is the
run-level one: every configuration an explicit small-step machine reaches
from the initial one stays under the bound, with the archive shared along the
search path rather than copied per frame. The promoted decider is big-step
and carries the archive by value, so its naive space is per-frame archives —
still 2^O(m·n), with a larger constant in the exponent. Either a small-step
decider with an agreement theorem, or a restated per-frame bound, closes the
row; the honest measure is the maximum over reached configurations, since a
measure that shrinks on a stack pop is not monotone along the run.

### C-31 — the encoding

[`../lean/SuperkoComplexity/Encoding.lean`](../lean/SuperkoComplexity/Encoding.lean).
`enc` writes the dimensions in self-delimiting binary, one bit for the color
to move, two bits per point in row-major order — pinned inside the project by
`cellFlat_idx` — marked as a bridge, with `boardOf_boardBits` and
`blackWinsFrom_black` — and the parity lemmas rather than by any Mathlib
equivalence whose order is unstated — and the komi as a sign, a numerator and a
denominator. `dec_enc` is an exact round trip, `enc_inj` injectivity, and
`length_enc` an equation. The two bounds serve different claims and are not
interchangeable: the lower bound `C31_enc_length_lower`,
`2·m·n ≤ length(enc)`, is what membership consumes, because only it makes
`3^(m·n)` singly exponential in the input length; the upper bound is what a
hardness reduction's output-size argument consumes. `sparse_encoding_refuted`
shows, by `decide` at 4×4, why the lower bound is load-bearing.

The language `goLang` is built over `BlackWinsFrom`, the predicate of
[`formal-model.md`](formal-model.md) §7 with the color to move as input; its
Black-to-move slice is `BlackWins` definitionally. Komi is encoded exactly,
which is the decision that resolves OPEN-3's quantification half.

### C-32 — the sentence that is prose

The named residue of C-3, written out in [`../proofs/C-32.md`](../proofs/C-32.md).
`folklore`: it is the invariance thesis instantiated to this project's own
Lean functions, which no source proves; the algorithm-to-RAM half is cited to
Dershowitz and Falkovich-Derzhavetz 2015, Theorem 2, and the RAM-space to
machine-space half is covered by no source this project has read (Cook and
Reckhow 1973 is time-only; Slot and van Emde Boas is unread). It is also the
whole of the published one-line proof. The per-step claim is polynomial in
the configuration size, not in the board, which is what absorbs the promoted
liberty test's route through Mathlib's walk enumeration. Uniformity in the
board dimensions and rejection of malformed strings within the bound are part
of the sentence; `dec` is total and recovers the dimensions, and its resource
cost is asserted.

### C-33 — the transfer of the hardness construction

Open, with no source. Lichtenstein and Sipser's game differs from SUPERKO-GO
in five ways recorded under C-2, admits infinite play with no stated outcome,
and has White to move in the constructed positions. Under §7's language the
last is no obstacle. The formalizable content is a map from geography
instances to positions, the theorem that the geography player wins exactly
when Black wins from the image, and an output-size bound; the "no" direction
uses C-28's hypothesis-free exclusivity. Under situational superko the
punishing moves in the source's one-sentence arguments may be illegal, so the
gadgets are re-proved for this game rather than transcribed. The class-level
sentence stays prose.

### C-35 — the clamp

Open. `winner` is constant once `⌊komi⌋` is at or beyond the board's area in
either direction, and the two areas sum to at most `m·n`; both are proved in
`C27_Komi.lean`. The lift through `WinsFor` is not attempted. It is needed for
the sentence "input size polynomial in m·n" over arbitrary inputs and for
normalizing an input's komi; it is not needed by membership, where a longer
komi only enlarges the budget, nor by a reduction, which chooses its komi.
