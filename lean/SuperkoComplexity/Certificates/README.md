# Certificate checkers

Rust searches; Lean checks. This directory holds the checkers.

A checker takes a witness from [`../../../certificates/`](../../../certificates/)
— a position, a strategy, a fooling set — and proves the property it claims.
Checking a witness is far easier to formalize than reproducing the search, so
the search code never enters the trusted base.

Two rules:

1. **Route through a bridging lemma.** Checkers run on the decidable
   counterparts in `Basic.lean`. Without the lemma tying those to `Defs.lean`,
   a green check says nothing about Go.
2. **Prefer `decide` to `native_decide`.** The latter adds the Lean compiler to
   the trusted base. Where it is unavoidable, the resulting claim is isolated
   and marked in the ledger.

A claim moves from `computed` to `proved` when its certificate checks here.
Not before.

Empty. No certificates exist.
