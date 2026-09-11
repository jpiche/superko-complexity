# The Lean development

## Building

Pinned to `leanprover/lean4:v4.34.0-rc2`, matching the Mathlib revision in
`lake-manifest.json`.

```bash
lake build
```

If the dependencies are not present, `lake exe cache get` first — it fetches
Mathlib's prebuilt oleans, and skipping it means compiling Mathlib from source,
which takes hours.

`elan` installs to `~/.elan/bin` and puts itself on `PATH` through
`~/.zprofile`. A shell started before the install will not have it; export it
by hand rather than concluding the install failed.

## Layout

| | |
|---|---|
| `SuperkoComplexity/Defs.lean` | **the trusted core** — every definition the main theorem mentions, and nothing else |
| `SuperkoComplexity/Basic.lean` | derived notions and the decidable counterparts |
| `SuperkoComplexity/Sanity.lean` | kernel-checked checks on concrete positions |
| `SuperkoComplexity/Decide.lean` | the fuel-indexed archive decider, computable, with the lemmas tying it to the core definitions |
| `SuperkoComplexity/Encoding.lean` | the bit encoding of an instance, its decoder, and the length bounds the archive argument needs |
| `SuperkoComplexity/Certificates/` | checkers for witnesses the Rust side emits |
| `SuperkoComplexity/Results/` | one file per proved claim, named for its ledger id |
| `Axioms.lean` | the axiom record; outside the library, elaborated by `check-lean.sh` |

`Defs.lean` is the audit target and additions to it are expensive — see
[`../docs/trusted-base.md`](../docs/trusted-base.md). A derived notion goes in
`Basic.lean`, where the kernel checks it and no human need read it.

## The classical/decidable split

`Defs.lean` is classical, and therefore noncomputable. Chains and reachability
are transitive closures; making them decidable inline would fill the audit
target with instance plumbing.

`Basic.lean` carries decidable counterparts together with lemmas proving them
equal to the core definitions. Certificate checkers use the decidable versions,
and the bridging lemmas are what make a certificate check say something about
the definitions a reader actually audited. A checker that does not route
through a bridging lemma proves nothing about Go.

## Conventions

- Every top-level theorem gets `#print axioms`, and the output is committed to
  `results/axioms.txt`.
- `sorry` never lands in a committed file. `tools/check-lean.sh` rejects it.
- `native_decide` adds the Lean compiler to the trusted base and shows up as
  `Lean.ofReduceBool`. Prefer `decide`; isolate what needs it. See
  [`../CLAUDE.md`](../CLAUDE.md).
- A theorem proving a ledger claim is named for it: `C13_terminates`.
