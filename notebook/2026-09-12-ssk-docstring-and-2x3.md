# 2026-09-12 — the SSK docstring, and 2×3 under a budget

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`), alone: it edited the docstring,
ran the regeneration, the probes and the sweep, and wrote these records.

A second session on C-17, after the first
([`2026-09-12-c17-separation-search.md`](2026-09-12-c17-separation-search.md))
was merged. It took two items from that notebook's list, in small steps: the
`Defs.lean` docstring, then the 2×3 sweep.

## The SSK docstring

`Defs.lean`'s `SSK` docstring called the pass exemption the parity resource
separating the two rules, which C-50 and C-52 contradict. It now names the
playable moves the rules disagree on: those whose position has stood with the
mover to move and never with the opponent to move. That much is the two
definitions read side by side. For such plays occurring in games with no pass
it cites C-52, `proved` by hand.

**This is a change to `Defs.lean`.** Only a docstring changed: no definition,
statement or signature. `git diff 9f75544 685d6dd`, the file's blobs before and
after, shows the one hunk. `lake build` rebuilt the dependents without a
warning, and `tools/check-lean.sh` found the axiom dump unchanged.

Every Lean oracle fixture records `Defs.lean`'s SHA-256, so all fourteen were
regenerated with `tools/gen-oracle.sh`. Each differs from its predecessor in
its `defs-sha256` line alone, and `crates/superko-rules/tests/lean_oracle.rs`
replays them green. The regeneration took 25 minutes of Lean interpreter time at
1.6 GB peak resident (`observed`). The six-point boards took nearly all of it:
284 s for 1×6, 295 s for 6×1, 356 s for 2×3 and 411 s for 3×2. The fixture
README records 279 s and 340 s for the first and third.

The `defs-blob` lines of existing `results/` headers are left as they were.
Each records the blob its run was made against, and rewriting one without
re-running would record a run that did not happen.

## Reach on 2×3

Measured before choosing a budget, with the release build, from the empty 2×3
board with Black to move:

| rule | budget | value | max depth | time |
|---|---|---|---|---|
| PSK | 10⁸ | unresolved | 99 | 2.7 s |
| SSK | 10⁸ | unresolved | 101 | 2.7 s |
| PSK | 10⁹ | unresolved | 99 | 26.9 s |
| SSK | 10⁹ | unresolved | 103 | 27.0 s |

`superko solve --board 2x3 --rule psk --suicide forbid --root .../... --budget
1000000000`, and likewise. That is about 3.7 × 10⁷ nodes a second at 2 MB
resident. The empty 1×6 root resolves under PSK in 2.8 × 10⁷ nodes; the empty
2×3 root takes more than 35 times that under either rule. So an exhaustive 2×3
sweep with this solver is out of reach whenever a noticeable share of the
board's roots cost what its empty root does. Whether they do was unmeasured.

## The sweep at 10⁷ nodes per search

`superko separate --board 2x3 --suicide forbid --budget 10000000`, recorded in
`results/separate-2x3-forbid-budget-1e7.txt`: 1.35 × 10¹⁰ nodes in 363 s,
2 MB resident. Every number below is `computed`.

| field | value |
|---|---|
| roots | 1458 |
| resolved by both searches | 792 |
| unresolved | 666 |
| least unresolved root | `.../...`, Black to move |
| separating among the resolved | 0 |
| resolved roots whose SSK search made a play PSK refuses | 184 |
| all roots whose SSK search made one | 850 |

Three things to read off it.

- **The null result is not vacuous.** At 184 of the resolved roots the SSK
  search made a play PSK refuses, so the trees differed and the values still
  agreed.
- **The budget stops exactly the searches where the rules meet.**
  850 − 184 = 666: every unresolved root made such a play. That repeats what the
  budgeted runs of the first session showed. The roots with repetition cycles
  are the expensive ones, so a budget selects against what the sweep looks for.
- **Nothing on 2×3 is a minimum.** The least unresolved root is the least root
  of all, so experiment 005's row 5 fires: a null result for the board, with
  its budget and least unresolved root recorded.

## What is not established

- Anything about the 666 unresolved roots of 2×3, the empty board among them.
  A separating position may be one of them.
- Anything about 1×6, 3×2 or 6×1.
- That the 792 agreeing values mean agreeing winners at every komi: that runs
  through C-55, which is computed on smaller boards only.

## Where the next session picks up

Each is a separate step, and the choice among them is the maintainer's.

1. **A stone floor.** `--min-stones K` at the same budget. The least `K` at
   which nothing is unresolved gives a statement about every root of at least
   `K` stones, one run per `K` tried, each cheaper than the full sweep.
2. **A larger budget.** At 10⁸ nodes per search the worst case adds about an
   hour, 666 × 2 × 10⁸ nodes at the measured rate, and nothing says the empty
   root falls under it.
3. **Reach.** A transposition table keyed on the whole state, item 3 of the
   first session's list. It is the step that could make 2×3 exhaustive.
4. **1×6, then 3×2 and 6×1.**
