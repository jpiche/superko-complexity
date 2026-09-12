# 004 — Do the definitions reproduce the published game counts?

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`)
**Status:** done
**Closed:** 2026-09-12
**Opened:** 2026-09-11
**Claims touched:** C-9, C-23, C-36, C-37, C-38, C-39, C-40; the trusted base's definitional validation

## Question

Do the rules in `lean/SuperkoComplexity/Defs.lean`, mirrored in Rust,
induce the game counts Tromp–Farnebäck report in Table 7 of *Combinatorics
of Go* (rev. 2016): 1, 9, 907 and 2 098 407 841 on 1×1 through 1×4, and
386 356 909 593 on 2×2, all under positional superko?

This is the definitional validation `docs/trusted-base.md` rests on. Every
theorem proved so far is a theorem about `Defs.lean`; this experiment is
what makes it a theorem about Go, or shows that it is not.

## Hypothesis

All five counts reproduce, and reproduce identically under the two suicide
conventions in play: `Defs.lean` forbids suicide, the paper's ruleset removes
the mover's libertyless strings instead. The reason to expect agreement is
the hand argument in `notebook/2026-09-11-c9-primary-sources.md`: no
multi-stone suicide that does not repeat a position exists on 2×2, and the
first such shape on a line needs six points. Tromp's own program rejects
suicide outright, so the 2×2 number is a no-suicide count on its author's
reading as well.

## Falsification

**Filled in before the experiment runs.**

| Finding | Consequence |
|---|---|
| Any of the five counts differs under the no-suicide rule | The definitions are wrong, the source is wrong, or the mirror is wrong. Stop. Diff the mirror against `Defs.lean` first, then the small-board position graph against the paper's Figures 1 and 2. Those figures are drawn under the paper's suicide-permitting rules with self-loops removed: 5 nodes and 12 edges on 1×2, 15 and 42 on 1×3. Under `Defs.lean`'s rules the same census is 5 and 8, and 15 and 36 (scratch enumeration, 2026-09-11, `computed`); both figures are checked, each under its own convention. Nothing else proceeds until the cause is named. |
| The counts agree under the no-suicide rule and differ under suicide-removal on some board in the table | The hand argument is wrong, and Table 7 is not a no-suicide table on that board. C-9 or C-23 acquires a stated ruleset caveat, and `formal-model.md` §3 records that the suicide choice is observable at that size. |
| All five agree under both conventions | Hypothesis confirmed. C-9 and C-23 move to `computed` with witness files under `results/`; the trusted base's validation table gets its first checked row. |
| 2×2 does not finish in the compute available | A null result on the load-bearing row. Record the node rate and the projected time; the 1×n rows still land. |

## Method

1. Build `superko-rules` as the executable mirror of `Defs.lean`, with the
   suicide convention as a parameter so both counts come from one code path.
2. A depth-first enumerator in `superko-graph` over history states: archive
   of positions, pass counter, player to move; count a game at the second
   consecutive pass. Symmetry reduction on 2×2 is permitted only if the
   unreduced 1×n counts also come from the same enumerator without it.
3. Run 1×1 through 1×4, then 2×2, under both conventions. Each count lands in
   `results/` with the command, the commit and the rule as its witness header.
4. Cross-check the small-board position graphs against Figures 1 and 2 of the
   paper, as a second, independent quantity, under the paper's convention
   (suicide removed, self-loops excluded) and separately under `Defs.lean`'s.
5. Cross-check the legal-position counts L(m, n) against Tromp–Farnebäck's
   table, which exercises chains and liberties and no history at all.

## Result

Runs of 2026-09-11 and 2026-09-12 at commit `dbeac39`. Every value is
`computed` by the Rust mirror
([`../../docs/plans/rules-mirror-plan.md`](../../docs/plans/rules-mirror-plan.md)),
from the empty board with Black to move, root archived, passes exempt, and
the game ended at the second consecutive pass. Bodies with their witness
commands are under `results/`.

| board | rule | suicide forbidden (`Defs.lean`) | suicide removed (the paper's) | published |
|---|---|---|---|---|
| 1×1 | PSK | 1 | 1 | 1 |
| 1×2 | PSK | 9 | 9 | 9 |
| 1×3 | PSK | 907 | 907 | 907 |
| 1×4 | PSK | **719 178 893** | 2 098 407 841 | 2 098 407 841 |
| 1×4 | SSK | 1 359 471 437 | not run | none |
| 2×2 | PSK | **386 356 909 593** | 386 356 909 593 | 386 356 909 593 |
| 2×2 | SSK | 1 391 718 029 753 | not run | none |

The legal-position counts `L(1,1..8)`, `L(2,2)`, `L(2,3)` and `L(3,3)`
reproduce the published table, and the position-graph census reproduces
Figures 1 and 2 under the paper's convention and gives 8 and 36 edges under
`Defs.lean`'s (`cargo test -p superko-graph --test census`).

The 1×4 runs took 36 s, 107 s and 65 s single-threaded at about
6 × 10⁷ nodes per second (`data/`, not committed; wall time is not part of a
witness body). The 1×4 run under the paper's convention explores
6.3 × 10⁹ nodes for 2.1 × 10⁹ games, a ratio of three, which projected
1.2 × 10¹² nodes for 2×2; the two 2×2 runs each explored 1 159 070 728 779
nodes, 5790 s and 5842 s on seven threads at 2.0 × 10⁸ nodes per second.
They agree on every field of the body. Under the no-suicide rule
153 930 578 384 plays are refused as suicide; under suicide removal none is,
and the repetition refusals rise by exactly that number, which is the
mechanism of C-38 seen in the census: every such play returns to the
archived empty board. The 2×2 run under situational superko, launched
afterwards, explored 4 175 154 089 259 nodes in 11 431 s on fourteen threads
and gives 1 391 718 029 753 games, 3.6 times the PSK figure (C-40); the
ratio on 1×4 was 1.9.

## Verdict

**Falsification row 2 fired on 1×4.** The counts agree under `Defs.lean`'s
rule on 1×1 through 1×3 and disagree on 1×4, where the published figure is
reproduced exactly under the paper's suicide-removal convention and not
under the no-suicide rule. The hand argument in the hypothesis was wrong: a
two-stone self-capture that changes the position fits on four points. From
`.OX.` Black plays the right end; the chain of two has the white stone on
one side and the edge on the other, the white stone keeps its liberty, and
under suicide removal the result is `.O..`, a position the game need not
have visited.

**Row 3 holds on 2×2.** Both conventions give Tromp's 386 356 909 593 from
the empty 2×2 board, so the definitions of `Defs.lean` reproduce the one
game count another person computed from an independent formalization. That
is the definitional validation of `docs/trusted-base.md`, and it lands at
`computed`: an untrusted mirror agreeing with an untrusted program, on one
board, about one quantity.

Consequences, applied: C-23 now states its ruleset; C-36 and C-37 record the
1×4 counts under `Defs.lean`'s rules as `computed`; `formal-model.md` §3
records that the suicide choice is observable at four points; C-38 records
why 2×2 agrees under both conventions, now `computed`; C-39 records the 2×2
count under `Defs.lean`'s rules; C-9 moves to `computed`.

What this does not establish: that the mirror agrees with `Defs.lean` on
1×4 or 2×2 beyond what its tests reach (the naive arbiter has matched the
fast enumerator to twelve plies on 1×4 and ten on 2×2, and the Lean oracle
covers no game tree past two plies on either); that 719 178 893 or
1 359 471 437 or 1 391 718 029 753 is right, none having an independent
source; that the
definitions describe Go in any respect the 2×2 count does not see, komi and
scoring among them; or anything `proved`, since no kernel has checked a
count.
