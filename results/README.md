# Results

Small, committed outputs that a claim cites. Distinguished from `../data/`,
which is gitignored scratch, by three properties: a claim cites it, it is
small, and a command regenerates it.

## Format

Every file carries a witness header:

```
# witness: cargo run -p superko-cli -- count-games --board 2x2 --rule psk
# commit: <sha at which the value was recorded>
# date: <ISO date>
#
<body: the command's output>
```

[`../tools/verify-results.sh`](../tools/verify-results.sh) re-runs each witness
command and diffs. A measurement nobody can reproduce is an opinion
([`../docs/style.md`](../docs/style.md)).

Two more header lines are in use. `# defs-blob:` is `git hash-object` of
`lean/SuperkoComplexity/Defs.lean` at the time of the run, so a change to the
audit target invalidates every witness that claimed to mirror it. `# slow:`
marks a witness that takes hours; `verify-results.sh` skips such a file,
visibly, unless `SUPERKO_VERIFY_SLOW=1` is set.

## Contents

| File | Holds |
|---|---|
| `axioms.txt` | the `#print axioms` dump for every headline theorem, checked by `check-lean.sh` |
| `count-games-<board>-<rule>-<suicide>.txt` | one game count from the empty board by the Rust mirror (`computed`): the body `superko count-games` prints, with the resolved rules, the divergence list, the count, the node count and the refusal census. Cited by C-23, C-36, C-37 and, once the 2×2 runs land, C-9 and C-38 |

A `count-games` body is thread-independent; the witness command runs
single-threaded and a `# produced-with:` line says how many threads the
recorded run used. The 2×2 bodies are the ones marked `# slow:`.
