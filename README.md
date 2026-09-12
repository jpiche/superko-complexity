# Superko Complexity

## Abstract

Generalized Go under the superko rule has an open complexity classification.
The bounds have not moved since 1980: PSPACE-hard by Lichtenstein and Sipser,
and in EXPSPACE by an archive argument that the literature records as
folklore. Robson settled the Japanese-rules variant as EXPTIME-complete in
1983, but both halves of that proof depend on properties the superko rule
removes. The three live answers are PSPACE-complete, EXPTIME-complete and
EXPSPACE-complete, and no consensus exists among the people who have looked.

This project attempts the classification. Its output is a set of **claims**,
each carrying an explicit status and an explicit witness, and — for as much of
the novel content as the tooling permits — a machine-checked proof in Lean 4.

[`docs/state-of-the-art.md`](docs/state-of-the-art.md) surveys what is known
and by whom. [`docs/formal-model.md`](docs/formal-model.md) fixes the exact
decision problem this project is about. [`docs/claim-ledger.md`](docs/claim-ledger.md)
is the ledger of every claim and its evidence.

## Why

Curiosity. Go provides an interesting problem space to explore unanswered
questions in computational complexity theory.

The author is unaffiliated and does not hold any relevant academic
credentials; this project is being pursued for fun.

## AI Usage

Joseph J. Piché is the author and maintainer, with AI models assisting.
Claude Code is the primary harness used. The model used per task is recorded
in the relevant notebook or experiment file.

## Status

Early. Six claims proved; the scaffolding, the definitions and the gates are
in place; the grounding for complexity-theoretic claims is decided
([`docs/plans/complexity-grounding.md`](docs/plans/complexity-grounding.md)).

The Lean development **builds** against Lean 4.34.0-rc2 and Mathlib, and the
definitions are exercised by kernel-checked sanity theorems in
[`lean/SuperkoComplexity/Sanity.lean`](lean/SuperkoComplexity/Sanity.lean)
covering board geometry, chains, liberties, capture, suicide and area scoring.
No `sorry`; no `native_decide`; the axiom record is
[`results/axioms.txt`](results/axioms.txt).

**What is proved.** Play terminates (C-13): under either superko rule the play
relation is well-founded, so no infinite sequence of legal moves exists. A game
begun from a position as the root of play makes at most 4·3^(m·n) moves (C-26).
From every state exactly one color has a winning strategy (C-28). A
fuel-indexed archive decider, run at the floor of the komi with that
game-length bound as its fuel, decides `BlackWins` (C-29); every komi is
equivalent to a half-integer one (C-27); and the string encoding of an
instance is injective, with the board linear in its length (C-31). All are
machine-checked on the three standard axioms, under
[`lean/SuperkoComplexity/Results/`](lean/SuperkoComplexity/Results/) and
[`lean/SuperkoComplexity/Encoding.lean`](lean/SuperkoComplexity/Encoding.lean).

**What is computed.** A Rust mirror of `Defs.lean`, not trusted, reproduces
the one game count another person computed from an independent
formalization: 386,356,909,593 games on 2×2 under positional superko, under
this project's rules and under Tromp's (C-39, C-38), together with the
smaller published counts and the legal-position table (experiment 004). The
same run found the suicide convention observable from four points on a line
(C-36). Every such number is `computed`: an untrusted program agreeing with
another untrusted program, checked by no kernel. The mirror is held to the
Lean by a second reading of the definitions at every position of every
board with at most six points, evaluated by the Lean compiler and graded
`observed`.

**What that is not.** The sanity checks are hand-computed and so can only
catch errors already imagined. The reproduced count is the independent
agreement the trusted base asks for and no more: it says the definitions
agree with Tromp's on one quantity, on one board, and it sees neither komi
nor scoring. The decider cannot do this work — a winner is not a game count,
and kernel evaluation of the decider reaches only small boards. Nor is any of
it a complexity result: the classes are grounded in prose, cited to Hearn
2006, and the one sentence that turns the decider into an EXPSPACE membership
— that a Turing machine iterating its step uses space polynomial in the
configuration — is claim C-32, `folklore`, written out in
[`proofs/C-32.md`](proofs/C-32.md). Both bounds on SUPERKO-GO remain inherited
rather than held. See [`docs/trusted-base.md`](docs/trusted-base.md) for what
a reader must believe.

## What this project trusts

A result here is worth exactly as much as its weakest dependency, so the
dependencies are named rather than implied.

- **Lean's kernel**, and the three standard Mathlib axioms (`propext`,
  `Classical.choice`, `Quot.sound`). Any theorem whose `#print axioms` shows
  more than these is flagged as such in the ledger.
- **The definitions in [`lean/SuperkoComplexity/Defs.lean`](lean/SuperkoComplexity/Defs.lean)**,
  and nothing else. That file is the trusted definitional core: it holds every
  definition the main theorem statement mentions, and it is kept short so that
  auditing it is an afternoon's work rather than a project.
- **Cited results from the literature**, each named in the ledger with its
  source. Where a source asserts without proving, the claim is marked
  `folklore` and treated as unproved.

Nothing else. In particular the Rust code is **not** trusted: it searches, and
what it finds is checked in Lean or not believed. See "Search and check" below.

## Project Standards

- Two languages, two jobs. **Lean 4 + Mathlib** states definitions and
  certifies theorems. **Rust**, with standard Cargo tooling, searches: it
  enumerates, solves and explores. Rust output enters the record only as a
  certificate that Lean checks.
- Every claim carries a status and a witness. An assertion without one does
  not belong in `docs/`.
- A computation on finite instances is evidence, never a theorem. The ledger
  keeps `computed` and `proved` apart, and
  [`tools/check-ledger.sh`](tools/check-ledger.sh) refuses to let a `proved`
  claim rest on an unproved one.
- Every number appearing in prose has a witness command that regenerates it.
  Committed under `results/`, checked by
  [`tools/verify-results.sh`](tools/verify-results.sh).
- Functions and algorithms are deterministic. Deviations are documented.
- Generated data that is not committed lives under `data/`, which is
  gitignored. Agent working files, solver dumps and captured output all go
  there — not to `/tmp`, not to the repository root.

### Documentation and comments

Documentation describes the work **as it stands**: what is known, what is
believed, what is open, and on what evidence. Write for a reader whose only
other context is this file.

The rule is [`docs/style.md`](docs/style.md); the two clauses that matter most
here are its own:

- **Status is part of every statement.** "Superko Go is in EXPTIME" is not a
  sentence this project may write. "Robson is reported to believe superko Go
  is in EXPTIME; no proof is known" is.
- **Keep the caveats, and keep the dead ends.** A refuted approach recorded is
  worth more than a fresh one imagined. Core docs state present-tense limits;
  `notebook/` keeps the history of how they were found.

## Project Structure

```
superko-complexity/
├── lean/                   Lean 4 development — definitions and certified proofs
│   └── SuperkoComplexity/
│       ├── Defs.lean       THE TRUSTED CORE. Audit target. Keep it short.
│       ├── Basic.lean      derived lemmas about the core
│       ├── Certificates/   checkers for witnesses the Rust side emits
│       └── Results/        one file per proved claim, named for its claim id
├── crates/                 Rust workspace — search and enumeration. Not trusted.
├── docs/                   reference, plans, archive — see docs/README.md
├── proofs/                 human-readable exposition, one file per claim
├── experiments/            one directory per experiment: spec, command, verdict
├── certificates/           witnesses the Rust side emits and Lean checks
├── results/                small committed outputs that claims cite
├── references/             bibliography and primary-source acquisition status
├── notebook/               dated lab notebook — hunches, dead ends, session logs
├── test_data/             fixture positions with expected values
├── tools/                  the gates: check-ledger, check-lean, verify-results
├── paper/                  the output
└── data/                   gitignored scratch
```

### Crates

The Rust side is search tooling. It is fast and disposable; correctness lives
in Lean.

| Crate | Responsibility |
|---|---|
| `superko-rules` | board, moves, PSK/SSK legality, history-carrying state — the executable mirror of `Defs.lean` |
| `superko-graph` | situation-graph construction, canonicalization, exhaustive enumeration |
| `superko-solve` | exact solvers over history states; the history-congruence quotient |
| `superko-geography` | Geography variants (UVG/DVG/UEG) — the comparator for the folklore argument |
| `superko-reduce` | reduction gadgets and their verification |
| `superko-cli` | the `superko` binary |

`superko-rules` carries an obligation the others do not: it must agree with
`Defs.lean`. The agreement is not proved, it is tested — see "Definitional
validation".

## Search and check

The division of labor, stated once because everything depends on it:

**Rust searches. Lean checks. Neither the search nor the searcher is trusted.**

Rust finds a candidate — a minimal position separating positional from
situational superko, a winning strategy, a fooling set — and emits it to
`certificates/` as data. Lean checks the certificate. Checking a witness is
far easier to formalize than reproducing the search, so the search code never
enters the trusted base and never needs to be formalized.

A claim moves from `computed` to `proved` when its certificate checks. Not
before.

## Definitional validation

A machine-checked proof establishes the theorem you *stated*. If the
definitions in `Defs.lean` are subtly not Go, the checkmark is worthless. The
definitions are therefore validated against numbers computed independently by
other people:

| Quantity | Published value | Source |
|---|---|---|
| 2×2 games under positional superko | 386,356,909,593 | Tromp |
| Legal positions L(m,n) | table | Tromp–Farnebäck |
| 1×n minimax scores under PSK | table, one entry disputed | Weninger–Hayward |

Reproducing these is the acceptance suite. Disagreement means the definitions
are wrong, not the literature. The disputed 1×9 entry — read as 0 in one
source and 4 in another — is the first thing the kernel can settle.

## The gates

Beyond a clean build with no new warnings:

| Gate | Enforces |
|---|---|
| [`tools/check-ledger.sh`](tools/check-ledger.sh) | no `proved` claim depends on an unproved one; every `formalized` claim names a theorem that exists |
| [`tools/check-lean.sh`](tools/check-lean.sh) | `lake build` is clean; no `sorry`; the axiom dump matches `results/axioms.txt` |
| [`tools/verify-results.sh`](tools/verify-results.sh) | every file in `results/` regenerates from its own header command |
| [`tools/check-docs.sh`](tools/check-docs.sh) | the prose rules in `docs/style.md` |

## Building

Rust: standard Cargo tooling, no external native dependencies. The toolchain
pin lives in [`rust-toolchain.toml`](rust-toolchain.toml).

Lean: install elan, which reads the pin in
[`lean/lean-toolchain`](lean/lean-toolchain).

```bash
curl https://elan.lean-lang.org/elan-init.sh -sSf | sh
```

Then `cd lean && lake build`. The first build compiles Mathlib and takes a
while. `lake` must be on `PATH` for
[`tools/check-lean.sh`](tools/check-lean.sh) and
[`tools/verify-results.sh`](tools/verify-results.sh) to check anything.

## Contributing

**I am not accepting issues or pull requests yet.** This is a personal research
project, early enough that the trusted definitional core is still moving —
review effort spent on it now would mostly be wasted. That changes as the
project matures, and this section changes with it.

Two things are welcome in the meantime:

- **Corrections.** A definition that is not Go, a claim stated above its
  evidence, a source misattributed — those are worth knowing straight away.
  Contact details are in [`CITATION.cff`](CITATION.cff).
- **Forking.** The license exists so that nobody has to ask: clone it, run the
  build, test a definition you doubt, and disagree in public.

A correction to a claim's status, a demotion included, is worth more here than
a new result.

## Legal

Copyright 2026 Joseph J. Piché. Licensed under the Apache License, Version 2.0
— see [`LICENSE`](LICENSE).

Personal research. It is not the work of, and carries no claim on behalf of,
any company.

Apache-2.0 is chosen rather than defaulted to. This project's whole claim to
attention is that a reader can check it instead of trusting the author, and
checking means running the build, re-running the search, and forking the
development to test a definition they doubt. A license that forbade any of
that would remove the one property the work has going for it. Apache-2.0 also
matches Mathlib's, leaving the door open to proposing results upstream.
[`docs/plans/publication-plan.md`](docs/plans/publication-plan.md) has the
fuller reasoning.
