# 005 — Is there a minimal position separating positional from situational superko?

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)
**Status:** running
**Opened:** 2026-09-12
**Claims touched:** C-17, C-8, C-12, C-24, C-11; opened C-50 to C-55

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

Runs of 2026-09-12. The bodies under `results/` were produced at commit
`b93f20e`, except the 2×3 sweep, produced at `475f550`. Every value is `computed` by `superko-solve`, from `Defs.lean`'s
rules unless a table says otherwise.

**The gate of row 1.** `crates/superko-solve/tests/published.rs` reproduces
the empty-board 1×n values under PSK for n ≤ 6 — 0, 0, 3, 4, 0, 1, the first
six entries of the transcribed table. 1×7 and 1×8 do not resolve within the
test's budget of 4 × 10⁷ nodes, and 1×9 is not attempted.

**The solver's own checks, row 2.** In `crates/superko-solve/tests/agreement.rs`:

- The naive and fast engines agree on every value and every verdict at every
  root of every board of at most three points, under both rules and both
  suicide conventions.
- On 1×1, 1×2, 1×3, 1×4 and 2×2, exactly one color wins at every komi floor
  from −(m·n)−1 to m·n+1, and Black wins exactly when the value exceeds it
  (C-55).
- The value is unchanged under a second, reversed move order at the 816 of
  the 960 searches through 1×4 that the reversed order finishes within
  2 × 10⁶ nodes; the test pins that count.
- A line board and its transpose sweep identically, every field, with no root
  unresolved: through 1×5 with suicide forbidden and through 1×4 with it
  removed. 1×5 with suicide removed leaves roots unresolved at the test's
  budget and is not compared.

**The sweep, suicide forbidden** (`results/separate-<board>-forbid.txt`):

| board | roots | resolved | separating | resolved roots meeting an SSK-only play | empty board, PSK / SSK |
|---|---|---|---|---|---|
| 1×1 | 6 | 6 | 0 | 0 | 0 / 0 |
| 1×2 | 18 | 18 | 0 | 0 | 0 / 0 |
| 1×3 | 54 | 54 | 0 | 0 | 3 / 3 |
| 1×4 | 162 | 162 | 0 | 4 | 4 / 4 |
| 2×2 | 162 | 162 | 0 | 98 | 1 / 1 |
| 1×5 | 486 | 486 | 0 | 328 | 0 / 0 |
| 2×3, at 10⁷ nodes per search | 1458 | 792 | 0 | 184 | unresolved / unresolved |

"Meeting an SSK-only play" means the SSK search made a play PSK would refuse —
made, not merely generated at a node where a cutoff then pruned it. The count
depends on the order alpha-beta visits moves in and is a lower bound. On 1×3
the searches made none: every such play they generated was cut off. The 1×3
row therefore carries no evidence from the searches that the rules met there,
and the 1×3 legality gap is the one `crates/superko-graph/tests/containment.rs`
exhibits.

**2×3 is not exhaustive.** Its sweep ran under a budget of 10⁷ nodes per search
(`results/separate-2x3-forbid-budget-1e7.txt`: 1.35 × 10¹⁰ nodes in 363 s,
2 MB resident) and resolved 792 of the 1458 roots. The 666 it left unresolved
include the least root by rank, the empty board with Black to move. None of
the 792 separates, and 184 of them made a play PSK refuses. All 666 unresolved
roots made one as well, 850 roots in all: the budget stopped exactly the
searches in which the rules met. The empty root does not resolve within 10⁹
nodes under either rule (`superko solve --board 2x3 --rule psk --suicide
forbid --root .../... --budget 1000000000`, and the same with `--rule ssk`;
27 s each).

**The sweep, suicide removed** (`results/separate-<board>-remove-own.txt`),
which has no counterpart in `Defs.lean`:

| board | separating | least separating root |
|---|---|---|
| 1×1 | 0 | — |
| 1×2 | 4 | `X.`, Black to move: PSK −2, SSK 0 |
| 1×3 | 0 | — |
| 1×4 | 0 | — |
| 2×2 | 0 | — |

The four witness verdicts at komi floor −2, each a separate search: under PSK
White wins and Black does not; under SSK Black wins and White does not. Traced
by hand in `notebook/2026-09-12-c17-separation-search.md`.

Method step 4's verdicts came from the sweep body, which runs the four verdict
searches separately from the score searches, rather than from separate
`superko solve` runs.

**Not run.** The boards of six points, exhaustively: the empty 1×6 root
resolves under PSK in 27 925 122 nodes (`superko solve --board 1x6 --rule psk
--suicide forbid --root ......`), and a sweep of 1×6 was started and lost to a
machine crash before it finished; 2×3 was swept under a budget only, above, and
3×2 and 6×1 not at all. `remove-own` on 1×5. The Lean route of row 3,
which needs a witness under `Defs.lean`'s rules.

**The mechanism.** The analysis the sweep prompted became C-50, C-51 and C-52:
a separating play closes an odd walk of at least three plays with no pass at
the recurring board, and such plays occur in games with no pass. That
corrected the mechanism `docs/formal-model.md` §OPEN-1 stated.

## Verdict

**Row 1 passed, and row 2 did not fire**, so the sweep's numbers are worth
reading.

**Row 6 fired, on the boards swept.** Under `Defs.lean`'s rules no root
separates on any board of at most five points (C-53); under the
suicide-removing convention the least witness is `X.` on 1×2 (C-54). The
separation found is a fact about `superko-rules`, not about the audit target.
As the row prescribed: C-17 stays `open` for `Defs.lean`, C-54 records the
fact, and `docs/formal-model.md` §3 records that the suicide convention is
observable in the value.

**Row 4 held through five points, and six was reached only on 2×3, in part.** C-53 records the
exhausted region exactly. Under the suicide-removing convention separation is
not monotone in the board (C-54); under `Defs.lean`'s rules nothing is known
either way, so the null result bounds nothing above five points. Row 4's second
consequence — a row asking whether separation is impossible below some size —
is not opened: whether any position separates at all is C-17, and a proof of the
absence C-53 computes would move C-53 itself to `proved`, so the question
already has the rows it needs.

**Row 5 fired on 2×3.** At 10⁷ nodes per search the least unresolved root is
the empty board, so no minimum on 2×3 is unconditional, and the board holds a
null result: none of its 792 resolved roots separates, and nothing is known of
the 666 unresolved.

**The hypothesis is not refuted under `Defs.lean`'s rules, and is narrowed.**
It placed the smallest separating board at `m · n` between 4 and 6; 4 and 5
hold none, so it now requires six points, which are swept only in part. Under the
suicide-removing convention the least separating board has two points, outside
the guessed range.

**Row 3 was not reached**, so C-17 has no certificate.

Consequences applied: ledger rows C-50 to C-55; the witness columns of C-17
and C-24; `docs/formal-model.md` §OPEN-1 and §3; `docs/open-questions.md` §2
and §3; the validation table of `docs/trusted-base.md`.

What this does not establish: anything about boards of six points or more,
beyond the values at the 792 resolved roots of 2×3;
that the SSK-only plays counted are all such plays in the trees; that the two
rules give the same winner on 1×5 at every komi, which needs C-55 beyond the
boards it was checked on; and anything `proved` about C-17.

**Status stays `running`.** It resumes at the six-point boards, where the empty
2×3 root, unresolved at 10⁹ nodes, is the first obstacle.

**Tooling since these runs**
([`../../notebook/2026-09-12-solver-parallel-and-symmetry.md`](../../notebook/2026-09-12-solver-parallel-and-symmetry.md)).
`superko separate --threads N` spreads a sweep's roots over threads without
changing the body: the budgeted 2×3 body above came out byte-identical at
fourteen threads in 40 s wall, against 363 s for the recorded one-thread run.
`--symmetry on` searches one root per orbit of the board's symmetries and the
color exchange, and skips mirrored plays, under the unlicensed
`board-symmetry` and `color-swap` divergences; it changes the body and is off
by default. 2×3 has not been swept with either at a larger budget. A bench run
with mirrored moves resolved the empty 2×3 root under PSK to the value 0 in
93 137 907 nodes, where the plain search is unresolved at 10⁸ and 10⁹. That is
a measurement under an unlicensed divergence, not a result, and no claim
cites it.
