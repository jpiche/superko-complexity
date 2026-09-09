# Certificates

Witnesses the Rust side emits and the Lean side checks.

**Rust searches. Lean checks. Neither the search nor the searcher is trusted.**

A certificate is data: a position, a winning strategy, a fooling set, a
correspondence between a reduction's two sides. The search that found it may be
fast, heuristic and unverified, because nothing rests on it. What rests on
something is the checker, and the checker is in
[`../lean/SuperkoComplexity/Certificates/`](../lean/SuperkoComplexity/Certificates/).

A claim moves from `computed` to `proved` when its certificate checks. Not
before.

## Format

Each certificate is a single file named for its claim — `C17-minimal-psk-ssk.json`
— and carries, in its own header or a sibling `.toml`:

- the claim it witnesses;
- the command that produced it, and the commit;
- the Lean checker that consumes it.

## Why this exists

The alternative is formalizing the search, which is enormously harder and buys
nothing. A minimax search is thousands of lines with a transposition table and
symmetry reduction, all of it subtle under superko because of the
graph-history-interaction problem. Checking that a *given* strategy wins is a
recursive walk over a finite tree.

The asymmetry is the whole design.

Empty. No certificates exist.
