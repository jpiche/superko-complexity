# 2026-09-11 — designing the Rust mirror, with lessons from moyodojo

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`) for the reading, the
synthesis and this entry; Claude Opus 5 (`claude-opus-5`) for five
subsystem readers, three independent design drafts and two judges, run as
one workflow

The outcome is [`../docs/plans/rules-mirror-plan.md`](../docs/plans/rules-mirror-plan.md).
This entry keeps what that document is not allowed to hold: where the
choices came from, which sketch was overturned, and two arithmetic
corrections made along the way.

## Where the lessons came from

The maintainer's Go engine, moyodojo, is proprietary and was read for
lessons only; nothing was copied. It is a 19×19 engine built for
reinforcement-learning loops, so most of its machinery is the wrong shape
for boards of four points, and it says so itself: it cannot represent a
board smaller than 5×5 or a rectangle at all. What transfers is the defect
record, which is unusually honest and which names, in its own code, the
mistakes this crate must not repeat.

- **A hash is a lossy key.** moyodojo's repetition archive holds 64-bit
  Zobrist hashes and its search keeps a redundant cycle check "as a cheap
  backstop against reasoning errors and hash collisions". Fine at 2⁻⁶⁴ per
  comparison in a game engine; fatal for a count that must equal
  386 356 909 593, where a collision prunes a subtree and undercounts with no
  symptom. The mirror uses exact keys everywhere.
- **Sentinels collide.** An earlier `unwrap_or(0)` on the previous-board
  hash flagged legal moves as ko violations, because 0 is a legitimate hash
  of a real board. The mirror has no sentinel keys.
- **Three construction paths, three archive semantics.** A live game on an
  empty board left the archive empty, navigation seeded it, and a test-only
  constructor inserted under both colors. The mirror has one constructor,
  `start`, with private fields, and the plan removes the "root archived or
  not" toggle the first sketch had.
- **Recording pass situations is not optional.** moyodojo implemented
  "natural" situational superko, which does not archive a pass, and
  rejected it: pass, opponent takes the ko, immediate retake becomes legal.
  `Defs.lean`'s `step` inserts on every move, passes included, so the
  project was already on the right side; the shape belongs in the mirror's
  regression tests.
- **A per-path ban is not a move ban.** Superko forbids a resulting
  situation, not a move; a move banned two plies up may be legal deeper in
  the line. The enumerator skips an archived successor and never removes a
  move from a list.
- **Hash-container iteration order is per-process random**, and moyodojo
  names it as the cause of two identical runs diverging. `clippy.toml` now
  disallows `HashMap` and `HashSet` across the workspace.
- **The regime that is reported and the regime that is enforced can
  drift**, because nothing in the type system connects a `ko_rule()`
  accessor to the code that applies a seen-set. The mirror makes the rule a
  value the enumerator takes, as `WinsFor` takes `Repetition`.
- **No benchmark harness means no performance claims.** Every rate figure
  in moyodojo is an ad-hoc A/B or a hope. The 2×2 projection here may use
  only a measured 1×4 rate.
- **A settledness gate on the two-pass ending** exists there to stop a
  static scorer inflating results. Importing anything like it would change
  which leaves are terminal and break the published counts; mechanical
  scoring (C-16) is the reason the mirror has no such layer.

## What was overturned

The first sketch had two engines, a slow transliteration and a hand-written
bitboard layer, tested against each other forever. All three drafts kept the
transliteration; the winning one replaced the bitboard with a transition
table built by calling the transliteration, once per board. At these sizes
the table is tiny and the inner loop is two array reads, so the design that
is easiest to audit is also the fastest, and the unproved shortcut "only
chains adjacent to the played point can die", which every bitboard engine
takes silently, is never taken.

The sketch indexed the archive over legal positions. Rejected: `Defs.lean`
takes any of the `3^(m·n)` colorings as a root and archives it, and a root
with a libertyless chain has no slot in a legal-position index. The full
code space costs two words instead of one.

The sketch had a hash set as the fallback archive for larger boards.
Rejected for the reason above; the fallback is an exact key or a refusal.

## Two corrections

**The census.** Experiment 004's falsification table proposed the paper's
Figures 1 and 2, 5 nodes and 12 edges on 1×2 and 15 and 42 on 1×3, as a
cross-check on the no-suicide mirror. Both judges enumerated and found those
are remove-own figures with self-loops dropped. A scratch enumeration here
agrees: under `Defs.lean`'s rules the same census is 5 and 8, 15 and 36
(`computed`, not evidence). The row now checks each figure under its own
convention, and the plan's census tool counts self-loops separately.

**The root symmetry identity.** The sketch's formula for the empty 2×2
board, one plus eight times the count after a fixed black corner stone, was
criticized by all three drafts: the orbit of a corner has four elements, and
the pass branch recurses. Both points stand and the formula still holds
under PSK, for a reason the drafts missed. The orbit gives
`games = games(after pass) + 4·G`, and after a pass White faces a position
whose color swap is Black's, so under PSK, which reads boards alone,
`games(after pass) = 1 + 4·G`. Under SSK the swap fails, because the archive
after a pass holds the situation "empty board, White to move" that the
swapped tree lacks. The identity checks on 1×2, where it gives 9. None of it
is used: symmetry is off, unimplemented, and reserved for a cross-check.

## What the design does not settle

Whether the kernel can pin any oracle table by `decide`; every fixture is
`observed` until measured. Whether the `RemoveOwn` arm deserves a Lean twin
in `Basic.lean`; until it has one, its numbers are about the crate alone.
And whether 2×2 under SSK finishes at all: nobody has published that count,
it is bounded below by the PSK count, and the 1×n ratios are the only guide.
