/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Decide

/-!
# C-29 — the archive decider decides SUPERKO-GO

**C-29.** The fuel-indexed archive decider of `SuperkoComplexity/Decide.lean`,
run with fuel `4 * 3 ^ (m * n) + 1` and the integer leaf test at `⌊komi⌋`,
returns `true` exactly when `BlackWins m n b komi` holds. Both directions are
proved here: soundness from `decideWins_sound`, completeness from
`decideWins_complete_aux` fuelled by `playMeasure'_start`. The same statement
holds under positional superko, because the decider is parametric in the
repetition rule and `PSK` also excludes repeats.

The `⌊komi⌋` is not an approximation. `winnerZ_eq_winner` proves that `winner`
sees the komi only through its floor, so the decider run at `⌊komi⌋` settles the
question at `komi` itself — and it has to be run that way, since a rational komi
has no kernel normal form at this toolchain.

## What this does not establish

* **Nothing about Go beyond what `Defs.lean` says Go is.** C-29 is a statement
  about `BlackWins`, and `docs/trusted-base.md` item 3 records that the
  faithfulness of those definitions is the one part of the audit that is not
  mechanical. The bridging lemmas this result routes through are marked
  **Bridge.** in `Decide.lean` and their statements join that audit.
* **Nothing about space, and nothing about complexity.** A decider that
  terminates is not a complexity bound. C-3 — SUPERKO-GO in EXPSPACE — remains
  `folklore`, and nothing here is an argument for it: the decider's archive is
  a `Finset` in the kernel, not a tape, and no resource measure is defined.
* **Nothing about search.** The decider is a proof object that happens to
  reduce. It walks the game tree with no move ordering, no memoization and no
  pruning beyond what `List.any` and `List.all` short-circuit, and kernel
  reduction reaches only the smallest boards — the spot checks below run at
  `m * n ≤ 2` (`computed`). The Rust side is where search lives. A winner is
  also not a count: C-9's 2×2 game count is not something any run of this
  decider produces.

## The fuel is C-26's constant

`4 * 3 ^ (m * n)` enters the statement through `playMeasure'_start`, which is
C-26's bound (`C13_length_bound_explicit`) made computable. The constant is
therefore not incidental to the statement: tightening C-26 — a lexicographic
measure would, as `C13_Termination.lean` notes — changes the fuel figure these
theorems are stated at, and they would have to be restated.
-/

namespace Superko

variable {m n : ℕ}

/-! ## The decider is correct -/

/-- **C-29.** With fuel `4 * 3 ^ (m * n) + 1` — the game-length bound of C-26 —
and the integer leaf test at `⌊komi⌋`, the archive decider decides
`BlackWins`.

Note the `⌊komi⌋`: the decider is not a decision procedure for `BlackWins` at a
komi handed to it directly. It is one for `BlackWins` at `komi` when run at
`⌊komi⌋`, and `winnerZ_eq_winner` is what makes that sound. -/
theorem C29_decideWins_iff_blackWins (b : Position m n) (komi : ℚ) :
    decideWins ⌊komi⌋ Color.black SSK' (4 * 3 ^ (m * n) + 1) (start' b Color.black) = true
      ↔ BlackWins m n b komi := by
  constructor
  · intro h
    have := decideWins_sound ssk'_faithful _ _ h
    rwa [start'_toState] at this
  · intro h
    have hc := decideWins_complete_aux ssk'_faithful ssk_excludesRepeats
      (start b Color.black) h (start' b Color.black) (start'_toState b Color.black)
    rwa [playMeasure'_start] at hc

/-- The same under positional superko — free, because the decider is
rule-parametric and `PSK` also excludes repeats. -/
theorem C29_decideWins_iff_blackWinsPSK (b : Position m n) (komi : ℚ) :
    decideWins ⌊komi⌋ Color.black PSK' (4 * 3 ^ (m * n) + 1) (start' b Color.black) = true
      ↔ BlackWinsPSK m n b komi := by
  constructor
  · intro h
    have := decideWins_sound psk'_faithful _ _ h
    rwa [start'_toState] at this
  · intro h
    have hc := decideWins_complete_aux psk'_faithful psk_excludesRepeats
      (start b Color.black) h (start' b Color.black) (start'_toState b Color.black)
    rwa [playMeasure'_start] at hc

/-- The same question when White moves first: the decider still asks whether
Black has a winning strategy, from a root of play at which it is `c`'s turn.
`BlackWins` is the case `c = Color.black`; this form is what a reduction
building positions with either player to move would consume. -/
theorem C29_decideWins_iff_winsFor_black (b : Position m n) (c : Color) (komi : ℚ) :
    decideWins ⌊komi⌋ Color.black SSK' (4 * 3 ^ (m * n) + 1) (start' b c) = true
      ↔ WinsFor m n SSK komi .black (start b c) := by
  constructor
  · intro h
    have := decideWins_sound ssk'_faithful _ _ h
    rwa [start'_toState] at this
  · intro h
    have hc := decideWins_complete_aux ssk'_faithful ssk_excludesRepeats
      (start b c) h (start' b c) (start'_toState b c)
    rwa [playMeasure'_start] at hc

/-! ## Spot checks

What the bridge buys: a `decide` that settles `BlackWins` — the `Defs.lean`
predicate, over a rational komi — rather than a fact about a decider. The komi
below are written the way Go komi are written, with a half point, and the floor
of each is discharged by `norm_num` because a rational literal does not reduce
in the kernel (`Rat.inv` is `@[irreducible]`; see `Decide.lean`). The decider
itself runs by `decide`, so the kernel performs every reduction and no axiom
beyond the three appears.

On 1×1 Black's only play is suicide, so both players must pass and the game ends
with both areas zero. Black's margin is `-komi`, and a tie goes to White. -/

/-- The empty 1×1 board. -/
def empty11 : Position 1 1 := fun _ => none

/-- The empty 1×2 board. -/
def empty12 : Position 1 2 := fun _ => none

/-- At komi 1/2 Black cannot win on the empty 1×1 board. -/
theorem not_blackWins_11_half : ¬ BlackWins 1 1 empty11 (1/2) := by
  intro h
  have h2 := (C29_decideWins_iff_blackWins empty11 (1/2 : ℚ)).mpr h
  rw [show (⌊(1/2 : ℚ)⌋ : ℤ) = 0 by norm_num] at h2
  have h3 : decideWins (m := 1) (n := 1) 0 Color.black SSK'
      (4 * 3 ^ (1 * 1) + 1) (start' empty11 Color.black) = false := by decide
  rw [h3] at h2; exact Bool.false_ne_true h2

/-- At komi -1/2 Black wins it. -/
theorem blackWins_11_neg_half : BlackWins 1 1 empty11 (-1/2) :=
  (C29_decideWins_iff_blackWins empty11 (-1/2 : ℚ)).mp
    (by rw [show (⌊(-1/2 : ℚ)⌋ : ℤ) = -1 by norm_num]; decide)

/-- Nor at komi 13/2 on the empty 1×2 board, where Black's best is two points. -/
theorem not_blackWins_12_six_half : ¬ BlackWins 1 2 empty12 (13/2) := by
  intro h
  have h2 := (C29_decideWins_iff_blackWins empty12 (13/2 : ℚ)).mpr h
  rw [show (⌊(13/2 : ℚ)⌋ : ℤ) = 6 by norm_num] at h2
  have h3 : decideWins (m := 1) (n := 2) 6 Color.black SSK'
      (4 * 3 ^ (1 * 2) + 1) (start' empty12 Color.black) = false := by decide
  rw [h3] at h2; exact Bool.false_ne_true h2

/-- At komi -1/2 Black wins on the empty 1×2 board. -/
theorem blackWins_12_neg_half : BlackWins 1 2 empty12 (-1/2) :=
  (C29_decideWins_iff_blackWins empty12 (-1/2 : ℚ)).mp
    (by rw [show (⌊(-1/2 : ℚ)⌋ : ℤ) = -1 by norm_num]; decide)

/-- And under positional superko, same board, same route. -/
theorem not_blackWinsPSK_12_half : ¬ BlackWinsPSK 1 2 empty12 (1/2) := by
  intro h
  have h2 := (C29_decideWins_iff_blackWinsPSK empty12 (1/2 : ℚ)).mpr h
  rw [show (⌊(1/2 : ℚ)⌋ : ℤ) = 0 by norm_num] at h2
  have h3 : decideWins (m := 1) (n := 2) 0 Color.black PSK'
      (4 * 3 ^ (1 * 2) + 1) (start' empty12 Color.black) = false := by decide
  rw [h3] at h2; exact Bool.false_ne_true h2

end Superko
