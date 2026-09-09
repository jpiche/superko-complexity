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
those words.

Empty. No claim is proved.
