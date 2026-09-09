# Results

One file per proved claim, named for its ledger id: `C13_Termination.lean`.

A file lands here when its claim is `proved` in
[`../../../docs/claim-ledger.md`](../../../docs/claim-ledger.md), and the
ledger's formalization column names the theorem it contains.

Every top-level theorem in a file here is listed in
[`../../Axioms.lean`](../../Axioms.lean), whose output is committed to
`results/axioms.txt` and diffed by `tools/check-lean.sh`. The `#print axioms`
lines live there rather than in the result file itself: a cached `lake build`
prints nothing, so an inline record would make the check vacuous.

| file | claims | theorem |
|---|---|---|
| [`C13_Termination.lean`](C13_Termination.lean) | C-13, C-26 | `C13_terminates`, `C13_length_bound_explicit` |
