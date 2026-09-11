# Proofs

Human-readable exposition, one file per claim, named for its ledger id.

For a `proved` claim that is machine-checked, the file here is **not** the
proof — the proof is in [`../lean/SuperkoComplexity/Results/`](../lean/SuperkoComplexity/Results/)
and the kernel checked it. The file here explains it: the idea, why the
construction is what it is, where the difficulty was. A referee reads this; a
machine reads the Lean.

Both are needed and they serve different readers. Neither substitutes for the
other, and where they disagree the Lean is authoritative
([`../docs/trusted-base.md`](../docs/trusted-base.md)).

For a `proved` claim that is not machine-checked — anything marked `infra-gap`
— the file here *is* the proof, and carries that fact in its first line, in
those words. A `folklore` claim this project writes out in full lives here on
the same terms, with a first line saying that no source proves it either.

## Contents

| File | Claim | Kind |
|---|---|---|
| [`C-32.md`](C-32.md) | the machine step of the archive argument | **the proof itself** — `folklore`, not machine-checked, and the file's first line says so |

The machine-checked claims (C-13, C-26, C-27, C-28, C-29, C-31) have no
exposition here yet. Their proofs are in
[`../lean/SuperkoComplexity/Results/`](../lean/SuperkoComplexity/Results/) and
[`../lean/SuperkoComplexity/Encoding.lean`](../lean/SuperkoComplexity/Encoding.lean),
and the ideas are recorded in the notebook entries the ledger's detail
sections cite.
