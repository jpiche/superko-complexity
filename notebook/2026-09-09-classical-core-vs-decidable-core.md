# 2026-09-09 — classical core vs decidable core

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)

First session with a working Lean toolchain. `Defs.lean` compiled after three
fixes, none interesting: imports must precede the module docstring, one Mathlib
module had been renamed away (`Mathlib.Data.Rat.Order`), and the classical
definitions needed `noncomputable section`.

The interesting thing was what happened next.

## What went wrong

`Defs.lean` opens `scoped Classical`, so every definition in it is
noncomputable and nothing evaluates. That was a deliberate choice — it keeps
the audit target short, and computation was supposed to be the Rust side's
job.

But it is worse than "noncomputable". Instance resolution cannot see into
`Adj`, which is decidable on its face, because a plain `def` is semireducible.
So `decide (Adj p q)` fails on the most elementary predicate in the
development. The core as written could not be tested *at all*, not even for
board geometry.

That matters more than it first looks. The project's whole validation strategy
is independent numerical agreement — reproduce 386,356,909,593 and the
definitions are probably right. If the number is produced by counterparts
rather than by the audited definitions, the validation only transfers across a
bridge, and the bridge is then load-bearing.

## The three options

**A — classical core, bridged.** Keep `Defs.lean` short and classical. Put
decidable counterparts in `Basic.lean` with lemmas tying them to the core. The
trusted base grows by the *statements* of those lemmas.

**B — decidable core.** Make `Defs.lean` compute. This requires deciding
reachability, which requires either a graph construction with its correctness
proof or a Finset-closure argument with a path-length bound. Either one lands
20–40 lines of real mathematics — not inert plumbing — in the file a reader
must audit.

**C — reformulate the core** so decidability is free: define chains by bounded
iteration rather than transitive closure. Cheapest to compute, worst to audit,
because "iterate adjacency `card` times" is not what a Go player means by a
chain and a reader has to reconstruct the argument that it coincides.

## Chose A

Two reasons.

The bridging lemmas turned out cheap. Seven statements, each an equation
between a definition and its twin, each two or three lines. A reader who has
already accepted `Defs.lean` can check them in a minute, and the proofs are
kernel-checked.

And B's cost is concentrated in exactly the wrong place. Adding a `SimpleGraph`
construction plus `mem_chain_iff_reachable` to the trusted core would trade
seven short equations for a graph, a symmetry proof, an irreflexivity proof and
a reachability equivalence — all of which a reader must then audit *as part of
deciding whether the file describes Go*, which they do not, because they are
about Lean rather than about Go.

`docs/trusted-base.md` now says so, and lists the bridge as item 4.

## Worth revisiting if

The bridge count grows. Seven is fine; twenty would mean the audit has
migrated into `Basic.lean` without anyone deciding that it should. Watch it.

## Also

`Reaches` needed its own treatment: its step relation is not symmetric, since a
step requires its *source* empty while the endpoint is a stone, so it is not a
graph adjacency. The fix is a small observation worth keeping — the
intermediate points of such a walk are all empty, being sources of later steps,
so the walk splits into a symmetric path through empty points followed by one
step onto a stone. That prefix is a graph, and the split is `reaches_iff`.

The degenerate case fell out for free: no walk leaves an occupied point, so
from a stone `Reaches` collapses to that stone's own color.
