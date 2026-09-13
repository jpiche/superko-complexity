# 2026-09-13 — positions separating the superko rules on six points

**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`): the main session ran the sweeps
and the witness checks directly; Opus 5 subagents of a workflow wrote these
records and reviewed them.

**Status of everything here: `computed`.** Nothing below is proved, and the
Rust solver that produced it is not trusted. The ledger rows are C-56 and
C-57; C-17 stays `open`.

## What was run

Every run below used the release build of `superko-cli`, suicide forbidden
(the rule of `Defs.lean`), and ran on its own on the 14-core machine.

### The budget ladder

The main session ran four sweeps on 2026-09-13 at commit `855617f` with a
clean tree, sequentially, each with `--suicide forbid --threads 10 --symmetry
on`, standard output to `data/separate-<board>-on-<budget>.txt` and standard
error, with `/usr/bin/time -l`'s report, to the matching `.err`. The log
`data/runs-2026-09-13.log` records the head and the start and end times, not
the command lines; the flags above are the ones the bodies and the `.err`
files name. The command was of the form

```
superko separate --board <board> --suicide forbid --threads 10 --symmetry on --budget <N>
```

`--symmetry on` searches one root per orbit of the board's symmetries (two on
1×6, four on 2×3) and the exchange of colors, and skips a play at a state a board symmetry fixes when
the symmetry maps an earlier play onto it. Both are unlicensed divergences
(`board-symmetry`, `color-swap`), and each body names them.

| board | budget per search | times | resolved | unresolved | separating | resolved with ssk-only | searched / transported | empty PSK / SSK | nodes | mirrored skips | elapsed | max RSS (bytes) |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1×6 | 10⁷ | 11:47:47–11:48:03 | 522 | 936 | 0 | 338 | 378 / 1 080 | unresolved / unresolved | 3 924 176 675 | 74 | 15.908 s | 3 850 240 |
| 2×3 | 10⁷ | 11:48:03–11:48:13 | 788 | 670 | 0 | 180 | 216 / 1 242 | unresolved / unresolved | 1 978 344 864 | 253 | 9.757 s | 3 309 568 |
| 1×6 | 10⁸ | 11:48:36–11:49:53 | 1 218 | 240 | 4 | 1 034 | 378 / 1 080 | 1 / unresolved | 15 997 820 530 | 97 | 75.546 s | 3 833 856 |
| 2×3 | 10⁸ | 11:49:53–11:51:22 | 992 | 466 | 4 | 384 | 216 / 1 242 | 0 / unresolved | 15 787 640 549 | 341 | 78.218 s | 3 342 336 |

Each board has 1 458 roots. In all four runs the least unresolved root is the
empty board with Black to move (`......:black`, `.../...:black`), and
`separating-liberties` equals `separating`. The recorded plain 2×3 sweep at
10⁷ (`results/separate-2x3-forbid-budget-1e7.txt`) resolved 792; the symmetric
one resolves 788. That run had both canonical roots (a transported root
resolves exactly when its representative does) and mirrored moves (which
change which roots resolve under a budget) on, and no run separated the two
effects, so the difference is not attributed to either.

### The witness blocks

Both 10⁸ bodies carry a witness block, and the `minimal-liberties-` block
names the same root with the same lines.

| board | least separating root found | to move | stones | libertyless chain | PSK value | SSK value | transported | `komi-floors` (read from the values, advisory) | witness floor | PSK Black / White wins | SSK Black / White wins | separates | determined |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 1×6 | `X.X.X.` | Black | 3 | none | 1 | 6 | no | 1, 2, 3, 4, 5 | 1 | false / true | true / false | true | true |
| 2×3 | `OOO/.X.` | White | 4 | none | 0 | −1 | no | −1 | −1 | true / false | false / true | true | true |

The two roots were searched themselves, not transported. Their witness
verdicts were searched with mirrored moves on, so under the `board-symmetry`
divergence. Neither minimum is unconditional (`minimum-unconditional=false`),
because the empty board ranks below both and is unresolved.

**Sanity check.** The 1×6 sweep at 10⁸ gives the empty 1×6 board the value 1
under PSK, the sixth entry of the transcribed table of C-24, which
`crates/superko-solve/tests/published.rs` reproduces with the plain solver.

### The plain confirmations

The main session then solved each witness root under each rule with no
`--symmetry`, from 11:52:25 to 11:52:55 with the release binary at `855617f`
(`data/witness-check-2026-09-13.txt`, standard output and standard error
interleaved):

```
superko solve --board <board> --rule <psk|ssk> --suicide forbid --root <root> --to-move <color> --komi-floor <k> --budget 100000000
```

| board | rule | root | to move | floor | value | nodes | max-depth | Black wins | White wins | elapsed |
|---|---|---|---|---|---|---|---|---|---|---|
| 1×6 | psk | `X.X.X.` | Black | 1 | 1 | 12 770 190 | 119 | false | true | 0.341 s |
| 6×1 | psk | `X/./X/./X/.` | Black | 1 | 1 | 12 770 190 | 119 | false | true | 0.338 s |
| 1×6 | ssk | `X.X.X.` | Black | 1 | 6 | 8 048 112 | 131 | true | false | 0.228 s |
| 6×1 | ssk | `X/./X/./X/.` | Black | 1 | 6 | 8 048 112 | 131 | true | false | 0.229 s |
| 2×3 | psk | `OOO/.X.` | White | −1 | 0 | 91 894 350 | 86 | true | false | 2.448 s |
| 3×2 | psk | `O./OX/O.` | White | −1 | unresolved | 100 000 001 | 91 | true | false | 2.653 s |
| 2×3 | ssk | `OOO/.X.` | White | −1 | unresolved | 100 000 001 | 96 | false | true | 2.735 s |
| 3×2 | ssk | `O./OX/O.` | White | −1 | unresolved | 100 000 001 | 97 | unresolved | unresolved | 2.815 s |

Each body's `divergences=` line names `dims-are-runtime` and
`rule-table-memo`, and under PSK `psk-archive-projection`, all unlicensed, and
`winner-via-floor-komi`, licensed by `Superko.winnerZ_eq_winner`. Neither
symmetry divergence appears. The four verdicts on each of 1×6 and 2×3 resolve
and agree with the sweep's witness block. So does every value that resolves.
The 2×3 SSK value does not resolve at 10⁸ in the plain search. Its verdicts
do.

The transposed 6×1 root gives identical values, verdicts, node counts and
depths. On a line the transposition does not give the search a different move
order, which is what identical node counts suggest, so it is not an
independent check of the search. On 3×2 the transposed 2×3 root's PSK
verdicts agree while its PSK value is unresolved at 10⁸, and its SSK searches
are unresolved at 10⁸. Neither transposed run is promoted to `results/`.

**Regenerated for `results/`.** The records step then ran the four 1×6 and
2×3 witness commands with the release binary at `389b984`, a clean tree, one
at a time
(`cargo run --release -q -p superko-cli -- solve ... --budget 100000000
2>/dev/null`). Each body's value, nodes, max-depth and verdicts equal the
lines above. The wall times under `time -p`, `cargo run` included, were
0.81 s, 0.41 s, 5.58 s and 8.16 s for 1×6 PSK, 1×6 SSK, 2×3 PSK and 2×3 SSK.
`tools/verify-results.sh` on the four files passed, in 15.20 s wall.

## What was not run, and why

- **The naive engine.** The four naive runs of the witness roots did not
  start: the command used `timeout`, which the macOS shell lacks (exit 127).
  They were not retried. At 8 to 92 million nodes for the pruned search, an
  unpruned search of these roots would not finish. That is a reading of the
  node counts, not a measurement.
- **3×2 and 6×1 sweeps.** Not run. The transposed witness roots above are the
  only runs on those boards.
- **The 10⁸ sweeps were not re-run by the records step.** 10⁸ nodes per
  search is over the cap an agent may run a sweep at; the main session
  verifies the two sweep files.
- **No hand analysis.** Why these two positions separate is not analyzed
  here beyond their description. No game tree was searched by hand or by
  script.

## Decisions the maintainer made

- **C-17 stays `open`.** The claim names a *minimal* position, and
  minimality in experiment 005's order is its substance. The existence of
  separating positions is recorded in a new `computed` row, C-56. The least
  board area carrying one, six, is C-57, which rests on C-53's comparison of
  winners on every board of at most five points.
- **Promoted to `results/`:** the two 10⁸ sweeps, as
  `separate-1x6-forbid-symmetry-on-budget-1e8.txt` and
  `separate-2x3-forbid-symmetry-on-budget-1e8.txt` with `# produced-with:
  threads=10`, and the four plain witness solves, as
  `witness-<board>-<rule>-forbid.txt`. Nothing else: not the 10⁷ sweeps, not
  the transposed solves.

Before those decisions the maintainer confirmed the reading: separating
positions exist on six-point boards, and none exists below six points under
`Defs.lean`'s rules, with the 1×5 winner gap closed by `389b984`
(`notebook/2026-09-13-closing-the-1x5-gap.md`).

## What is not established

- **The least separating position.** On 1×6 and on 2×3 the empty board with
  Black to move ranks below both witnesses and is unresolved at 10⁸ nodes per
  search under symmetry. 3×2 and 6×1 are not swept. C-17 stays `open`.
- **C-57 outside the floors tested.** Below six points the winners are
  compared at komi floors −(m·n)−1 to m·n+1 only.
- **The 2×3 SSK value.** The plain search does not resolve it at 10⁸; the
  value −1 comes from the symmetric sweep only, under two unlicensed
  divergences. The witness verdicts do not need it.
- **Anything proved.** No verdict here is checked by a kernel, and no Lean
  certificate was attempted. The Rust transition table and solver are
  untrusted; their agreement with `Defs.lean` is tested, not proved.
- **The other separating roots.** Each body counts four separating roots and
  names only the least it found. The other three on each board are not listed,
  and were not examined.
- **The separating roots among the unresolved ones.** A root the 10⁸ sweeps
  left unresolved, 240 on 1×6 and 466 on 2×3, may separate too.

## Next

- **The Lean angle**, experiment 005 row 3: kernel evaluation of
  `Superko.decideWins` at `X.X.X.` on 1×6 at komi floor 1 under both rules,
  and at `OOO/.X.` on 2×3 at floor −1, by `decide` and not `native_decide`.
  Whether the kernel reaches a six-point board is the question.
- **The least position.** The empty 1×6 and 2×3 boards under SSK are the
  obstacle; until they resolve, no minimum on a six-point board is
  unconditional. 3×2 and 6×1 are not swept, and experiment 005's order is
  per board.
- **The two 10⁸ sweep files** await the main session's verification. Their
  witness commands run on one thread and carry no `# slow:` line; whether they
  should is the maintainer's decision.
