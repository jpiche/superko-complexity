# The Rust mirror of `Defs.lean`

The plan for `superko-rules` and the enumerator in `superko-graph`: the
executable mirror of
[`../../lean/SuperkoComplexity/Defs.lean`](../../lean/SuperkoComplexity/Defs.lean)
and the tool that runs experiment 004. Written 2026-09-11 from a reading of
the maintainer's own Go engine (moyodojo, proprietary, read for lessons and
not copied) and from three independently drafted designs scored by two
judges; the notebook entry of the same date records what was learned there.
This document is the specification an implementer works from.

**Status of everything here: design.** Nothing below is a claim about Go.
The crate is not trusted, and no number it produces enters the ledger above
`computed`.

## The one idea

There is exactly **one implementation of the rules**: a module that
transliterates `Defs.lean` item for item, in Lean's order, written for a
reader holding the Lean open and not for speed. The fast path is not a second
engine. It is a **transition table built by calling the transliteration**,
once per board, over every position, color and point. On the boards this
project enumerates the table is small (2×2: 81 positions × 2 colors × 4
points), so the inner loop of the enumerator is two array reads and a bit
test and never runs a rule.

That removes the drift class a hand-written bitboard engine would carry
forever, removes the need for the unproved shortcut "only chains adjacent to
the played point can die", and is faster than the bitboard would have been at
these sizes. A bitboard layer may be added later for boards past the table's
reach; when it is, the table is its differential oracle.

## `superko-rules`

Rust 1.96, edition 2024, **no dependencies**. `clippy.toml` disallows
`std::collections::HashMap` and `HashSet` in the whole workspace: their
iteration order is per-process random, and determinism is a hard requirement.

### `reference.rs` — the transliteration

One item per `Defs.lean` item, in the file's order, each carrying a doc line
`**Mirrors** \`Superko.<Name>\`.` that `tools/check-mirror.sh` greps. Sets
are `BTreeSet`. Chains are the reflexive-transitive closure of `joined`,
computed by flood fill; at an empty point the chain is `{p}`, as in Lean.
`clear` reads the original board throughout and returns a fresh one; removing
in place while iterating is a different function, and that the two agree is a
theorem `Defs.lean` does not state (tested, never relied on). `resolve` is
total and sweeps the whole board, as `clear` does.

Items: `Color`, `Color::other`, `Dims` (the `{m n}` every Lean item takes
implicitly; rows and columns checked separately, never a flat index alone),
`Point`, `adj`, `Position` (a boxed slice of `Option<Color>`, equality on the
cells alone), `joined`, `chain`, `has_liberty`, `clear`, `resolve`,
`Situation`, `Move`, `situation_after`, `State` (private fields; `start` is
the only constructor; `passes` is an unbounded count, never saturated),
`step` (inserts on every move, passes included), `ended`, `playable_at`,
`ssk`, `psk`, `reaches`, `area`, `margin`, `winner`. `margin` and `winner`
are exact over a rational komi (`Rat { num: i64, den: i64 }`, `den > 0`);
`winner_z(b, komi_floor)` mirrors `Decide.lean`'s `winnerZ` and is registered
as a divergence licensed by `winnerZ_eq_winner`. `all_moves(dims)` is
`Decide.lean`'s `allMoves`: `Pass` first, then plays row-major. Every count
depends on that order, and a test pins it against a literal list.

Legality is a conjunction, so the crate also exposes a diagnostic
`legality(...) -> Result<(), Refusal>` with `Refusal { Occupied, Suicide,
Repetition }` in that fixed priority, and the contract
`legality(..).is_ok() == rule(..)` is property-tested. A count reproduced by
accident, with two conventions swapped and cancelling, is what the decomposed
verdict exists to catch.

### The two variant axes

`Suicide { Forbid, RemoveOwn }`, default `Forbid`, which is `Defs.lean`.
Under `RemoveOwn` exactly two things change, each behind a
`// DIVERGENCE: suicide-remove-own` marker: `resolve_remove_own` is
`clear (clear (update b p c) c.other) c`, and `playable_at` keeps the
emptiness conjunct alone. `RemoveOwn` has **no counterpart in `Defs.lean`**;
a number produced under it is a number about this crate, its divergence is
unlicensed, and every witness header says so. Closing that gap means adding a
`resolveTT` twin to `Basic.lean`, never to the audit target.

`Repetition { Ssk, Psk }`, default `Ssk`, which is the object; `Psk` is the
comparison and the rule the published counts are under. Both are values the
enumerator takes as a parameter, mirroring `Repetition` being a parameter of
`WinsFor`.

There is **no root-archiving variant**. `start` seeds the root
unconditionally, so does the mirror, and a second way to build a `State` is
the defect shape moyodojo actually shipped (three construction paths that
disagreed on the seed). The root seed is observable: on 1×1 under `RemoveOwn`
and PSK it is what keeps the count at the published 1.

### `code.rs`, `table.rs`, `archive.rs`

`PosCode(u32)`: base-3 index over **all** `3^(m·n)` positions, digit
`row·n + col` (0 empty, 1 black, 2 white); valid while `m·n ≤ 20`, asserted.
Internal only; nothing writes a code to a file. Positions cross a file
boundary as a row-major string over `.`, `X`, `O`.

`RuleTable::build(dims, suicide)` is the only constructor and its body calls
`reference::resolve` (or `resolve_remove_own`), `reference::playable_at` and
`reference::area`, and nothing else — `// DIVERGENCE: rule-table-memo`. It
holds, per (code, color, point): the successor code, and whether the play is
playable; per code: the two areas. Default cap `m·n ≤ 12` (51 MB); above it
the build is refused with a message rather than attempted.

The archive is a dense bitset over the full code space with **two bits per
position code**: bit `2·code` says the position was seen with Black to move,
bit `2·code + 1` with White. SSK reads one bit, PSK reads the OR of the two.
One structure serves both rules, so a difference between SSK and PSK counts
is attributable to the rule and not to bookkeeping, and the containment
"PSK-legal implies SSK-legal" is a free per-edge check. `insert` returns
whether the bit was new and is `#[must_use]`; `undo(key, was_new)` clears
only what `insert` set. The guard is mandatory, not an optimization: `now ∈
seen` is an invariant, so a pass always inserts a key whose position is
already archived, and a make/unmake that cleared unconditionally would forget
a genuine earlier occurrence and undercount. No key is ever a hash. At
2 × 10¹¹ archive operations a 64-bit digest is in birthday range, and a
collision prunes a subtree silently.

### `divergence.rs`

A closed enum. Every way the workspace departs from a literal reading of
`Defs.lean` is a variant with a slug, an optional Lean license, and a
one-sentence consequence printed in every witness header:
`dims-are-runtime`, `suicide-remove-own` (unlicensed),
`psk-archive-projection` (the PSK archive reads boards only; license: a
one-line lemma in `Basic.lean` that `PSK` reads `seen` through `.board`),
`rule-table-memo` (licensed by construction), `winner-via-floor-komi`
(`winnerZ_eq_winner`). `tools/check-mirror.sh` requires the set of
`// DIVERGENCE:` markers in the source to equal the set of slugs.

## `superko-graph`

`enumerate.rs`: depth-first search over history-carrying states, make and
unmake, **no memoization** — the 1×3 scratch run held 1646 memo states for
907 games, so an archive-keyed memo is about the size of the tree, and a memo
keyed on less than the archive is the graph-history-interaction error. Move
order is `all_moves`. A play whose key is archived is skipped, not removed
from a move list: superko forbids a resulting situation, not a move. Counts
are `u64` with `checked_add`. The report carries games, nodes, maximum
depth, maximum archive size and a refusal census (occupied, suicide,
repetition), all deterministic; wall time goes to stderr and never into a
results body, because `verify-results.sh` diffs the body.

Parallelism, when needed for 2×2: split at a fixed depth into the ordered
list of legal prefixes in `all_moves` order, count each independently on
`std::thread`, and sum in prefix order with exact integer addition. The
headline value is also run single-threaded and the two must agree.

`walk.rs`: the naive enumerator over `reference::State` with its `BTreeSet`
archive, cloning per node, sharing no control flow with `enumerate.rs`. It is
unusable past 1×3 and it is the arbiter: the fast count is quotable only
where the naive one has agreed with it, including a depth-capped 2×2 run
where the archive is non-trivial.

`census.rs`: legal-position counts `L(m, n)` by brute force, and the
position-graph census (legal positions as nodes, distinct successor
positions as edges) under both suicide conventions, with self-loops counted
separately so the paper's figures can be matched under its own convention.

Symmetry is **off and unimplemented** in this pass. When it comes, it is a
root-only decomposition and a cross-check, never a compression: two positions
in one orbit can carry archives in different orbits. The group is derived
from `(m, n)` — D4 only on a square, the Klein four-group on a rectangle,
order 2 on a line. On the empty 2×2 board under PSK the decomposition is
`games = games(after pass) + 4·G` with `G` the count after a fixed black
corner stone, and the color swap gives `games(after pass) = 1 + 4·G`; under
SSK the color swap does not apply, because the archive after a pass holds a
situation the swapped tree lacks.

## The Lean oracle

`lean/Oracle.lean`, outside the library like `Axioms.lean`, `#eval`s tables
from the computable twins whose bridge lemmas are proved (`resolve'`,
`PlayableAt'`, `area'`, `afterC`, `step'`, `start'`, `SSK'`, `PSK'`,
`winnerZ`), so a row is a statement about `Defs.lean` and not about a
Rust-convenient restatement. `tools/gen-oracle.sh` writes
`test_data/lean-oracle/<m>x<n>.txt`, line-oriented, with a header carrying
the toolchain, the SHA-256 of `Defs.lean`, `Basic.lean` and `Decide.lean`,
and `grade: observed`. Observed means the Lean compiler evaluated it, the
same trust surface as `native_decide`: a legitimate test oracle, evidence for
no ledger row. A `kernel-checked` grade, a `decide`-proved digest theorem per
table, is the intended upgrade where the kernel reaches, and its feasibility
is unmeasured.

Tables, in dependency order so a failure names the culprit: `resolve` at
every position, color and point, occupied points included, since `resolve`
is total; `playable` with its two conjuncts emitted separately; `area` per
position; `play` traces from `start'` to a fixed depth with the verdict of
every move in `allMoves` order under both `SSK'` and `PSK'`, which pins root
seeding, insertion on pass and the pass exemption; and `count` lines from a
fuel-indexed Lean counter for 1×1 through 1×3. Boards: every `(m, n)` with
`m·n ≤ 6`. A Rust test replays every table against `reference` and against
`RuleTable`. `tools/check-oracle.sh` refuses a fixture whose recorded hashes
do not match the working tree.

## Tests, by tier

Default `cargo test`, seconds: the `Sanity.lean` checks transliterated as
Rust tests, so each fact is asserted by two routes; the traps from moyodojo
(two neighbors of the played point in one dying chain; two chains dying at
once; `clear` reads the original board; single-stone suicide under
`RemoveOwn`, legal under SSK and a repetition under PSK, on 1×1; a pass is
legal though its situation is archived; `step` does not saturate `passes`);
`table_is_its_generator` for `m·n ≤ 6`; the naive and fast enumerators agree
on 1×1 through 1×3 under both rules and both conventions, and on 2×2 to a
depth cap; the counts 1, 9, 907; the census `L(1,1)..L(2,3)` and the
position-graph figures under both conventions; `all_moves` order pinned;
`legality` agrees with the rule predicate exhaustively; invariants over every
state a small enumeration visits (`now ∈ seen`; a legal play grows the
archive by one and a pass by zero or one; every leaf is `Ended`; the two
areas sum to at most `m·n`); two runs produce byte-identical reports.

`#[ignore]`, minutes: 1×4 under PSK, both conventions, and the fixed-workload
benchmark whose measured rate is the only number the 2×2 projection may use.
Hours: 2×2 under PSK, and 2×2 under SSK, which nobody has published and is
`computed` with no independent check.

## `superko-cli`

`superko count-games --board 2x2 --rule psk --suicide forbid [--naive]
[--threads N]`, `superko count-positions --board 3x3`,
`superko graph-census --board 1x3`, `superko check-oracle`. A command that
produces a results body prints the resolved configuration, the divergence
list and the counts; the witness header of
[`../../results/README.md`](../../results/README.md) gains a
`# defs-blob:` line with `git hash-object` of `Defs.lean`, so a change to the
audit target invalidates every witness that claimed to mirror it.

## What this plan does not do

No bitboard engine, no symmetry, no checkpointing, no solver, no
`kernel-checked` oracle grade, no `resolveTT` in Lean. Each is named here so
that its absence is a decision and not an oversight.
