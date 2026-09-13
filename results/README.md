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
| `scc-census-<board>-<suicide>.txt` | the strongly connected components of one board's situation graph (`computed`): vertices, edges, components, the largest, the empty board's, the count outside the largest, and the condensation depth, under both readings of the vertex set — `legal-` over the positions play can reach, `all-` over every coloring `Defs.lean` admits. `outside-largest` is the number C-46 cites: it bounds what C-42's forward-cone prune can remove. Cited by C-45 and C-46 |
| `separate-<board>-<suicide>[-budget-<N>].txt` | the separation sweep of one board (`computed`), described below. Cited by C-53 and C-54 |

A `separate` file holds every root under both superko rules, the resolved and
unresolved counts, the empty board's value under each rule, the number of
separating roots, the plays positional superko would refuse that situational
superko's searches made and the roots — the resolved ones separately — whose
search made one, whether each of the two minima is unconditional, and, when a
root separates, the least in experiment 005's order with its four witness
verdicts at one komi floor. A sweep run under a node budget names the budget in
the file name as well as in the body, and what it shows holds at its resolved
roots only.

A `count-games` body is thread-independent; the witness command runs
single-threaded and a `# produced-with:` line says how many threads the
recorded run used. The 2×2 bodies are the ones marked `# slow:`.

A `separate` body is thread-independent too: `--threads N` spreads the sweep's
roots over N threads and adds nothing to the body
(`crates/superko-solve/tests/threads.rs` holds the sweep and its body equal at
one, two and fourteen threads on the boards it names). The witness command
carries no `--threads`, and a file whose recorded run used more than one
thread says so in a header line of the form

```
# produced-with: threads=14
```

A file without that line was produced on one thread. The two forms differ: the
`count-games` files already committed carry a free-form line that begins
`# produced-with: --threads N` (N is 4, 7 or 14 among them) and goes on in free
text, saying why the body does not depend on it, while `threads=N` is the form
for `separate`, which no committed `separate` file carries yet. Both are header
lines, outside the body `verify-results.sh` diffs.

A `separate` body produced with `--symmetry on` is a different body, and says
so. The sweep searches one root of each orbit under the board's symmetries and
the exchange of the two colors, and gives every other root its representative's
values — negated when the colors were exchanged — before adding the sweep up
(`crates/superko-solve/src/separate.rs`). The body then carries

- `board-symmetry:unlicensed` and `color-swap:unlicensed` at the end of the
  `divergences=` line;
- after `skipped=`, the lines `symmetry=on`, `symmetry-searched=N`, the roots
  whose two searches ran, and `symmetry-transported=M`, the roots whose values
  were transported, with `N + M + skipped = roots`;
- after those, the line `symmetry-mirrored-moves=on`: every search, the
  witness verdicts included, skips a play at a state a board symmetry fixes,
  archive included, when the symmetry maps an earlier play onto it
  (`crates/superko-solve/src/search.rs`). No test checks that the witness
  verdict searches skip; that they do is read from the code;
- in each witness block, after `-ssk=`, a line `minimal-transported=` (or
  `minimal-liberties-transported=`) saying whether that root's values were
  transported. The witness verdicts are searched on the root itself either way.

That values are unchanged by a board symmetry and negated by the color
exchange is not proved. It is `computed` at every root of every board of at
most five points, except on 1x5 and 5x1 with suicide removing its own stones,
where it is `computed` only at the roots resolved within the node budgets
`crates/superko-solve/tests/symmetry.rs` names, which leave most of those roots
uncompared (the two divergences' consequence sentences). Under a node budget a
transported root is resolved exactly when its representative is, so a budgeted
body with symmetry on can report different resolved, unresolved and ssk-only
counts from the same sweep without it; mirrored moves change the ssk-only
counts too, and under a budget which roots resolve. That values, and verdicts
at komi floors `-(m·n) - 1` to `m·n + 1`, are the same with mirrored plays
skipped as without is `computed` at every root of every board of at most five
points, except on 1x5 and 5x1 with suicide removing its own stones, where it is
`computed` only where both searches resolved within the budgets
`crates/superko-solve/tests/mirrored.rs` names (440 of 972 values and 15 976 of
25 272 verdicts, both rules, on each). Verdicts at floors outside that range are
not tested, and none of it is proved. Without `--symmetry on` none of these lines appears
and the body is unchanged.

A `solve` body produced with `--symmetry on` turns on mirrored moves alone,
in the value search and in any verdict search. It carries
`board-symmetry:unlicensed` at the end of the `divergences=` line, and after
`max-depth=` the lines `symmetry-mirrored-moves=on` and
`symmetry-mirrored-skips=N`, the plays the value search skipped. Its `nodes=`
and `max-depth=` are the mirrored search's. Without the flag the body is
unchanged.
