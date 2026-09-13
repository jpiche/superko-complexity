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
| C-15 | The index of the congruence on histories that agrees on legality and on the value, maximized over roots and komi floors, grows as 2^poly(m·n) | open | formalizable | C-13 | - ; the row used to say "the history-congruence index H(n)", which named neither the congruence nor the quantifier over roots, and open-questions.md §4 leans on it |
| C-16 | Mechanical area scoring agrees with AGA agreed scoring under optimal play | conjecture | formalizable | - | docs/formal-model.md §4; the resumption mechanism the argument uses is AGA Rules 9 and 10, held |
| C-17 | A minimal position exists whose game value differs under PSK and SSK | open | formalizable | - | KGS anecdotes; no published minimal case. Under Defs.lean's rules there is none on any board of at most five points, nor at any of the 792 of 2x3's 1458 roots resolved within 10^7 nodes per search (C-53); under suicide removal the least is X. on 1x2 (C-54); experiment 005, and "game value" is defined in the detail below |
| C-18 | Under AGA rules a pass is exempt from the superko restriction | cited | prose-only | - | AGA Rules 2, 6 and 7 (held): Rule 6 restricts playing, Rule 2 says a pass is always legal; Tromp–Taylor Rule 6 (held) makes a turn either a pass or a non-repeating move |
| C-19 | Pass stones do not affect the outcome under area scoring | cited | prose-only | - | AGA Rule 12 (held): under area counting prisoners are ignored, and a pass stone is a prisoner; Rule 11's extra White pass changes no point of the board |
| C-20 | Mathlib has no resource-bounded complexity class; the downstream Lean 4 libraries surveyed supply classes, a generic space-membership lemma that is axiom-clean at this project's Mathlib pin, and space measures over machines and programs, and none supplies a PSPACE-complete source problem under polynomial-time many-one reductions over a string-encoded Turing-machine class | computed | prose-only | - | experiments/001-mathlib-complexity-audit and notebook/2026-09-11-complexity-grounding.md; mathlib f5e9087 checked 2026-09-09; descriptive-complexity de212562, complexitylib 6c248df and EdouardBonnet/classical-complexity 026a662 checked 2026-09-10; cslib ec768ef, Shreyas4991/Algolean f64556d and zksecurity/caliper b62f7c8b checked 2026-09-11 |
| C-21 | Undirected vertex geography is solvable in time polynomial in the graph size | cited | infra-gap | - | Fraenkel-Scheinerman-Ullman 1993 |
| C-22 | Directed vertex, directed edge and undirected edge geography are PSPACE-complete | cited | infra-gap | - | Schaefer 1978 and Lichtenstein–Sipser 1980 Thm 2 for the directed variants; Fraenkel–Scheinerman–Ullman 1993 for undirected edge geography, via secondary reading; arXiv:2108.09367 |
| C-23 | The PSK game counts on 1x1, 1x2, 1x3, 1x4 from the empty board under Tromp–Farnebäck's suicide-permitting rules are 1, 9, 907, 2098407841 | cited | formalizable | - | Tromp–Farnebäck 2016 Table 7, held; reproduced by the Rust mirror under remove-own suicide (computed): results/count-games-1x1-psk-remove-own.txt through results/count-games-1x4-psk-remove-own.txt; the first three also hold under Defs.lean's rule, the fourth does not (C-36) |
| C-24 | The 1xn empty-board PSK minimax scores for n <= 8 are 0, 0, 3, 4, 0, 1, 2, 3 | cited | formalizable | - | test_data/literature/linear-go-scores.toml (transcribed, unverified); the entries n <= 6 are reproduced under Defs.lean's rule, whose suicide convention is the one the transcribed header names (computed): crates/superko-solve/tests/published.rs; 1x7 and 1x8 do not resolve within its budget of 4*10^7 nodes, and 1x9 is not attempted |
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
| C-41 | Value transfer along a simulation: a relation preserving the situation, the pass counter and legality, and preserved by step on legal moves, preserves who wins; parametric in the repetition rule | proved | formalized:winsFor_transfer | - | lean/SuperkoComplexity/Compress.lean; every compression statement in this project factors through it |
| C-42 | The archive may be pruned to the forward cone of the current situation without changing who wins, and an unreachable situation may be added to it freely | proved | formalized:winsFor_seen_inter_cone | C-41 | lean/SuperkoComplexity/Compress.lean; the cone is reachability under board-legal moves, with the repetition rule out of the way |
| C-43 | The fuel-indexed archive decider decides WinsFor from every state whose archive is nonempty, not only from a root of play | proved | formalized:decideWins_iff_winsFor | C-26, C-29 | lean/SuperkoComplexity/Compress.lean; C-29 is stated at start' only, so without this no mid-game verdict transports to WinsFor |
| C-44 | The played stone survives its own move, so no play produces the empty board and the empty situation has in-degree zero under play edges, on every board | proved | formalized:sitStep_empty_board | - | lean/SuperkoComplexity/Compress.lean and resolve_self in Basic.lean; resolve clears c.other only |
| C-45 | Over the positions play can reach, the situation graph has exactly two strongly connected components — the empty board's pass cycle and one component holding every other situation — with condensation depth 2, on every board censused: 1x3 through 1x7, 2x2, 2x3, 2x4, 3x3 in crates/superko-graph/tests/scc.rs, and 3x4, 2x6, 1x12 at m·n = 12. 1x2 has three components and 1x1 one | computed | formalizable | C-44 | results/scc-census-1x3-forbid.txt, -2x2-, -3x3-, -3x4-, and -1x2- for the exception; crates/superko-graph/tests/scc.rs. The censused boards do not exhaust 3 <= m·n <= 12 |
| C-46 | The forward-cone prune of C-42 removes at most the two empty situations from an archive that may hold 2·3^(m·n), so it is not a compression; under the suicide-permitting convention it removes none at all | computed | formalizable | C-42, C-45 | results/scc-census-*.txt, field outside-largest; the remove-own contrast is results/scc-census-1x3-remove-own.txt |
| C-47 | A game whose reachable states number 2^poly(n) in the input length n, with names of 2^O(n) bits and a successor computable in time polynomial in a name, is decidable in EXPTIME by backward induction over the state graph | folklore | infra-gap | - | no source held; the step every "bounded effective state implies EXPTIME" argument consumes, this project's included, and it appeared nowhere in docs/, proofs/, notebook/ or lean/ before this row |
| C-48 | APSPACE = EXPTIME, and APTIME(poly) = PSPACE | folklore | infra-gap | - | Chandra–Kozen–Stockmeyer 1981, sought; held only through Hearn 2006's uncited restatement. open-questions.md §5 consumes it |
| C-49 | Undirected vertex geography with unconditional pass edges and a terminal scored by area is solvable in time polynomial in the graph size | open | infra-gap | C-21 | no source; C-21 is a normal-play theorem — last player able to move wins — while a superko Go game never becomes immobile (C-18) and ends by scoring. This is the theorem generalizing C-6 to arbitrary Go, and nobody has it |
| C-50 | A pass made while the board is X archives both of X's situations, so at every state whose history contains the history that pass left, no play recreating X is permitted under either superko rule | proved | formalized:C50_pass_closes_board | - | lean/SuperkoComplexity/Results/C50_Mechanism.lean; pass_seen_both, with now_mem_seen_start and now_mem_seen_step for the invariant it needs. That every later state of a game is such a state is Basic.seen_subset_of_step iterated along the moves, which is not written as a theorem about sequences of moves |
| C-51 | A play never removes the mover's own stones, puts a mover stone at no point but the one played, and adds no opponent stone; so no play recreates the board it was played on, and no two plays by one color with only passes between them do | proved | formalized:C51_play_changes_board | - | lean/SuperkoComplexity/Results/C50_Mechanism.lean; resolve_keeps_mover, resolve_mover_of_ne, resolve_other_of_other and C51_two_plays_by_one_color_change_board. Two plays by different colors can recreate a board: a ko |
| C-52 | Where SSK permits a play that PSK refuses, the board it creates has stood only with the mover to play; the walk back from its last occurrence has odd length, begins and ends with plays by the mover, holds at least three plays, and no pass is made at that board before the separating play; each color's plays remove exactly as many of the other's stones as the other plays, so with three plays and no pass the middle play removes two stones and the outer two remove one between them. Such plays occur in games with no pass, so the separation does not depend on C-18 | proved | formalizable | C-50, C-51 | proofs/C-52.md, by hand: the argument over a walk is checked by no kernel, its local steps are (C-50, C-51); the pass-free 1x3 game is also replayed by crates/superko-graph/tests/containment.rs |
| C-53 | Under Defs.lean's rules no position separates PSK from SSK on any board of at most five points: at every coloring of 1x1, 1x2, 1x3, 1x4, 2x2 and 1x5 and of their transposes, with either color to move, the minimax area difference is the same under both rules. At every root of every board of at most five points in both orientations, under the rules of Defs.lean, the winner under PSK also equals the winner under SSK at every komi floor from -(m*n)-1 to m*n+1, both colors' verdicts computed directly under each rule; floors outside that range are not tested. The rules do differ in legality on those boards: the SSK searches made plays PSK refuses at 4, 98 and 328 roots of 1x4, 2x2 and 1x5. On 1x3 they made none, every such play they generated being cut off, so the 1x3 agreement carries no evidence from the searches that the rules met there; the 1x3 legality gap is exhibited by crates/superko-graph/tests/containment.rs. On 2x3 the sweep is not exhaustive: at 10^7 nodes per search both searches resolve 792 of its 1458 roots, the values agree at all 792, and 184 of them made a play PSK refuses; the other 666 roots, the empty board among them, are unresolved and nothing is stated about them | computed | formalizable | - | results/separate-1x1-forbid.txt through -1x5-forbid.txt and -2x2-forbid.txt, and -2x3-forbid-budget-1e7.txt for 2x3, experiment 005; the line transposes by crates/superko-solve/tests/agreement.rs, which compares every field with no root unresolved; the winners on the boards of at most five points by the same file's the_winners_agree_under_both_rules_on_every_board_of_five_points, ignored in debug, which pins 1608 roots and 76944 verdict searches; the counts of roots making such a play depend on the search order and are lower bounds |
| C-54 | Under the suicide-removing convention, which Defs.lean does not model, the least position separating PSK from SSK in experiment 005's order is X. on 1x2 with Black to move: its minimax area difference is -2 under PSK and 0 under SSK, and at komi floor -2 White wins under PSK and Black under SSK, as at -1 read through C-55. No 1x1 position separates, and none of 1x3, 1x4 or 2x2 does, so under that convention separation is not monotone in the board | computed | formalizable | C-55 | results/separate-1x2-remove-own.txt with its four witness verdicts, each a separate search; -1x1-, -1x3-, -1x4- and -2x2-remove-own.txt for the absences; traced by hand in notebook/2026-09-12-c17-separation-search.md; formalizing it needs a suicide-removing rule, which Defs.lean does not have |
| C-55 | At every position of 1x1, 1x2, 1x3, 1x4 and 2x2 as the root under either suicide convention, and of 2x1, 3x1, 4x1, 1x5 and 5x1 as the root under the suicide-forbidding convention, with either color to move, under either superko rule, exactly one color wins at each komi floor k from -(m*n)-1 to m*n+1, and Black wins at k exactly when the minimax area difference exceeds k | computed | formalizable | - | crates/superko-solve/tests/agreement.rs, verdicts_are_determined_and_track_the_value for the boards under both conventions and the_winners_agree_under_both_rules_on_every_board_of_five_points, ignored in debug, for the suicide-forbidding convention on every board of at most five points, each computing the winners and the value by separate recursions; the general statement is a short induction over WinsFor and is not written |

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
None has occurred as far as the solver reaches: `superko-solve` reproduces the
entries for n ≤ 6 exactly, under `Defs.lean`'s rule, whose suicide convention is
the one the transcribed header names (`computed`,
`crates/superko-solve/tests/published.rs`). The row stays `cited`. Reproduction
is evidence that this workspace computes what the table records, not that
either describes Go. 1×7 and 1×8 do not resolve within the test's budget of
4 × 10⁷ nodes, and the disputed 1×9 of C-11 is not attempted.


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

### C-41 to C-46 — what the value sees of the archive, and why it does not help

These rows are one result with two halves, and the halves point opposite ways.

The proved half is
[`../lean/SuperkoComplexity/Compress.lean`](../lean/SuperkoComplexity/Compress.lean).
`winsFor_transfer` (C-41) is the general schema: a relation that preserves the
situation, the pass counter and legality, and survives `step`, preserves who
wins. It is parametric in the repetition rule, so the positional twin is proved
rather than asserted, and every later compression statement factors through it.
`winsFor_seen_inter_cone` (C-42) is the instance the upper bound wanted: the
value reads the archive only inside the forward cone — the situations still
reachable by moves the board permits, with the repetition rule out of the way —
so entries outside it may be dropped and unreachable situations may be added.
`decideWins_iff_winsFor` (C-43) lifts C-29 off the root, which every mid-game
statement needs and which C-29 alone does not give. `Defs.lean` is untouched.

The computed half says the prune is worth nothing. C-45's census, on every
board it reaches — 1×3 through 1×7, 2×2, 2×3, 2×4 and 3×3 in the test suite,
and 3×4, 2×6 and 1×12 at the top of the range, none of them exhaustive over
the range: the legal part of the situation graph has exactly two
components — the empty board's pass 2-cycle, and one component holding every
other situation — and the condensation is two deep. So the cone of any
non-empty situation is the whole of that giant component, and C-46 is the
consequence: the prune removes at most the two empty situations from an archive
that may hold `2 · 3^(m·n)`.

C-44 is why the two components are two rather than one, and it is a theorem
rather than a measurement: `resolve` clears `c.other` only, so the played stone
survives (`resolve_self`) and no play produces the empty board. The census
corroborates it in the sharpest available way — under the suicide-permitting
convention, which has no counterpart in `Defs.lean` and where a self-capture
*can* take the played stone back off, the graph collapses to a single component
on every board censused and the prune removes nothing at all.

**What this refutes.** Not C-42, which is exact. The subset-of-the-archive
route to a smaller state: pruning to a forward cone, to a strongly connected
component, to a reachability ball. Every such scheme is bounded above by C-46's
two entries, and the reason is structural rather than incidental — Go's
situation graph is one mutually reachable mass as soon as a stone is on the
board. A summary that is *not* a subset of the archive — a hash, a quotient
representative, an automaton state — is untouched by this and remains the only
live form. So does the question of C-15's index, which is about how many
archives are distinguishable, not about which subset of one suffices.

**What it does not bear on.** C-14, in either direction, for the reason
[`open-questions.md`](open-questions.md) §4 now states: a bound on one summary
scheme bounds no complexity class. The census is `computed` at `m·n ≤ 12` and
is terminal there — no certificate turns a census of other boards into a
statement about all of them, and the two-component finding would become a
theorem only by proof, which nobody has attempted.

### C-47, C-48 — the steps this project was consuming silently

Neither is new mathematics and neither is in doubt. They are here because the
ledger's rule is that a step a proof consumes has a row, and these two were
being spent without one.

C-47 is the backward-induction meta-theorem — bounded effective state implies
EXPTIME — which is the second half of every argument of the form "the history
compresses, therefore C-14". Before this row it appeared nowhere in `docs/`,
`proofs/`, `notebook/` or `lean/`, while the shape of the argument appeared
repeatedly. C-48 is APSPACE = EXPTIME, which [`open-questions.md`](open-questions.md)
§5 has been quoting; Chandra, Kozen and Stockmeyer are `sought` in
[`../references/README.md`](../references/README.md) and the restatement this
project has read is Hearn's, uncited.

Both are `folklore` on this ledger's vocabulary rather than `cited`, which is
the vocabulary working as intended and not a slight: the sources exist and are
correct, and this project has not read them. Obtaining Chandra–Kozen–Stockmeyer
moves C-48 to `cited` and costs an afternoon.

### C-49 — the theorem C-6 would need, which nobody has

C-21 — undirected vertex geography is polynomial — is a **normal-play**
theorem: the player unable to move loses, and the algorithm is a maximum
matching. Neither half survives the transfer to Go. A superko game never
becomes immobile, because a pass is always legal (C-18), and it ends by area
scoring rather than by immobility, so the terminal condition the matching
argument reads is absent.

This is a third obstruction to generalizing C-6, alongside the two
[`open-questions.md`](open-questions.md) §5 already names. It is recorded
because C-6 is the whole reason to believe C-14, and an obstacle to C-6 that
lives only in a reader's head is the kind of thing this ledger exists to stop.
Robson's construction engineers a normal-play-like payoff; arbitrary Go has
none. Whether the matching argument survives the addition of free pass edges
and a scored terminal is open, and is the concrete form of the question.

### C-17 — what "game value" means, and where the search stands

`Defs.lean` defines no game value: `Superko.WinsFor` is a two-valued game at a
fixed komi. This project reads C-17's "game value" as the **minimax area
difference** — Black's area less White's at the finished game, Black
maximizing and White minimizing. That quantity belongs to `superko-solve`, and
it reaches `WinsFor` only through the threshold agreement of C-55, which is
`computed` and not proved. A witness for C-17 is therefore recorded as a pair
of verdicts at one named komi floor. That is the quantity `Defs.lean` defines
and the one a kernel evaluation of `Superko.decideWins` can check; the value
locates candidates and is not the evidence. The absence of a witness and the
minimality of one are statements about values, and reach winners only through
C-55, with one exception: under `Defs.lean`'s rules, on every board of at most
five points in both orientations, the winners under the two rules are compared
directly at every komi floor from −(m·n)−1 to m·n+1 (C-53).

Minimal is taken in experiment 005's order: board area, then the number of
stones, then the position code, then Black to move before White. Under `Defs.lean`'s rules there is no witness on any board of at
most five points (C-53). Under the suicide-removing convention the least
witness is on 1×2 (C-54), and under that convention separation is not
monotone in the board: 1×2 separates while 1×3, 1×4 and 2×2 do not. Under
`Defs.lean`'s rules nothing is known either way, so the absence of a witness on
the boards swept bounds nothing about the boards above them. Of the boards of
six points only 2×3 is swept, and only under a budget of 10⁷ nodes per search:
none of the 792 roots it resolves separates, and the 666 it leaves unresolved
include the empty board, so no minimum on 2×3 is shown (C-53). 1×6, 3×2 and
6×1 are not swept.

### C-50 to C-52 — what separates the rules

The two rules differ exactly at a play whose board has stood with the mover to
play and never with the opponent. C-50 and C-51 are the two local facts about
`resolve` and `step` that decide how a game reaches such a play, and C-52
assembles them over the walk back from the board's last occurrence: odd length,
at least three plays, the mover's first and last, and no pass at that board
before the play.

The pass exemption is not the parity resource separating the rules. A pass at
the recurring board closes it to both rules; separating plays occur in games
with no pass; a pass at another board can sit inside a walk.

C-52 is `proved` with formalization `formalizable`: a hand proof in
[`../proofs/C-52.md`](../proofs/C-52.md), and the first `proved` row no kernel
has checked. Its local steps are kernel-checked. The argument over a walk
needs a notion of a game as a sequence of states, which the development does
not have.

### C-53, C-54 — the separation sweep

`superko separate` solves every root of a board under both rules: every
coloring `Superko.Position` admits, with each color to move, since
`formal-model.md` §5 takes a position as the root of play. A null result is
worth only as much as the searched trees differed, so each body counts the
roots whose SSK search made a play PSK would refuse. That count depends on the
order alpha-beta visits moves in, and is a lower bound. On a sweep run under a node
budget it must be read over the resolved roots alone, because the roots with
repetition cycles in them are exactly the expensive ones.

2×3 is the first board swept under a budget, 10⁷ nodes per search. It resolves
792 of its 1458 roots, and 184 of the 792 made a play PSK refuses, so the
agreement recorded there is not vacuous. The 666 unresolved roots all made
one: the budget stopped exactly the searches in which the rules met, as on
the earlier budgeted runs. A separating root among them is not excluded.

The sweep compares the value rather than the winner because one search answers
for every komi at once. The winners are compared by a separate test: at every
root of every board of at most five points in both orientations, under
`Defs.lean`'s rules, `the_winners_agree_under_both_rules_on_every_board_of_five_points`
in `crates/superko-solve/tests/agreement.rs` computes both colors' verdicts
under each rule at every komi floor from −(m·n)−1 to m·n+1, and the PSK winner
equals the SSK winner at every one (`computed`). On those boards, 1×5 and 5×1
included, C-53 is therefore a statement about winners in that floor range as
well as about values, and does not reach winners only through C-55. Floors
outside the range are not tested. On 2×3 only values are compared.

### C-55 — the threshold agreement

`superko-solve` computes the minimax area difference and `WinsFor` by separate
recursions, neither derived from the other, and its test holds them together
at every komi floor from one below the lowest possible score to one above the
highest. The general statement is a short induction over the game tree: at a
finished game the leaf test is `winnerZ`, the mover's disjunction is a maximum,
and the waiter's conjunction is a minimum. It is not written in Lean. Nothing
`proved` rests on it, and no C-17 witness may. The minimality of C-54's witness
is a statement about values, and reaches winners only through it. The absence
of separation C-53 records now reaches winners directly on the boards the test
`the_winners_agree_under_both_rules_on_every_board_of_five_points` covers —
every board of at most five points in both orientations, suicide forbidden,
at every komi floor of the range above — and not only through this agreement;
on 2×3 it is a statement about values.
