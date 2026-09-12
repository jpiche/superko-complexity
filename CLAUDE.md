# Working agreements

Instructions for an agent working in this repository. [`README.md`](README.md)
describes the project and is the place to start; this file says what an agent
is expected to *do* differently.

This is a research repository; the deliverable is a set of claims with
evidence. The failure mode that matters is not a bug — it
is a claim that is believed more strongly than its evidence supports.

## The one rule

**Never assert a mathematical claim without its status.**

Every statement about the mathematics belongs to one of the ledger statuses in
[`docs/claim-ledger.md`](docs/claim-ledger.md): `proved`, `cited`, `folklore`,
`computed`, `conjecture`, `open`, `refuted`. Prose that asserts without one is
a defect, in a commit message and in a session summary as much as in `docs/`.

Two specific traps, both live in this literature:

- **Folklore is not proof.** "Go under superko is in EXPSPACE" is asserted in
  the literature with a one-line argument. It is probably right. It is marked
  `folklore` until this project or a cited source proves it.
- **Computation is not proof.** An exhaustive check on 2×2 says nothing about
  the asymptotics. `computed` never becomes `proved` without a Lean-checked
  certificate.

If you find yourself writing "clearly", "obviously", or "it follows that" in a
document, you are probably about to skip a status.

## Where work happens

This repository is Apache-2.0 and intended to be public. Anyone may clone it
and point an agent at it, so this file is written for any such agent — not
only for the maintainer's.

Work happens on a branch, never on `main`.

The maintainer additionally uses a hierarchy of worktrees:

- main at `../superko-complexity`
- feature at `../superko-<thread>` — one per research thread, long-lived
  (`../superko-upper-bound`, `../superko-psk-ssk`)
- ephemeral at the harness default, for experiments

**Before starting any work, confirm the current working directory and git
status are as expected.**

## Git

- Work on a branch. Do not commit to `main` and do not merge into it.
- **Do not `git push`.**. This project maintains that humans gate both merging
  to main and submitting a PR.
- Do not rewrite a commit that has been pushed.

Commit subjects take the form `<scope>: <lowercase description>`, no trailing
period. `scope` is a single short word — a crate name, a Lean module, `docs`,
`ledger`, or a claim id (`c14`). No Conventional Commits: do not write `feat:`
or `fix:`.

Describe what the change establishes, not which plan it came from. A commit
that changes a claim's status says so in the subject.

## Lean

- **`Defs.lean` is the trusted core and changes are expensive.** Every
  addition to it enlarges what a reader must audit. Before adding a
  definition, ask whether it can live in `Basic.lean` as a derived notion
  instead. A change to `Defs.lean` invalidates prior auditing and must be
  called out explicitly when handing work back.
- **Never leave `sorry` in a committed file** outside a file whose name marks
  it as a draft. `tools/check-lean.sh` rejects it.
- **`native_decide` is not free.** It adds the Lean compiler to the trusted
  base and appears as `Lean.ofReduceBool` in `#print axioms`. Prefer `decide`.
  Where `native_decide` is genuinely needed, the result is isolated in its own
  claim, marked in the ledger, and kept out of the dependency chain of any
  headline theorem. Do not launder it by proving a `decide`-shaped corollary.
- **A theorem statement is only as good as its definitions.** When you prove
  something, re-read what you actually stated. Restating a theorem so that it
  becomes provable is the most common way a formalization becomes worthless,
  and it is easy to do without noticing.
- Run `#print axioms` on every new top-level theorem and record the output.

## Rust

Rust is search tooling and is not trusted. It may be fast, heuristic and ugly.

What it may **not** be is a source of claims. A Rust computation produces a
`computed` ledger entry and a certificate under `certificates/`; it produces a
`proved` entry only once Lean checks that certificate.

`superko-rules` must agree with `Defs.lean`. When you change one, check the
other. The agreement is tested through the acceptance suite, not proved.

## Attribution

**Every file in `notebook/` and every experiment `README.md` carries an
`**Author:**` line and a `**Models:**` line at the top**, naming the person
accountable and the specific model or models used, with exact ids:

```
**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)
```

`**Models:** none` when no model was involved. List every model that did
substantive work, not just the last one.

This is not a disclaimer. The README discloses that AI assists the project;
what they can act on is which model produced *which* artifact: model behavior
differs enough between versions that a result's provenance is part of
reproducing it, and a reader assessing machine-assisted work in a field where
such work is now common needs the specific claim, not the general one.

The same reasoning applies as to the rest of the project — the reader is being
asked to check rather than to trust, so tell them exactly what they are
checking. `tools/check-docs.sh` enforces the presence of both lines.

## Experiments

An experiment gets a numbered directory under `experiments/` and a `README.md`
filled in from [`experiments/TEMPLATE.md`](experiments/TEMPLATE.md) **before it
is run**. The falsification field is not optional: state what result would kill
the hypothesis while you still do not know the answer.

A null result is a result. Record it and leave it in place.

## Scratch files

Write generated and throwaway output under `./data`, which is gitignored — not
to `/tmp`, and not to the repository root. Promote a file to `results/` only
when a claim cites it, and give it a witness header.

## Before handing work back

- `cargo build --workspace` and `cargo test --workspace` are clean, no new
  warnings.
- `tools/check-lean.sh` passes, once there is a Lean toolchain.
- `tools/check-ledger.sh` passes.
- Any claim whose status changed is updated in the ledger, with its witness.
- Say plainly what is *not* established. A summary that reads as though more
  was proved than was proved is the defect this project can least afford.
