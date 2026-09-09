# Tools

The gates. Each is a shell script with no dependencies beyond a POSIX shell,
`awk` and the project's own toolchains. Run them before handing work back
([`../CLAUDE.md`](../CLAUDE.md)).

| Script | Enforces | Needs |
|---|---|---|
| [`check-ledger.sh`](check-ledger.sh) | no `proved` claim rests on an unproved one; every `formalized:` claim names a theorem that exists | nothing |
| [`check-docs.sh`](check-docs.sh) | the mechanical parts of [`../docs/style.md`](../docs/style.md) | nothing |
| [`check-lean.sh`](check-lean.sh) | clean `lake build`, no `sorry`, axiom dump unchanged | Lean, for all but the `sorry` check |
| [`verify-results.sh`](verify-results.sh) | every file in `../results/` regenerates from its own witness command | the CLI |

All four run today and pass. `check-lean.sh` builds the development, scans for
`sorry`, elaborates `lean/Axioms.lean` and diffs the axiom record against
`results/axioms.txt`, failing on any appearance of `Lean.ofReduceBool`.
`verify-results.sh` currently checks one file, the axiom record.

## CI

[`../.github/workflows/ci.yml`](../.github/workflows/ci.yml) runs all four on
every push and pull request. The Lean job is the one that matters publicly: a
green build is the cheapest evidence a reader has that the development
compiles, holds no `sorry`, and depends on no axiom beyond the three standard
ones.

## What these do not catch

`check-docs.sh` finds banned words and dangling claim references. It cannot
find an assertion made at greater strength than its ledger status allows,
which is the defect that matters most. A clean run means no *detectable*
violation.

`check-ledger.sh` checks the shape of the dependency graph, not whether a
claimed dependency is the real one. A proof that quietly relies on something it
does not list passes.

Both gates are worth having anyway: they make the *cheap* failures impossible,
which leaves review for the expensive ones.
