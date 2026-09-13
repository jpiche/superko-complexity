# 2026-09-13 — closing the 1×5 gap in C-53

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`): a main session orchestrated a
workflow whose Opus 5 subagents implemented, reviewed and committed this step.

**Status of everything here: `computed`.** Nothing below is proved, and the
Rust solver that produced it is not trusted.

## The gap

C-53 records that under `Defs.lean`'s rules no position separates positional
from situational superko on any board of at most five points. The sweep behind
it compares **values**, the minimax area difference, because one value search
answers for every komi at once. The claim a reader cares about is about
**winners**, `Superko.WinsFor` at a komi, and values reach winners only through
the threshold agreement of C-55: Black wins at komi floor `k` exactly when the
value exceeds `k`.

C-55 was `computed` on 1×1, 1×2, 1×3, 1×4 and 2×2 and never on 1×5. So on
1×5 — and on the transposes of the lines — C-53 was a statement about values
only, and that the two rules give the same winner there was inferred, not
computed. Experiment 005's verdict listed it as not established.

It matters because experiment 005's narrowed hypothesis — a guess, labeled one
there — that six points is the smallest board carrying a separation under
`Defs.lean`'s rules leans on the five-point boards holding none, and a
difference in winners on 1×5 would not have shown in a comparison of values had
the threshold agreement failed there. The question
arose from the six-point sweeps of 2026-09-13.

## What the test checks

`the_winners_agree_under_both_rules_on_every_board_of_five_points` in
`crates/superko-solve/tests/agreement.rs`, ignored in debug. Under
`Suicide::Forbid`, the rule of `Defs.lean`, on 1×1, 1×2, 2×1, 1×3, 3×1, 1×4,
4×1, 2×2, 1×5 and 5×1, at every root (every position code, each color to
move), with the plain `Solver` — no symmetry, no mirrored moves, the default
order, no budget — it computes under each rule the value and, at every komi
floor from −(m·n)−1 to m·n+1, both colors' verdicts by the verdict recursion,
and asserts:

1. exactly one color wins;
2. Black wins exactly when that rule's value exceeds the floor;
3. the PSK value equals the SSK value;
4. at every floor, the PSK winner equals the SSK winner, compared for Black and
   for White.

The roots are spread over eight threads, each with its own two solvers; the
checks do not depend on the split.

## Counts and time

Pinned from arithmetic, not from a run: `2 · 3^(m·n)` roots a board and
`roots · (2·m·n + 3) · 2 · 2` verdict searches, asserted per board and in
total. 1×5 has 486 roots and 25 272 verdict searches; the ten boards have
1 608 roots and 76 944 verdict searches, with 3 216 value searches beside them.

It passed. `cargo test --release -p superko-solve --test agreement -- --ignored
the_winners_agree_under_both_rules_on_every_board_of_five_points` reported the
test finished in 0.03 s (0.31 s wall under `time -p`, the binary already
built), a single run on the 14-core machine. The same test in debug finished
in 0.29 s.

## What this establishes, and what it does not

`computed`: C-55 now holds, at the floors tested, on 2×1, 3×1, 4×1, 1×5 and
5×1 under the suicide-forbidding convention as well as on the boards it
covered; and C-53's absence of separation is a statement about winners, not
only values, on every board of at most five points in both orientations, at
every komi floor from −(m·n)−1 to m·n+1.

Not established:

- Floors outside that range are not tested. That the searches give there the
  verdict of the nearer end floor is not checked.
- Nothing is proved. The threshold agreement is a short induction over
  `WinsFor` that is not written, and no verdict here is checked by a kernel.
- The Rust transition table and solver are untrusted; their agreement with
  `Defs.lean` is tested, not proved.
- The suicide-removing convention on 1×5 and 5×1 is not covered, nor on the
  transposes 2×1, 3×1 and 4×1.
- Nothing about boards of six points or more.
