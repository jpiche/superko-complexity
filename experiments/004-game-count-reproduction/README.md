# 004 — Do the definitions reproduce the published game counts?

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`)
**Status:** planned
**Opened:** 2026-09-11
**Claims touched:** C-9, C-23; the trusted base's definitional validation

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

Not yet run.

## Verdict

Not yet run.
