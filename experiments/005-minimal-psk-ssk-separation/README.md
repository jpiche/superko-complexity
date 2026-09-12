# 005 — Is there a minimal position separating positional from situational superko?

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)
**Status:** planned
**Opened:** 2026-09-12
**Claims touched:** C-17, C-8, C-12, C-24, C-11

## Question

Is there a position whose game value differs under positional superko (PSK)
and situational superko (SSK), and what is the smallest such position?

Claim C-17 asserts that one exists and is `open`. No published source computes
a minimax value under SSK on any board, so no minimal separating case exists
in the literature (C-17's witness row records KGS anecdotes and nothing
computed).

### What "game value" means here, since nothing in the repository defines it

Two quantities, kept apart on purpose.

The **value** of a root is the minimax *area difference* — Black's area less
White's at the finished game, Black maximizing and White minimizing.
`Defs.lean` does not define it: `Superko.WinsFor` is a two-valued game at a
fixed komi. The value is a notion of `superko-solve`.

The **verdict** at a komi floor `k` is `Superko.WinsFor` itself, computed by
the recursion `Superko.decideWins` uses.

A **separating position** is a root whose value differs between the two rules.
The evidence a claim cites is never the value: it is a pair of verdicts at one
named komi floor, because that is the quantity `Defs.lean` defines and the
quantity a Lean kernel evaluation can check. Scores are how the search finds
candidates; verdicts are what the ledger records. Their agreement — Black wins
at komi floor `k` exactly when the value exceeds `k` — is a short induction
that is **not proved**, is `computed` by `crates/superko-solve/tests/agreement.rs`,
and is deliberately not load-bearing.

### What the sweep ranges over, and the order minimality is taken in

Every one of the `3^(m·n)` colorings `Superko.Position` admits, with each
color to move: `docs/formal-model.md` §5 takes a position as the root of play
(encoding (C)), and §7's language carries the color to move as input, so a
coloring no play can reach is an instance of the problem. Positions carrying a
libertyless chain are swept and reported separately, because OPEN-4 asks
whether restricting to reachable positions changes anything.

Minimality within a board: **stones ascending, then position code ascending,
then Black to move before White.** The stone count is the substantive part —
the empty board is the simplest root there is. The code order is an arbitrary
deterministic tiebreak and carries no meaning. Across boards: `m · n`
ascending. Fixed here, before the sweep runs, so that a witness cannot be
chosen to suit an order picked afterwards.

A minimum is reported as **unconditional** only when no root of lower rank
went unresolved at the node budget. A board with an unresolved root of lower
rank than the minimum found has not been shown to have that minimum.

## Hypothesis

A separating position exists, and the smallest board carrying one has
`m · n` between 4 and 6.

**This is a guess and is labeled one.** The reasons, such as they are:

*For a separation existing at all.* PSK is the strictly stronger restriction —
every play it permits, SSK permits — and the restriction bites early: the two
rules already differ in *legality* five moves into a game from the empty 1×3
board (`crates/superko-graph/tests/containment.rs`, `computed`). A rule change
that removes plays from both players generically changes some game's value.

*For the smallest board not being the smallest of all.* A legality divergence
is necessary and nowhere near sufficient. Both players always have the pass,
which is exempt from both rules (C-18, `cited`), so neither is ever forced to
play a losing move; the extra SSK plays buy a player something a pass does not
only when the board must *change* to escape a bad final position. On boards
with few points the score takes few values and there is little to escape to. A
scratch run of the sweep on 1×3 before this file was written found no
separating root among its 54, which is one board's worth of evidence for the
lower end of the guess and against nothing.

## Falsification

**Filled in before the sweep runs. Not optional.**

| Finding | Consequence |
|---|---|
| The empty-board 1×n PSK values disagree with the published table of `test_data/literature/linear-go-scores.toml` for any `n` the solver reaches | The solver is wrong, or `Defs.lean` is, or the transcription is. **Stop.** Nothing the sweep says is worth reading. This gate ran first and passed for `n ≤ 5` (0, 0, 3, 4, 0), which is why the sweep is being run at all. |
| The score search and the verdict search disagree at any komi floor, or both colors win from one state, or neither does | A bug in the solver. Determinacy is C-28, `proved`, so the solver is what is wrong. **Stop.** |
| A separating root is found, with the four witness verdicts confirming it at a named komi floor | C-17 moves to `computed` with that root as its witness. Then attempt the Lean route: kernel evaluation of `Superko.decideWins` at that root under both rules, `decideWins_sound`, and determinacy's exclusivity. If the kernel reaches it, C-17 moves to `proved` and this is the project's first novel result. If it does not, C-17 stays `computed` and the obstruction is recorded. |
| No separating root on any board the sweep resolves up to `m · n = 6` | C-17 stays `open`. A new `computed` row records the exhausted region exactly — which boards, which roots resolved, which did not. That is a real narrowing and it also raises a question worth its own row: whether a separation is *impossible* below some size, which would be a theorem rather than a search. |
| A board's roots are unresolved at the budget in numbers that make its minimum conditional | Null result for that board. Record the budget, the node counts and the least unresolved root by rank. Do not report a minimum as unconditional. |
| A separating root exists only under `Suicide::RemoveOwn` | The separation is about `superko-rules` and not about the audit target: `RemoveOwn` has no counterpart in `Defs.lean`. C-17 stays `open` for `Defs.lean`, a separate `computed` row records the fact, and `formal-model.md` §3 records that the suicide convention is observable in the *value* and not only in the game count (C-36). |
| The 1×n values reproduce but 2×2's PSK value has no published counterpart to check | Expected, and not a falsification: no source publishes a 2×2 minimax score. Recorded as unchecked. |

## Method

1. `cargo test --workspace`. The solver's validation is the gate of
   falsification row 1: `crates/superko-solve/tests/published.rs` against the
   published 1×n table, and `tests/agreement.rs` for the two engines, the two
   recursions, determinacy and the move-order independence of alpha-beta.

2. The sweep, board by board in `m · n` ascending order:

   ```
   superko separate --board MxN --suicide forbid [--budget N]
   ```

   `m · n = 1, 2, 3, 4, 5, 6`. Both orientations where cheap, as a check that
   the geometry is transposition-invariant; the headline is stated for
   `m ≤ n`.

3. The same under `--suicide remove-own`, which has no Lean counterpart, for
   contrast with the convention the published counts are under.

4. For the minimal separating root, the four verdicts written out
   independently of the sweep:

   ```
   superko solve --board MxN --rule psk|ssk --suicide forbid --root POS \
                 --to-move black|white --komi-floor K
   ```

5. If a root is found, a Lean file under `lean/SuperkoComplexity/Results/`
   proving the two verdicts by `decide` on `Superko.decideWins` — no
   `native_decide` — plus `#print axioms` recorded in `results/axioms.txt`.

Every body lands in `results/` with a witness header.

## Result

Not yet run.

## Verdict

Not yet reached.
