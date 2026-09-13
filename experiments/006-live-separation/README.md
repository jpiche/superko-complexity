# 006 — Does a game's history ever carry a separation of the superko rules?

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`): wrote this file from the
maintainer's decisions after the walk recorded in
`notebook/2026-09-13-why-the-witnesses-separate.md`.
**Status:** planned
**Opened:** 2026-09-13
**Claims touched:** C-1, C-17, C-56, C-58

## Question

Is there a state reached by a game from the empty board — a position, a color
to move, and the history that game archived — from which the winner at some
komi floor differs between positional and situational superko?

Experiment 005 asked the question under encoding (C), the position as the
root of play with its own situation alone archived, and found two six-point
positions that separate (C-56, `computed`). Both win by returning to a board
the fresh history leaves open, and the 1×6 one, reached by any game, is won
by Black under either rule (C-58, `proved` by hand). So C-56 says what the
rules do to an arbitrary root and nothing yet about a game in progress, which
is what the KGS anecdotes behind C-17 report and what C-1 asks encoding (C)
to be faithful to. Call a state reached by a game whose verdicts differ a
**live separation**.

### Terms fixed before the run

A **game** is a sequence of moves from `Superko.start empty black` each
permitted by SSK at a state not ended; SSK is the weaker rule, so every PSK
game is one. A **state** is what the game leaves: board, color to move, pass
count and archive. The **PSK reading** of a state archives the boards of the
situations the game archived, and the PSK verdicts are computed with that
archive; the SSK reading archives the situations. This is what
`Solver::solve_line` and `Solver::decide_line` compute when handed the game as
a line, and it is the comparison `crates/superko-solve/tests/lines.rs` makes
for the games it replays.

The comparison is over games PSK permits throughout. A game PSK refuses
somewhere has no PSK reading; such games are counted and excluded, and the
count is reported.

**Depth** is the length of the game. The sweep is over every state reached by
a game of length at most `k`, deduplicated by state, in the order the search
visits them. `k` is a budget, not a claim: a null result at depth `k` says
nothing about longer games.

## Hypothesis

**On 1×6 and 2×3 no live separation exists at any depth the sweep reaches.**

The reason, such as it is: the device the witnesses use needs a board that has
stood with the winner to move and never with the loser to move, at a moment the
archive has closed the loser's replies. A history that reaches a position also
fills the archive around it, and the six shortest games reaching the 2×3
witness leave the rules agreeing (`computed`, six histories). That is a
guess from two positions and is labeled one. A live separation on a larger
board is not excluded by anything and is not this experiment's question.

## Falsification

| # | result | consequence |
|---|---|---|
| 1 | a state reached by a game of length at most `k` on 1×6 or 2×3 whose verdict at some floor differs between the readings | the hypothesis is refuted; a `computed` row records the state, its game, the floor and both verdicts, with a plain `solve`-style witness in `results/`; C-1's pressure note is rewritten, and the 2×3 or 1×6 position of C-56 is or is not the one, as found |
| 2 | no such state, every state resolved, through depth `k` on both boards | the hypothesis stands to depth `k`; a `computed` row records the boards, the depth, the state counts and the games excluded for a PSK refusal; nothing about longer games |
| 3 | unresolved states at the budget | reported by count with the least unresolved state; a null result with unresolved states is bounded by them, as 005's row 5 was |
| 4 | the reach check of `tests/lines.rs` disagrees with the sweep on a game it replays | the instrument is wrong; nothing is recorded until the disagreement is explained |
| 5 | a separation whose winning line uses no play PSK refuses | the instrument is wrong: the readings can differ only through such a play, so the state is re-solved by the naive engine where it reaches and the discrepancy is recorded |

## Method

1. A `live-separate` subcommand of `superko-cli`, added before the run:
   `--board MxN --suicide forbid --depth k [--budget N] [--threads N]`. It
   enumerates the states reached by SSK games of length at most `k` from the
   empty board, deduplicated by state, drops the ended ones, and for each
   state that PSK permits throughout computes both colors' verdicts under
   both readings at every komi floor from `-(m·n)-1` to `m·n+1`, by
   `decide_line` with the game as the line. It reports the state count, the
   games excluded, the unresolved states with the least one, and every state
   whose verdicts differ, least first, in 005's order extended by depth.
   Symmetry is off. The body is a results body; the header is written by hand.
2. Its test: on 1×3 and 2×2 to depth 6, every verdict equals the naive
   engine's on the stepped reference state, both readings.
3. Runs, in this order, each on its own, maintainer-started: 1×6 to depth 6,
   2×3 to depth 6, then depth 8 on each at a budget of 10⁷ nodes per verdict,
   then depth 10 if the depth-8 runs finish in under an hour. Standard output
   to `data/`, promoted to `results/` when a row cites it.
4. The counts of states per depth are recorded whether or not anything
   separates.

Nothing here is run by an agent; the depth-6 runs are the first, and the
maintainer starts each.

## Result

Not run.

## Verdict

Not reached.
