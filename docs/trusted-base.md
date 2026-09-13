# The trusted base

What a skeptical reader must believe in order to believe a result from this
project, and — more usefully — what they need not.

The document exists because of an asymmetry. This project's results will be
offered without institutional credentials, into a literature where the prior on
"outsider resolves long-open problem" is justifiably near zero. A reader cannot
be asked to extend trust. They can be asked to check something small. This
document defines the small thing.

## What must be believed

1. **Lean's kernel is sound**, and the Lean toolchain pinned in
   [`../lean/lean-toolchain`](../lean/lean-toolchain) implements it.

2. **The three standard axioms** — `propext`, `Classical.choice`, `Quot.sound`
   — are consistent. Every headline theorem's `#print axioms` output is
   committed to `results/axioms.txt` and checked by
   [`../tools/check-lean.sh`](../tools/check-lean.sh). A theorem depending on
   anything further is not a headline theorem.

3. **The definitions in [`../lean/SuperkoComplexity/Defs.lean`](../lean/SuperkoComplexity/Defs.lean)
   describe Go under the AGA rules.** This is the whole audit, and it is the
   only part that cannot be discharged mechanically.

4. **The statements — not the proofs — of the bridging lemmas** in
   `Basic.lean`, `Decide.lean` and `Encoding.lean`, each marked `**Bridge.**`
   in its docstring, together with the one root statement they serve. See
   "The bridge" below for why these are unavoidable and what reading them
   costs.

5. **The results cited from the literature**, each named in
   [`claim-ledger.md`](claim-ledger.md) with its source, and each carrying the
   status the source's own argument supports — `cited` where the source proves
   it, `folklore` where the source asserts it.

That is the list.

## The bridge

`Defs.lean` is classical, so nothing in it evaluates: chains and reachability
are transitive closures, and a `def` is semireducible, so instance resolution
cannot even see that adjacency is decidable. Every computational check
therefore runs on a counterpart in `Basic.lean`, and a **bridging lemma** ties
the counterpart to the core definition.

The consequence is that the trusted base is `Defs.lean` *plus the statements of
those lemmas*. A checker that does not route through one proves nothing about
Go; a bridging lemma whose statement is subtly wrong makes every check that
uses it worthless, in exactly the way a wrong definition would.

The cost is small and bounded. Seven of the statements are two or three lines
each and read as an equation between a definition and its computable twin —
`area' b c = area b c`. Their proofs are kernel-checked and need no review.

Seven more of the same kind live in
[`../lean/SuperkoComplexity/Decide.lean`](../lean/SuperkoComplexity/Decide.lean),
where a computable game layer is tied to the core: `afterC_eq_after`,
`step'_toState`, `ended'_iff`, `winnerZ_eq_winner`, `start'_toState`,
`ssk'_faithful` and `psk'_faithful`, each an equation or an equivalence
between a twin and a notion of `Defs.lean`. They serve one statement of a
new kind, the root: `Superko.C29_decideWins_iff_blackWins` says that a
fuel-indexed archive decider, run at `⌊komi⌋` with fuel `4·3^(m·n) + 1`,
returns `true` exactly when `Superko.BlackWins m n b komi` holds. Its
right-hand side is the audited definition itself, so a wrong twin on the left
makes the theorem unprovable rather than misleading; what a reader audits in
the root is that it is not vacuous and quantifies over every position and
every komi.

Three more enter with the string encoding of
[`../lean/SuperkoComplexity/Encoding.lean`](../lean/SuperkoComplexity/Encoding.lean):
`cellFlat_idx`, which pins the row-major order; `boardOf_boardBits`, the
board's round trip through its bits; and `blackWinsFrom_black`, which says the
language's predicate at Black to move is `BlackWins` itself. The encoding's
eleven short definitions have referents a reader can check against — concrete
bit strings, kernel-checked on small boards — and the language `goLang` is
built over the audited `WinsFor`, so no Go semantics re-enters through them.

That is seventeen marked statements and one root statement, against the
threshold of twenty the notebook set on 2026-09-09. Any further growth is a
decision recorded in the notebook, not something absorbed.

This is an honest enlargement of item 3, not a footnote to it. The alternative
was a decidable core, which would have moved the reachability machinery — a
graph construction and its correctness proof — into the file a reader must
audit, trading seven short equations for something considerably worse to read.
The reasoning is recorded in [`../notebook/`](../notebook/).

## What need not be believed

- **The Rust code.** It searches; it decides nothing. Its output is a
  certificate that Lean checks, or it is `computed` and marked as evidence
  rather than proof. No claim's status depends on the search being correct,
  only on the checker being correct — and the checker is in Lean.
- **The prose.** [`formal-model.md`](formal-model.md), this file, and every
  proof exposition under `../proofs/` are explanations. If one contradicts the
  Lean, the Lean wins.
- **The author.** By construction.

## Item 3 is the whole problem

A machine-checked proof establishes *the theorem you stated*. A subtly wrong
definition yields a green checkmark on a true theorem about a game that is not
Go. Every serious objection to a formalized result attacks the definitions;
none attacks the proof, because there is no point.

Three defenses, in increasing order of strength:

**Brevity.** `Defs.lean` holds every definition the main theorem statement
mentions and nothing else. Derived notions live in `Basic.lean`, which needs no
audit because the kernel checks it. The core is kept short enough that reading
it is an afternoon, not a project. Every addition to it is a cost, and
[`../CLAUDE.md`](../CLAUDE.md) treats it as one.

**Explicit divergences.** [`formal-model.md`](formal-model.md) enumerates every
place where a different reading of the rules would give a different formal
object, and says which reading the core takes and why. A reader who disagrees
with a choice can find it rather than having to notice its absence. The choices
still under debate are marked **OPEN** there.

The Rust code's own departures from a literal reading of `Defs.lean` are a
closed list, `superko_rules::divergence::Divergence`, which
[`../tools/check-mirror.sh`](../tools/check-mirror.sh) holds equal to the
`DIVERGENCE:` markers in `crates/superko-rules/src`. There are seven, and a
results body names the ones its run was under in its `divergences=` line:
`dims-are-runtime`, `suicide-remove-own`, `psk-archive-projection`,
`rule-table-memo`, `winner-via-floor-komi` — the one a Lean theorem licenses,
`Superko.winnerZ_eq_winner` — and `board-symmetry` and `color-swap`. The last
two are not readings of a definition but properties of the rules that a
search assumes to skip work, and a run is under them only when it asks:
`board-symmetry` under any `--symmetry` value but `off`, `color-swap` under
`separate`'s `roots` and `on`. That the transition table commutes with every board
symmetry and with the color exchange is `computed` on boards of at most six
points and on 3×3 and 3×4. That the solver's values, and its verdicts at komi
floors `-(m·n) - 1` to `m·n + 1`, are the same with mirrored moves as without
is `computed` on boards of at most five points; on 1×5 and 5×1 with suicide
removing its own stones, only where both searches resolved within the budgets
of `superko-solve`'s `tests/mirrored.rs`. That values are invariant under the
board group and negate under the color exchange is `computed` on the same
boards, and so is the verdict transport, at the floors of that range whose
image stays in it; on 1×5 and 5×1 with suicide removing its own stones, only
at the roots resolved within the budgets of `tests/symmetry.rs`. On 2×3 and
3×2 under the no-suicide rule — the four-element group of a non-square
rectangle, which the 2×3 sweep runs under — the values with mirrored moves,
with canonical roots and with both are `computed` equal to the plain sweep's
only at the roots and rules both resolved within 10⁵ nodes a search, 1 504 of
2 916 on each
(`tests/symmetry.rs`); the other 1 412 are uncompared and no verdict is
compared there. Verdicts at
floors outside that range are not tested. `superko-rules`'s crate docs say a
witness header prints each divergence's consequence sentence, though no CLI code calls `consequence()` today.
Neither divergence is proved.

**Independent numerical agreement.** This is the strong one. The definitions
are made to reproduce quantities that other people computed, independently,
from their own formalizations:

| Quantity | Published value | Source | Status |
|---|---|---|---|
| 2×2 games under positional superko | 386,356,909,593 | Tromp | reproduced under this project's rules and under Tromp's (C-39, C-38, `computed`) |
| Legal positions `L(m,n)` | table | Tromp–Farnebäck | reproduced to 3×3 (`cargo test -p superko-graph --test census`, `computed`) |
| 1×n minimax scores under PSK | table | Weninger–Hayward | reproduced for n ≤ 6 under `Defs.lean`'s rule, whose suicide convention is the one the transcribed table names (C-24, `computed`); 1×7 and 1×8 unresolved within 4 × 10⁷ nodes, 1×9 not attempted. Reproduced for n ≤ 6 again with the solver's mirrored moves on, which rest on the unlicensed `board-symmetry` divergence (`crates/superko-solve/tests/published.rs`) |

A definition of Go that is subtly wrong will not produce 386,356,909,593. The
argument is not airtight — a definition could be wrong in a way these
quantities do not see, and this one sees neither komi nor scoring — but it
converts "trust my reading of the rules" into "check that my reading yields
the number three other researchers published". The smaller rows a reader can
verify in minutes; the 2×2 count is hours on one machine
([`../results/`](../results/) records each witness command). The mirror
that produces them is not trusted; it is held to `Defs.lean` by the Lean
oracle fixtures, which the Lean compiler evaluated and which are graded
`observed`, and by nothing the kernel has checked.

## The `native_decide` hazard

Lean's `decide` runs in the kernel. `native_decide` compiles to native code and
adds the Lean compiler — a large, unverified program — to the trusted base,
appearing as `Lean.ofReduceBool` in `#print axioms`.

Finite board computations make `native_decide` tempting, and a reader looking
for a reason to dismiss this work will find it immediately. The rule is in
[`../CLAUDE.md`](../CLAUDE.md): prefer `decide`; isolate any `native_decide`
result in its own claim; keep it out of the dependency chain of any headline
theorem; never launder it behind a `decide`-shaped corollary. Whatever survives
is disclosed here and in the ledger rather than buried.

## The boundary

Some of what this project needs it declines to formalize. Where the boundary
falls, and what grounds the claims on the far side of it, is a decision,
recorded with its evidence in
[`plans/complexity-grounding.md`](plans/complexity-grounding.md); this section
states the arrangement a reader must accept.

**The classes are the textbook's, by citation.** PSPACE, EXPTIME, EXPSPACE and
polynomial-time many-one reducibility mean what Hearn 2006, *Games, Puzzles,
and Computation*, Appendix A defines them to mean — Sipser's definitions,
transcribed by a coauthor of the survey that states this project's open
problem. Each definition is written out in this project's prose with the
citation attached. Sipser is added as a second, independent referent once a
named edition is held: the independent-agreement defense above, applied to
definitions. The referent is a book a reader can open, which is the one
property no library definition has.

**No complexity library is in `lean/`.** Mathlib has no resource-bounded
complexity theory — no class, no space measure, no resource-bounded reduction.
Several downstream Lean 4 libraries do, and each was examined at a pinned
revision (C-20): `SamuelSchlesinger/complexitylib` defines the classes over
multi-tape machines in the string model on this project's exact toolchain and
proves Savitch and Cook–Levin; `PierreSenellart/descriptive-complexity`
defines them as logics over finite structures and proves PSPACE-complete
problems; `leanprover/cslib` has a machine with a space measure and no class;
others are smaller. The project adopts none of them, for one reason that item
3 already states: adopting a library puts its definitions —
`Cfg.WithinDecisionSpace`, `DSPACE`, `SOPFPDefinable`, `ComplexityClass.ofMem`
— into the trusted base, and not one of them proves, or cites a proof, that
its class is the textbook's. Every library's bridge to the class the Go
literature means is asserted in a docstring or declined outright. That is a
poor trade for a project whose entire claim is that a reader can check it
instead of trusting the author, and it would be a poor trade even if the
libraries were mature and multiply reviewed, which most are not.

What the libraries do supply is narrower than the earlier version of this
section said. complexitylib's window calculus is generic in the space bound and
axiom-clean at this project's own Mathlib pin, so a Go decider *could* be
space-bounded there; what no library supplies is a PSPACE-complete source
problem under polynomial-time reductions over a string-encoded machine class.
complexitylib is therefore the subject of a pre-registered probe in a side
workspace, [`../experiments/003-complexitylib-window-probe/`](../experiments/003-complexitylib-window-probe/),
whose success would yield a second witness in a disclosed side base and never
a dependency of a headline theorem.

**The mathematics of the archive argument is on the formalized side.** That a
fuel-indexed archive decider decides `BlackWins` (C-29), that the input
encoding is injective with the board linear in its length (C-31), that komi
enters only through its floor (C-27), and that the game is determined (C-28)
are kernel-checked over `Defs.lean` and Mathlib alone. The run-level bound on
the decider's configuration (C-30) is stated and open.

**One sentence is prose, and it is named.** That a Turing machine iterating
the decider's step uses work space polynomial in the configuration size —
hence that SUPERKO-GO is in EXPSPACE once C-30 and C-31 are in hand — is
claim C-32, `folklore`, written out in [`../proofs/C-32.md`](../proofs/C-32.md).
It is the invariance thesis instantiated to this project's own functions, and
it is asserted by inspection, in those words: the algorithm-to-RAM half is
cited to Dershowitz and Falkovich-Derzhavetz 2015, and the RAM-space to
machine-space half is covered by no source this project has read. It is also
the entire content of the one-line published proof that the literature calls
folklore.

The arrangement the project aims at is unchanged:

> **Every novel claim lands on the formalized side. Every claim on the
> unformalized side is one this project cites rather than proves.**

C-32 is the named exception. It is novel only in that no source proves it for
a Lean function; it is the same sentence at which every published membership
proof for a game stops. Where the arrangement holds, the trusted base contains
no new mathematics that a machine has not checked. Where it does not hold —
C-32 — the ledger says so, this document says so, and the paper will say so,
in those words.
