# Tools

The gates. Each is a shell script with no dependencies beyond a POSIX shell,
`awk` and the project's own toolchains. Run them before handing work back
([`../CLAUDE.md`](../CLAUDE.md)).

| Script | Enforces | Needs |
|---|---|---|
| [`check-ledger.sh`](check-ledger.sh) | no `proved` claim rests on an unproved one; every `formalized:` claim names a theorem that exists | nothing |
| [`check-docs.sh`](check-docs.sh) | the mechanical parts of [`../docs/style.md`](../docs/style.md) | nothing |
| [`check-mirror.sh`](check-mirror.sh) | every `Defs.lean` item is mirrored in `superko-rules` or listed as held elsewhere; every `**Mirrors**` line names an item that exists; the `DIVERGENCE:` markers and the slugs in `divergence.rs` are the same set | nothing |
| [`check-lean.sh`](check-lean.sh) | clean `lake build`, no `sorry`, axiom dump unchanged | Lean, for all but the `sorry` check |
| [`check-oracle.sh`](check-oracle.sh) | every fixture under `../test_data/lean-oracle/` records the SHA-256 of the `Defs.lean`, `Basic.lean` and `Decide.lean` it was generated from, and they still match | nothing |
| [`verify-results.sh`](verify-results.sh) | every file in `../results/` regenerates from its own witness command | the CLI |

One script generates rather than checks:

| Script | Produces | Needs |
|---|---|---|
| [`gen-oracle.sh`](gen-oracle.sh) | `../test_data/lean-oracle/<m>x<n>.txt` for every board with `m * n ≤ 6`, by running `../lean/Oracle.lean` through the Lean interpreter | Lean; about half an hour for the whole set |

All six gates run today and pass. `check-lean.sh` builds the development,
scans for `sorry`, elaborates `lean/Axioms.lean` and diffs the axiom record
against `results/axioms.txt`, failing on any appearance of
`Lean.ofReduceBool`.
`verify-results.sh` currently checks one file, the axiom record.

## CI

[`../.github/workflows/ci.yml`](../.github/workflows/ci.yml) runs all six on
every push and pull request. `gen-oracle.sh` is not among them: regenerating
the fixtures needs a Lean toolchain and half an hour, and `check-oracle.sh` is
the cheap gate that says whether it needs doing.

The Lean job is the one that matters publicly: a green build is the cheapest
evidence a reader has that the development compiles, holds no `sorry`, and
depends on no axiom beyond the three standard ones.

`check-oracle.sh` compares hashes and header shape. Whether a fixture's rows
are *right* is `cargo test -p superko-rules --test lean_oracle`, which replays
every one of them against the mirror; and the rows themselves are graded
`observed` — produced by the Lean compiler, not its kernel — so they support
no ledger entry. See [`../test_data/lean-oracle/README.md`](../test_data/lean-oracle/README.md).

`check-mirror.sh` compares declaration names and comment markers. It cannot
tell whether a mirrored item mirrors its Lean counterpart *correctly*, which is
what the crate's tests and the acceptance suite are for.

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
