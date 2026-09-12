/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity

/-!
# The Lean oracle

Not part of the library, like `Axioms.lean`. `tools/gen-oracle.sh` runs this
file with `lake env lean --run lean/Oracle.lean <m> <n>` and writes what it
prints to `test_data/lean-oracle/<m>x<n>.txt`;
`crates/superko-rules/tests/lean_oracle.rs` replays every row against the Rust
mirror. Nothing in the library imports this file and the lakefile does not
build it.

## What a row says

Every value printed here is produced by the computable twins in `Basic.lean`
and `Decide.lean` — `resolve'`, `PlayableAt'`, `area'`, `afterC`, `step'`,
`start'`, `SSK'`, `PSK'` — and by nothing else. Each of those is tied to the
`Defs.lean` item it computes by a bridge lemma the kernel has checked:
`resolve'_eq_resolve`, `playableAt'_iff`, `area'_eq_area`, `afterC_eq_after`,
`step'_toState`, `start'_toState`, `ssk'_faithful`, `psk'_faithful`. A row is
therefore a statement about the audit target and not about a restatement
convenient to Rust.

## Status of every row: `observed`

The rows are evaluated by the Lean *compiler*, through the interpreter that
`lean --run` starts. That is the trust surface `native_decide` has and not the
kernel's: no row below is backed by a proof, and none of them may support a
ledger entry. They exist to catch a Rust mirror that has drifted from the Lean
it claims to mirror, which is a use for which `observed` is enough.

The intended upgrade is a `decide`-proved digest theorem per table, a
`kernel-checked` grade; its feasibility at these board sizes is unmeasured
(`docs/plans/rules-mirror-plan.md`, "The Lean oracle").

The plan speaks of `#eval`; this file exposes a `main` and is run with
`lean --run` instead, which prints to stdout rather than into the message log
and so needs no text stripped out of it. The two share an evaluator and a
trust surface, so the grade is unchanged.

## The tables

In dependency order, so that a disagreement names the culprit rather than the
last thing that read it.

* `resolve <pos> <color> <row,col> <result>` — `resolve'` at every position,
  color and point of the board, occupied points included, because `resolve` is
  total and is defined there too.
* `playable <pos> <color> <row,col> <empty> <liberty>` — the two conjuncts of
  `PlayableAt'` separately: the point is empty, and the played stone's chain
  has a liberty in `resolve' b c p`. Emitting them apart is what distinguishes
  a mirror that refuses the right moves from one that refuses them for the
  wrong reason.
* `area <pos> <black> <white>` — `area'` for both colors.
* `play <rule> <depth> <path> <pos> <tomove> <move> <verdict> <passes>
  <archive>` — one line per move of `allMoves` at every state reached from
  `start'` on the empty board with Black to move by a legal sequence of at
  most `playDepthFor (m * n)` moves, the figure printed in the file's
  `# play-depth:` header. `verdict` is `SSK'` or `PSK'` at that state and
  move, `1` for legal. The recursion does not continue past a state where
  `Ended'` holds, and such a state still contributes its own lines. `path` is
  the move sequence from the root, `;`-separated, `root` when empty. This is
  the table that pins the root seeding of `start'`, the insertion `step'`
  performs on a pass, and the pass exemption both rules carry.
* `count <rule> <games>` — games from the empty board under that rule,
  counted by `countLevel` below. Emitted only where the walk finishes here,
  which measurement puts at `m * n ≤ 2`; see `countLimit`. A depth-capped
  number is not a count of games and is not offered as one.

A position crosses this boundary as its rows over `.`, `X` (black) and `O`
(white), row-major, separated by `/` — never as a position code, which is
meaningless without the convention that produced it. Points are `row,col`,
zero-based. Colors are `b` and `w`.

Positions are enumerated in base-3 code order: the code of a position is
`∑ digit(row · n + col) · 3 ^ (row · n + col)` with digit values 0 empty, 1
black, 2 white, and the enumeration runs over `0 ≤ code < 3 ^ (m * n)`. Every
position appears, legal and illegal alike.
-/

namespace Superko
namespace Oracle

variable {m n : ℕ}

/-! ## Rendering -/

/-- The glyph a cell crosses the file boundary as. -/
private def glyph : Option Color → Char
  | none => '.'
  | some .black => 'X'
  | some .white => 'O'

/-- A position as its rows over `.`, `X`, `O`, separated by `/`. -/
private def renderPos (m n : ℕ) (b : Position m n) : String :=
  String.intercalate "/"
    ((List.finRange m).map fun r =>
      String.ofList ((List.finRange n).map fun c => glyph (b (r, c))))

/-- A color as `b` or `w`. -/
private def renderColor : Color → String
  | .black => "b"
  | .white => "w"

/-- A point as `row,col`, zero-based. -/
private def renderPoint (p : Point m n) : String := s!"{p.1.val},{p.2.val}"

/-- A move as `pass` or `row,col`. -/
private def renderMove : Move m n → String
  | .pass => "pass"
  | .play p => renderPoint p

/-- A move sequence, `;`-separated; the empty sequence is `root` so that every
line has the same number of whitespace-separated fields. -/
private def renderPath (path : List (Move m n)) : String :=
  match path with
  | [] => "root"
  | _ => String.intercalate ";" (path.map renderMove)

/-- The position whose base-3 code is `code`: digit `row * n + col`, with 0
empty, 1 black and 2 white. -/
private def posOfCode (m n : ℕ) (code : ℕ) : Position m n :=
  fun p =>
    match (code / 3 ^ (p.1.val * n + p.2.val)) % 3 with
    | 1 => some Color.black
    | 2 => some Color.white
    | _ => none

/-- Every position of the board, in code order. -/
private def allPositions (m n : ℕ) : List (Position m n) :=
  (List.range (3 ^ (m * n))).map (posOfCode m n)

/-- Both colors, in the order every table below uses. -/
private def bothColors : List Color := [Color.black, Color.white]

/-- Every point, row-major — the order `allMoves` plays in and the order a
position's string form is written in. -/
private def allPoints (m n : ℕ) : List (Point m n) :=
  (List.finRange m ×ˢ List.finRange n)

/-! ## The board tables -/

/-- `resolve'` at every position, color and point. -/
private def resolveTable (m n : ℕ) : List String :=
  (allPositions m n).flatMap fun b =>
    bothColors.flatMap fun c =>
      (allPoints m n).map fun p =>
        String.intercalate " "
          ["resolve", renderPos m n b, renderColor c, renderPoint p,
            renderPos m n (resolve' b c p)]

/-- The two conjuncts of `PlayableAt'`, separately, at every position, color
and point. -/
private def playableTable (m n : ℕ) : List String :=
  (allPositions m n).flatMap fun b =>
    bothColors.flatMap fun c =>
      (allPoints m n).map fun p =>
        let isEmpty := decide (b p = none)
        let liberty := decide (HasLiberty (resolve' b c p) p)
        String.intercalate " "
          ["playable", renderPos m n b, renderColor c, renderPoint p,
            (if isEmpty then "1" else "0"), (if liberty then "1" else "0")]

/-- `area'` for both colors at every position. -/
private def areaTable (m n : ℕ) : List String :=
  (allPositions m n).map fun b =>
    String.intercalate " "
      ["area", renderPos m n b, toString (area' b Color.black),
        toString (area' b Color.white)]

/-! ## The play traces -/

/-- One line per move of `allMoves` at a reached state. -/
private def stateLines (rule : String) (L' : State' m n → Move m n → Bool)
    (depth : ℕ) (path : List (Move m n)) (s : State' m n) : List String :=
  (allMoves m n).map fun mv =>
    String.intercalate " "
      ["play", rule, toString depth, renderPath path, renderPos m n s.now.board,
        renderColor s.now.toMove, renderMove mv,
        (if L' s mv then "1" else "0"), toString s.passes, toString s.seen.card]

/-- Every state reachable from the frontier by at most `levels - 1` further
legal moves, each contributing its own lines. Recursion stops at a state where
`Ended'` holds; that state's own lines are still emitted. -/
private def traceLevels (rule : String) (L' : State' m n → Move m n → Bool) :
    ℕ → ℕ → List (List (Move m n) × State' m n) → List String
  | 0, _, _ => []
  | 1, depth, frontier =>
      frontier.flatMap fun entry => stateLines rule L' depth entry.1 entry.2
  | levels + 1, depth, frontier =>
      let here := frontier.flatMap fun entry => stateLines rule L' depth entry.1 entry.2
      let next := frontier.flatMap fun entry =>
        if Ended' entry.2 then []
        else ((allMoves m n).filter (L' entry.2)).map fun mv =>
          (entry.1 ++ [mv], step' entry.2 mv)
      here ++ traceLevels rule L' levels (depth + 1) next

/-- The play table for one rule, from `start'` on the empty board with Black to
move, to a depth of `playDepth` moves. -/
private def playTable (m n : ℕ) (rule : String) (L' : State' m n → Move m n → Bool)
    (playDepth : ℕ) : List String :=
  traceLevels rule L' (playDepth + 1) 0 [([], start' (fun _ => none) Color.black)]

/-! ## The game counter

A fuel-indexed count of the games from a state, by levels: a state where
`Ended'` holds is one game and is not expanded, and every other state expands
to the successors its rule permits, in `allMoves` order. `none` reports fuel
exhausted with states still live, which would be a silent undercount if it
were reported as a number. The fuel `playMeasure'` supplies at the root is
`4 * 3 ^ (m * n)` (`playMeasure'_start`), and `4 * 3 ^ (m * n) + 1` levels is
that bound with the root's own level added. -/
private def countLevel (L' : State' m n → Move m n → Bool) :
    ℕ → List (State' m n) → Option ℕ
  | _, [] => some 0
  | 0, _ :: _ => none
  | fuel + 1, frontier =>
      let ended := frontier.filter fun s => Ended' s
      let live := frontier.filter fun s => !Ended' s
      let next := live.flatMap fun s => ((allMoves m n).filter (L' s)).map (step' s)
      (countLevel L' fuel next).map fun rest => ended.length + rest

/-- Games from the empty board under one rule. -/
private def countGames (m n : ℕ) (L' : State' m n → Move m n → Bool) : Option ℕ :=
  countLevel L' (4 * 3 ^ (m * n) + 1) [start' (fun _ => none) Color.black]

/-! ## Emission -/

private def emit (lines : List String) : IO Unit :=
  match lines with
  | [] => pure ()
  | _ => IO.println (String.intercalate "\n" lines)

/-- How deep the play traces run, by the number of points on the board.

These figures are measured rather than chosen, and they are the reason the
`play` table thins out as the board grows. Walking the game tree through the
computable rules, in the interpreter that `lean --run` starts, costs about ten
to twenty times more per extra ply, and the constant grows with the board as
well. Measured on this toolchain (`observed`, wall clock, whole file):

| board | play depth | cost |
|---|---|---|
| 1x3 | 4 | 23 s |
| 1x3 | 6 | about 8 minutes per rule |
| 1x4 | 2 | 18 s |
| 1x4 | 3 | 300 s |
| 1x5 | 0 | 20 s |
| 2x3 | 0 | 340 s |

`docs/plans/rules-mirror-plan.md` asks for six plies wherever `m * n ≤ 3` and
four above it. That is affordable at two points and not at six, so the depth is
a function of the board and the figure a file was produced at is printed in its
header, where no reader has to infer it.

The `resolve`, `playable` and `area` tables are complete on every board
regardless: they cover every one of the `3 ^ (m * n)` positions, and they are
what a mirror is most likely to get wrong. -/
private def playDepthFor (points : ℕ) : ℕ :=
  if points ≤ 2 then 6
  else if points = 3 then 4
  else if points = 4 then 2
  else if points = 5 then 1
  else 0

/-- The largest board the `count` table covers.

Also measured, and the same cost at work. A game on the 1x3 board runs to
twenty-one plies (`computed`, by the Rust mirror), and the walk does not reach
that depth here at any figure this project was willing to wait for: the
depth-5 prefix of the 1x3 tree takes about thirteen seconds and the depth-6
prefix does not finish in ten minutes. The two-point boards finish in under a
second. A count is emitted where it can be produced and omitted where it
cannot; a depth-capped number is not a count of games and is not offered as
one. -/
private def countLimit : ℕ := 2

/-- Every table for one board, at a given play-trace depth. -/
private def emitBoard (m n : ℕ) (depth : ℕ) : IO UInt32 := do
  let points := m * n
  IO.println s!"# play-depth: {depth}"
  IO.println s!"# positions: {3 ^ points}"
  IO.println (if points ≤ countLimit then "# count-table: yes"
    else s!"# count-table: no (measured out of reach at {points} points; see the README)")
  emit (resolveTable m n)
  emit (playableTable m n)
  emit (areaTable m n)
  emit (playTable m n "ssk" SSK' depth)
  emit (playTable m n "psk" PSK' depth)
  if points ≤ countLimit then
    for entry in [("ssk", countGames m n SSK'), ("psk", countGames m n PSK')] do
      match entry.2 with
      | none =>
          IO.eprintln s!"oracle: the game count under {entry.1} exhausted its fuel"
          return 1
      | some g => IO.println s!"count {entry.1} {g}"
  return 0

/-- `lake env lean --run lean/Oracle.lean <m> <n>`. -/
def main (args : List String) : IO UInt32 := do
  match args with
  | [ms, ns] => board ms ns none
  | [ms, ns, ds] => board ms ns ds.toNat?
  | _ =>
      IO.eprintln "usage: lake env lean --run lean/Oracle.lean <m> <n> [<play-depth>]"
      return 1
where
  /-- The board named by two decimal arguments, at the default play depth or at
  one the caller overrides for a measurement. -/
  board (ms ns : String) (over : Option ℕ) : IO UInt32 := do
    match ms.toNat?, ns.toNat? with
    | some m, some n =>
        if m = 0 || n = 0 then
          IO.eprintln "oracle: both board dimensions must be positive"
          return 1
        else
          emitBoard m n (over.getD (playDepthFor (m * n)))
    | _, _ =>
        IO.eprintln "oracle: the first two arguments are the board dimensions m and n"
        return 1

end Oracle
end Superko

/-- The entry point `lean --run` looks for. -/
def main (args : List String) : IO UInt32 := Superko.Oracle.main args
