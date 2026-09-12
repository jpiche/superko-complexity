# 2026-09-12 — the search for a position separating PSK from SSK

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`). The main session wrote the solver,
ran every sweep, traced the 1×2 witness and wrote the records. A workflow of
Claude Opus 5 subagents did the rest: one scout wrote the structural analysis
of the legality gap (claims M-1 to M-19 below), a second audited the Lean
surface, a third read the held sources for novelty, a fourth reported the
workspace's conventions, and three adversarial verifiers per claim attacked
M-1 to M-14. The workflow was stopped by the machine crash recorded below,
before M-15 to M-19 were verified.

The target was C-17, a minimal position whose value differs between positional
and situational superko. Experiment 005 holds the question, the minimality
order and the falsification table, all written before the sweep ran.

## The solver

`superko-solve` was empty. It now holds two engines. `naive` recurses over
`superko_rules::reference::State`, clones per node and prunes nothing.
`search` runs alpha-beta over the transition table, making and unmaking moves
on the dense archive. Both answer two questions by separate recursions: the
minimax area difference, and `WinsFor` at a komi floor by `decideWins`'s
branch structure. There is no transposition table. One keyed on less than the
whole archive is the graph-history-interaction error; one keyed on the whole
state would be sound, and is the obvious next step for reach.

Validation, all `computed`, `cargo test -p superko-solve --release`:

- The naive and fast engines agree on every value and every verdict at every
  root of every board of at most three points, under both rules and both
  suicide conventions.
- On 1×1, 1×2, 1×3, 1×4 and 2×2, exactly one color wins at every komi floor
  from −(m·n)−1 to m·n+1, and Black wins exactly when the value exceeds it
  (C-55).
- The published 1×n values under PSK reproduce for n ≤ 6: 0, 0, 3, 4, 0, 1.
  The fixture's header names no suicide, which is `Defs.lean`'s suicide
  convention; nothing checks the table's other rule choices, and the fixture is
  transcribed second-hand, unverified against its source.

## Move ordering, measured

`superko solve --board 1x5 --rule psk --suicide forbid --root .....`, nodes
visited from the empty 1×5 board:

| order | nodes |
|---|---|
| `all_moves` unsorted: the pass, then points row-major | 35 121 |
| every move sorted by the successor's area difference, the pass included | 63 917 |
| the pass in place, then the plays sorted by that difference | 2 115 |

Sorting the pass in with the plays is worse than not sorting, because a play
raising the mover's own area always sorts ahead of standing still. And the
pass is what gives alpha-beta its cheapest bound — the opponent's pass ends
the game — so it has to come first. Putting it last is catastrophic: the
unsorted reversed order could not finish 2×2 roots at a budget of 2 × 10⁷
nodes. Under the adopted order the empty 1×6 board resolves under PSK to 1 in
27 925 122 nodes, which is what brought the published-table test from five
rows to six. Under the unsorted order a 1×6 sweep at a budget of 10⁷ nodes per
search resolved 156 of its 1458 roots in 260 s.

## Two mistakes in the sweep design, caught

**Transposition invariance off a line.** I asserted that a board's sweep
equals its transpose's field for field. That holds on a line, where the
position code is the one coordinate either way. It fails on rectangles: on
2×3 a point's index is `3·row + col` and on 3×2 it is `2·row + col`, so
transposing permutes codes and move orders, and alpha-beta explores
differently. At a budget of 2 × 10⁵ nodes the two sweeps resolved 302 and 380
roots respectively (unsorted order). The test now covers lines only.

**Counting the rules' meeting over unresolved roots.** The sweep counts the
roots whose SSK search met a play PSK refuses, so that a null result can show
the trees it searched actually differed. On 2×3 with `--min-stones 4` at a
budget of 3 × 10⁶ nodes (unsorted order), the sweep resolved 278 roots, left
714 unresolved and skipped 466, met 52 982 696 such plays, and counted 714
roots meeting one: exactly the unresolved ones. **None of the resolved roots
met one**, so that sweep's "no separation" was vacuous. The roots with
repetition cycles are the expensive roots, so a budget selects against exactly
what the sweep looks for. The body now carries `resolved-with-ssk-only`.

## What the sweep found

Under `Defs.lean`'s rules no root separates on any board of at most five
points (C-53); the witnesses are `results/separate-*-forbid.txt`. The SSK
searches make plays PSK refuses at 4, 98 and 328 roots of 1×4, 2×2 and 1×5
under the adopted order, counted when made; on 1×3 they make none.

The hypothesis of experiment 005 put the smallest separating board at
`m · n` between 4 and 6. Four and five hold none.

Under suicide removal the least witness is `X.` on 1×2, Black to move (C-54).
Traced by hand, with points 0 and 1 and komi floor −2, so Black needs an area
difference of at least −1:

- Black passing first is wrong: it archives `⟨X., White⟩`. White then plays 1,
  removing the Black stone, to `.O`, and Black's retake at 0 would recreate
  `⟨X., White⟩`, which both rules refuse. White wins at `.O`, worth −2.
- So Black plays 1, and the two Black stones remove themselves: `..` with
  White to play, archive `{⟨X., Black⟩, ⟨.., White⟩}`.
- White plays 0 to `O.`: Black plays 1, removing it, to `.X`, worth +2, and
  White's retake would recreate `⟨O., Black⟩`, refused by both. Black wins.
  White's other play is 1, to `.O`, worth −2, with Black to play.
- White can also pass at `..`. Black passes, and the game ends at `..`, worth 0,
  which Black wins at floor −2. That is where the SSK value of 0 comes from:
  under SSK both of White's plays end at +2, as the next step shows, so White
  passes. Under PSK White does better with the play at 1.
- Black's retake at 0 removes the White stone and gives `⟨X., White⟩`. PSK
  refuses it, since the board `X.` is the root. SSK permits it, since the root
  had Black to play. Under SSK Black retakes; White cannot answer at 1, which
  recreates `⟨.O, Black⟩`; both pass, and the game ends at +2 — so under SSK
  White prefers the pass above. Under PSK Black can only pass, White passes,
  and White wins at −2, which is the PSK value.

The parity comes from the self-capture: one play changes the board without
adding a stone. That is why the convention separates the rules on two points,
where under `Defs.lean`'s rules the sweep met no legality gap at all
(`ssk-only-plays=0` on 1×2). That no gap exists there is the scout's M-11,
which one verifier confirmed over fully closed state spaces in its own
untrusted mirror; it is not proved here. The convention does not separate 1×3,
1×4 or 2×2, so under it separation is not monotone in the board. Under
`Defs.lean`'s rules nothing is known either way, and C-53's null result bounds
nothing at six because no monotonicity theorem exists.

## The structural analysis, and what the verifiers did to it

The scout's claims, with the verifiers' votes (refuting votes out of three):

| claim | gist | refuted |
|---|---|---|
| M-1 | the exact gap condition | 0 |
| M-2 | the archive-and-parity invariant, as stated | 3 |
| M-3 | the parity characterization, as stated | 3 |
| M-4 | a play changes the board | 0 |
| M-5 | a pass at the recurring board closes it | 0 |
| M-6 | the walk's first and last moves are the mover's plays | 0 |
| M-7 | stone-count identities; no return after one or two plays; three plays minimum | 3 |
| M-8 | a sharp bound on passes inside the walk | 0 |
| M-9 | the forced capture profile of the three-play walk | 1 |
| M-10 | the four-plays-one-pass walk | 3 |
| M-11 | no gap on boards of two points | 1 |
| M-12 | a pass-free five-move 1×3 line | 0 |
| M-13 | the repository's stated mechanism is wrong | 1 |
| M-14 | legality gap necessary but not sufficient; the comparison not monotone | 1 of 1 |

What the refutations established:

- **M-7's identities survive and its "never after exactly two plays" does not.**
  A ko returns a board after two plays, one by each color: on 1×4 from `.XO.`
  with Black to play, Black 3 then White 2. The minimum of three plays holds
  only for a return that swaps the color to play, and it needs the parity fact
  that the walk's first and last moves are both the mover's, which the
  identities alone do not supply. `proofs/C-52.md` is written that way.
- **M-13's correction survives and its "the parity resource uses no pass at all"
  does not.** A pass at a board other than the recurring one can supply the
  parity. From the empty 2×2 board: Black (0,0), White (0,1), Black (1,0),
  White passes, Black (1,1), White (0,1) removing three stones, Black (0,0)
  brings `XO/..` back with White to play. I traced it by hand. What does hold
  is narrower: a pass at the recurring board closes it (C-50), and the
  separation needs no pass (the 1×3 line).
- M-10's 2×2 witness was mislabeled, with colors that do not alternate; its
  arithmetic survives. M-9 carried a false "rotation" sub-lemma. M-14's
  non-monotonicity was asserted without a separation of either sign in hand,
  and its "SSK enlarges the move set at every state" fails at every root.
- The name "sending two, returning one" is held from no source.

Applied: C-50 and C-51 are kernel-checked; C-52 is written by hand in
`proofs/`; `formal-model.md` §OPEN-1 is corrected; the containment test's
tally now skips ended states (M-19), and its claim to hold the shallowest
line is withdrawn in favor of M-12's line, which is now a test. Not applied:
the `SSK` docstring in `Defs.lean`, which carries the wrong mechanism, because
editing that file invalidates every Lean oracle fixture's recorded digest.

M-15 to M-19 were never verified. M-16 in particular is a useful heuristic
and no more: the separating walk ends at the situation a pass at its start
would have produced, differing only in the pass counter and the archive, so
the extra plays SSK allows buy a tempo with the counter reset.

## The crash

At 14:17:47 macOS raised a JetsamEvent
(`/Library/Logs/DiagnosticReports/JetsamEvent-2026-09-12-141747.ips`). Two
`python3.14` processes held 77.93 GiB and 57.27 GiB resident on a 36 GiB
machine. They were scratch transliterations of `Defs.lean` that verifier
agents had written for themselves in the session scratchpad and used for
memoized game-tree searches. WindowServer hit its watchdog and the machine
restarted. A `lake build` running at the time was killed as collateral, and
the exhaustive 1×6 sweep was lost unfinished. One orphaned verifier script
survived the restart and was killed by hand.

Neither the Rust sweep nor Lean was the cause. The Rust solver's memory is the
transition table and a `2·3^(m·n)`-bit archive: megabytes. The Lean build of
one results file peaks near 1.5 GB. The fault was in the workflow's brief,
which told verifiers to check claims concretely and set no bound on how.
Agents that may run code must be told not to, or given a ceiling.

## The wrap-up review

Three read-only reviewers read everything the session changed: one on the
records, one on the proof and Lean statements, and one on the code. They were
forbidden to run anything. They returned 43 findings. Each was weighed against
the definitions or the code before anything changed, and nearly all held.

- **C-52 claimed too much.** It said no pass is made at the recurring board
  anywhere in the game, but a pass there after the separating play is legal.
  The claim now stops at the separating play and states that the earlier
  moves are legal.
- **C-52's local steps were not all kernel-checked**, though the records said
  they were. The two per-play facts its stone count uses — a play puts a mover
  stone nowhere but where it is played, and adds no opponent stone — are now
  `resolve_mover_of_ne` and `resolve_other_of_other`.
- **The non-vacuity counter counted generated plays, not searched ones.** A
  play a cutoff pruned was counted. Counted when made, the resolved roots
  meeting such a play fall from 4, 8, 98 and 354 to 0, 4, 98 and 328 on 1×3,
  1×4, 2×2 and 1×5. The 1×3 searches never enter a play PSK refuses. The value
  agreement is unaffected, since alpha-beta's cutoffs show those plays do not
  change the value, but on 1×3 the counter no longer shows the rules meeting.
- **The liberties minimum could be printed unconditional when it was not**,
  under a budget. No committed body was affected; that minimum now has its own
  flag.
- **Two tests could pass vacuously.** The transposition test compared eight
  fields under a budget and never required that any root resolved. It now
  compares every field and requires none unresolved, which fails for 1×5 with
  suicide removed, so that board is left out. The order test's floor of 800
  let 160 of its 960 searches go unchecked. It now pins 816.
- **Wording.** Non-monotonicity was stated without its convention. Experiment
  005's hypothesis was called failed, though the data only narrow it. "Out of
  reach" was stated without its budget. C-24's table was said to share
  `Defs.lean`'s rule, when only its suicide convention is known to. History
  had leaked into the core docs. And the crate docs claimed nothing rested on
  the threshold agreement, when every null result does.

## Where the next session picks up

1. **The six-point boards.** 1×6, 2×3, 3×2 and 6×1, exhaustively, one at a
   time. The empty 1×6 root takes 2.8 × 10⁷ nodes under PSK; whether the SSK
   searches and the 2×3 roots are comparable is unmeasured.
2. **The `Defs.lean` docstring**, together with `tools/gen-oracle.sh` to
   regenerate the fixtures, and the `defs-blob` of every results header, which
   records that file's hash.
3. **Reach.** A transposition table keyed on the whole state — position, color
   to play, pass count and archive — is sound and has not been tried.
4. **Constructed candidates.** Exhaustive search will not reach the boards where
   sending two, returning one sits beside an independent region it could
   decide. Positions with many stones on 3×3 to 3×4 have few empty points and
   small trees, and are where a hand-built candidate would be checked. That
   this is where a separation lives is a guess.
5. **Formalizing C-52**, which needs a notion of a game as a sequence of states.
6. **CI time.** Under the debug profile CI uses, the `superko-solve` agreement
   tests took 448 s and the workspace suite 495 s before the wrap-up review's
   test changes, and 103 s and 154 s after them, against a few seconds before
   this session. Either an `opt-level` override for
   `superko-rules` and `superko-solve` in the dev profile, or moving the
   heaviest comparisons behind `--ignored`, would bring it back down. Neither is
   done.
