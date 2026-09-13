# 2026-09-13 — why the six-point witnesses separate the rules

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`), alone: it added the line
searches to the solver, wrote the scratch walker, ran the walks and the reach
check, and wrote these records and the proof of C-58. The Lean certificate
plan it drafted earlier in the session was shelved before this work, unrun,
in gitignored `data/`.

The maintainer asked, after C-56 was recorded, why the separating positions
exist rather than how to certify them: the suspicion had been that the rules
agree in actual play, so a separation at six points is worth understanding
before it is worth proving. Everything below is `computed` except where a
status says otherwise.

## The instrument

`Solver::solve_line` and `Solver::decide_line` (commit in this branch): seat
the root as `solve_root` does, make each move of a line with its situation
archived, search from the state that leaves, unmake. A scratch program in
`data/pv/` (gitignored, a cargo package with path dependencies on the two
crates) walks a line by re-solving after each candidate move: a value walk, a
verdict walk, a walk that follows the SSK winner into the part of the tree
where PSK's verdict differs until the winner makes a play PSK refuses, and a
breadth-first search for the shortest games from the empty board reaching a
situation, each then solved under both rules with its history. The walker is
not promoted; the lines it found are replayed by
`crates/superko-solve/tests/lines.rs`, which is the reproducible record.

## 1×6: `X.X.X.`, Black to move, floor 1

Values, plain search, no symmetry. The SSK main line is short: Black 2 (all
points numbered from 1), White has nothing better than a pass, Black passes,
+6. The difference from PSK lies in the refutation of White's resistance.
Following the SSK winner into the part of the tree where PSK disagrees:

| move | board after | note |
|---|---|---|
| Black 2 | `XXX.X.` | |
| White 4 | `...OX.` | captures three |
| Black 2 | `.X.OX.` | |
| White 6 | `.X.O.O` | |
| Black 3 | `.XXO.O` | |
| White 1 | `O..O.O` | captures two |
| Black 3 | `O.XO.O` | |
| White 2 | `OO.O.O` | captures one |
| Black 5 | `OO.OX.` | |
| White pass | `OO.OX.` | |
| Black 3 | `..X.X.` | captures three |
| White 2 | `.OX.X.` | |
| **Black 1** | **`X.X.X.`** | **SSK permits, PSK refuses** |

The board Black 1 recreates is the root, which has stood once, with Black to
move; now White is to move. White's only play there, 2, would recreate
`.OX.X.` with Black to move, the situation just left, refused under both
rules; 4 and 6 are suicides. White passes, Black passes, +6. Under PSK at
`.OX.X.` Black must play 4, and the value is 1. Values along the line under
SSK are 6 at every prefix; under PSK 1 at every prefix through move 12.

## 2×3: `OOO/.X.`, White to move, floor −1

The SSK value of this root is unresolved at 10⁸ (C-56), so the walk is by
verdicts at floor −1, where White wins under SSK and Black under PSK. The
verdict searches along the line are small: 1 713 nodes for the full line
under SSK, 420 713 for its first ten moves under PSK. Following White into
the part of the tree where PSK's verdict differs:

| move | board after | note |
|---|---|---|
| White pass | `OOO/.X.` | |
| Black (1,0) | `OOO/XX.` | |
| White (1,2) | `OOO/..O` | captures two |
| Black (1,1) | `OOO/.XO` | |
| White pass | `OOO/.XO` | |
| Black (1,0) | `.../XX.` | captures four |
| White (0,1) | `.O./XX.` | |
| Black pass | `.O./XX.` | |
| White (0,2) | `.OO/XX.` | |
| Black pass | `.OO/XX.` | a White pass now ends the game at 0, Black's win at floor −1 |
| **White (0,0)** | **`OOO/XX.`** | **SSK permits, PSK refuses** |

The board White (0,0) recreates stood after the second move, with White to
move; now Black is to move. Under PSK White's choices at `.OO/XX.` are the
pass, which ends the game at 0, and (1,2), which loses. The continuation
after the return was not traced by hand; the verdict search says White wins
from there.

## The reading

Both witnesses use one device: a play that brings back a board which has
stood only with the winner to move, now with the loser to move. That is
C-52's odd walk read as a resource, and the resource is a **delayed pass**.
Passing at the board when it stood would have given the same situation with
an empty archive and closed the board to both rules (C-50). SSK lets the
player obtain it later, once, when the archive has closed the opponent's
replies — on 1×6 White's recapture, on 2×3 the game-ending pass. PSK visits a
board once and has no such move.

Both need the history encoding (C) starts with: the root situation alone.

- **1×6.** In any game, a board carrying stones all of the mover's color
  follows a pass by the opponent (`Basic.resolve_self`: a play leaves the
  mover's stone). So `X.X.X.` with Black to move is reached only after a
  White pass, the pass count is one, and Black ends the game at +6 by
  passing, under either rule. This is C-58, `proved` by hand in
  `proofs/C-58.md`. The six shortest reaching games all give value 6 under
  both rules in 29 nodes.
- **2×3.** `OOO/.X.` with White to move is reached by a Black play at (1,1),
  no pass needed. The six shortest reaching games under SSK, all of seven
  moves with three passes, give under PSK and SSK alike: 1, −2, unresolved at
  10⁸ with agreeing verdicts at floor −1, −2, −2, −2. In the four games where
  the board had stood with Black to move the return is closed and White has
  −2 anyway; in the two where it had not, Black has 1 or the win. Six
  histories are evidence, not proof; `tests/lines.rs` replays two of them in
  an ignored release test.

So C-56 is a fact about encoding (C). The question the maintainer's suspicion
is about — whether a game's history can carry a separation — is experiment
006, pre-registered and not run. C-1 carries the pressure note.

On C-12 the reading gives no result. The difference between the rules is one
bit per board, two situations against one, which is what a reduction either
way must simulate; that sentence is in `docs/open-questions.md` §3 as a
reading and nothing more.

## What is not established

- The 2×3 continuation after White's return, beyond the verdict search.
- Anything about games longer than the shortest reaching ones, or about
  boards other than 1×6 and 2×3.
- The other three separating roots each sweep counted, not examined.
- That a live separation does not exist: the hypothesis of 006, unrun.
- No kernel has checked any of it; C-58's argument runs over a game as a
  sequence, which the development does not define.
