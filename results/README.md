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

## Expected contents

| File | Holds |
|---|---|
| `axioms.txt` | the `#print axioms` dump for every headline theorem, checked by `check-lean.sh` |
| `acceptance.txt` | the definitional-validation suite: the 2×2 PSK game count, legal-position counts, the 1×n score table |

Empty. Nothing has been computed.
