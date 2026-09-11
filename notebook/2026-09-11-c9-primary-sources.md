# 2026-09-11 — C-9's primary sources, read

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`)

The question was what it would take to move C-9, the 2×2 game count under
positional superko, from a transcribed citation toward something this
project holds. The first step turned out to be reading the sources, which
until today were `sought`. Two are now held: the 2016 revision of
Tromp–Farnebäck's *Combinatorics of Go*, saved under `data/`, and Tromp's
*Solving 2x2 go* page with its C source `2x2.c`. This entry records what
they say against `Defs.lean`, one scratch computation, and a conclusion
about formalizability that changes what "next" means.

## What the sources say

- **A game is a simple path plus forced passes.** Lemma 2 puts games in
  bijection with simple paths from the empty position in the game graph:
  insert one pass before each out-of-turn move and two at the end. Since two
  consecutive passes end the game, a lone pass can only precede an
  out-of-turn move, so the bijection covers every legal alternating
  sequence that ends at the second consecutive pass. That is the notion
  `Defs.lean` induces: `Ended` at two passes, passes always legal.
- **Tromp's program agrees on every other choice.** `2x2.c` plays area
  rules under PSK; it counts a game in `score()`, reached exactly at the
  second consecutive pass, so the immediate pass-pass game counts (it is
  the single 1×1 game); passes are exempt from the repetition check and add
  nothing to the history; captures resolve before the suicide test; and it
  rejects both suicide shapes 2×2 admits — a fourth stone onto a board the
  opponent has left, and a stone between two enemy stones on a diagonal.
  Its comment reads "suicide is not possible in this case". One caveat: the
  program's `ngames` is a full count only with `CUT 0`, plain minimax; with
  alpha-beta cutting on, it counts the games the search visited. The paper
  says the number "was recently independently verified" and names nobody.
- **The paper's ruleset permits multi-stone suicide.** Its move rule
  empties the mover's own libertyless strings after the capture step;
  `Defs.lean` forbids the move instead (AGA Rule 5). Single-stone suicide
  is excluded on both sides, by PSK there (a self-loop) and by the suicide
  rule here. So the counts in Table 7 and the counts `Defs.lean` induces
  can differ only through a multi-stone suicide that does not repeat a
  position.
- **The root counts as visited.** Lemma 2's paths start at the empty
  position, so a return to it repeats. `2x2.c` never marks the root
  visited, and does not need to: no play on 2×2 yields the empty board once
  the fourth-stone suicide is rejected. `start` seeds the archive with the
  root, matching the paper.

## Where the suicide divergence can bite — reasoned, not computed

By hand. On 2×2, a play that joins an own stone into a two-stone chain
without liberties has both opposite-row points white, and that white chain's
only liberties were the two points now black, so it is captured first; a
three-stone chain leaves one point, empty or a white stone whose only
neighbors are black; a four-stone chain is the fourth-stone case above, a
return to the root. So no multi-stone suicide arises on 2×2, which is what
Tromp's comment claims. On a line, the same argument runs along the flanking
enemy chains: each has its far end at the edge (captured), at an empty point
(a liberty for it, and the new chain dies), or at a stone. The first shape
where both flanks survive is `. W [B] B W .` on 1×6, which needs six points.
So the divergence should not reach 1×4 or 2×2. This is a hand argument
about a definition the project exists to distrust; experiment 004 counts
both ways rather than believing it.

## A scratch count

A Python enumerator of the `Defs.lean` semantics — chains by flood fill,
capture before the suicide test, PSK with the root archived, passes always
legal, end at two passes — memoized on (board, archive, passes, player)
gave 1, 9 and 907 games on 1×1, 1×2 and 1×3, identically with suicide
forbidden and with the paper's suicide-removal. Output is in
`data/count_games_scratch.txt`. This is scratch tooling in the language the
project does not use, run once, and it is not evidence for any ledger row:
the reproduction that counts is the Rust mirror's, under experiment 004.
Within a four-minute budget the script reached neither 1×4 nor 2×2.

One number from it matters for design. On 1×3 the memo held 1646 states for
907 games: keyed by the archive, the state space is about the size of the
game tree, because a simple path's archive nearly identifies the path.
Symmetry buys a constant. Nothing here compresses.

## What "formalizable" can mean for C-9

The ledger marks C-9 `formalizable`, and the statement is: a fuel-indexed
count over the computable state of `Decide.lean`, proved equal to the
cardinality of the legal complete move lists under `Defs.lean`'s `PSK`
through the existing bridge lemmas. The number is another matter. Kernel
reduction reaches the 1×n boards through 1×3 and probably not 1×4; the 2×2
count is 3.9 × 10¹¹ leaves, Tromp's search was "a few trillion nodes" over a
week in C, and counting simple paths has no compact certificate — the memo
observation above is that fact at small scale. So no certificate route ends
in `proved` at 2×2 by any means this project accepts, and an isolated
`native_decide` claim at that size is not credible either.

The honest shape is therefore: the definition and its correctness lemma in
Lean; the 1×1 through 1×3 entries of C-23 `proved` by `decide`, the first
kernel-checked agreement with numbers other people computed; 1×4 and 2×2
`computed` by Rust with witness files, which is the definitional validation
the trusted base actually asks for. The ledger row for C-9 should say so
when experiment 004 closes, so that `formalizable` is not read as a promise
about the number.
