/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Results.C29_DeciderCorrect
import Mathlib.Data.Rat.Floor

/-!
# C-27 — komi matters only through its floor

Komi in `Defs.lean` is a rational, and a rational is an infinite input. C-27
says the game does not see all of it. The winner of a finished board compares
the area difference — an integer — against komi, so two komi with the same floor
settle every board the same way, and `BlackWins` inherits the insensitivity. In
Go terms: moving komi from 6.5 to 6.9 changes nothing, and moving it from 6.5
to 7.5 is the only kind of change that can.

The file proves the statement twice over, at two levels.

**At the `winner` level** it is arithmetic. `winner_eq_of_floor_eq` turns
`0 < margin b k` into `⌊k⌋ < area b .black - area b .white` by `Int.floor_lt`,
after which the floor is all that is left of komi. `winner_congr_floor` proves
the same statement a second way, through the decider's integer leaf
(`winnerZ_eq_winner`) rather than through `Int.floor_lt`; the two routes are
independent and agree.

**At the `BlackWins` level** it comes from C-29. `BlackWins m n b komi` is
equivalent to a decider run whose only use of komi is `⌊komi⌋`, so the lift is
a rewrite on both sides of `C29_decideWins_iff_blackWins` rather than an
induction on the `WinsFor` derivation.

## What the tie convention costs

`Defs.lean` awards a tie — margin exactly 0 — to White, which is OPEN-3 in
`docs/formal-model.md`. `C27_blackWins_iff_halfInteger` shows the convention is
**unobservable for `BlackWins`**: every komi is equivalent to a half-integer
one, at which no tie arises. That is a statement about one fixed color asking
one fixed question. It is *not* a statement that the convention is harmless in
general — at margin exactly 0 the convention decides the board, so swapping the
colors while holding the komi fixed is observable, and a hardness construction
that relies on symmetry between the colors at margin 0 would see it. Nothing
here rules that out; the scope of the claim is `BlackWins` at Black.

## What this file does not establish

* **The komi clamp is proved only at the `winner` level.**
  `winner_const_of_floor_ge` shows that once `⌊komi⌋` reaches `m * n` the
  winner is White whatever the board, which together with
  `area_add_area_le` bounds the interesting floors to `[-(m·n)-1, m·n]` —
  `2·m·n + 2` classes. The corresponding statement about `WinsFor` — that the
  game, not merely the score, is constant outside that window — is **open**
  (C-35). `C27_blackWins_iff_halfInteger` normalizes komi to a half-integer but
  does not bound it.
* **Situational superko only.** `C27_blackWins_iff_halfInteger` is about
  `BlackWins`, which is `WinsFor … SSK …`. The positional superko twins are not
  written here, though C-29's PSK half would supply them the same way.
* Nothing about Go beyond what `Defs.lean` says Go is, and in particular nothing
  about whether area scoring without dead-stone determination is the right
  scoring rule (C-16).
-/

namespace Superko

variable {m n : ℕ}

/-! ## The score sees only the floor -/

/-- Black's and White's area sets are disjoint, so the areas sum to at most the
number of points on the board: a point is a Black stone, a White stone, a point
reaching exactly one color, or a dame reaching both, and only the first three
count for anybody.

This is what bounds the margin's integer part, and so the number of komi
classes. -/
lemma area_add_area_le (b : Position m n) :
    area b .black + area b .white ≤ m * n := by
  classical
  have hdisj : Disjoint
      {p : Point m n | b p = some Color.black ∨
        (b p = none ∧ Reaches b p Color.black ∧ ¬ Reaches b p Color.white)}
      {p : Point m n | b p = some Color.white ∨
        (b p = none ∧ Reaches b p Color.white ∧ ¬ Reaches b p Color.black)} := by
    rw [Set.disjoint_left]
    intro p hp hq
    rcases hp with hp | ⟨hp, hpb, hpw⟩
    · rcases hq with hq | ⟨hq, _, _⟩
      · rw [hp] at hq; exact absurd hq (by decide)
      · rw [hp] at hq; exact absurd hq (by simp)
    · rcases hq with hq | ⟨_, hqw, _⟩
      · rw [hp] at hq; exact absurd hq (by simp)
      · exact hpw hqw
  have hcard : Fintype.card (Point m n) = m * n := by
    simp [Point]
  unfold area
  calc _ = _ := (Set.ncard_union_eq hdisj (Set.toFinite _) (Set.toFinite _)).symm
    _ ≤ Fintype.card (Point m n) := by
        rw [Set.ncard_eq_toFinset_card']
        exact Finset.card_le_univ _
    _ = m * n := hcard

/-- **C-27, at the score.** Who won a finished board depends on komi only
through `⌊komi⌋`: the area difference is an integer, and `Int.floor_lt` turns
the rational comparison into an integer one. -/
lemma winner_eq_of_floor_eq (b : Position m n) (k k' : ℚ)
    (h : ⌊k⌋ = ⌊k'⌋) : winner b k = winner b k' := by
  have key : ∀ q : ℚ, (0 < margin b q) ↔ ⌊q⌋ < (area b .black : ℤ) - (area b .white : ℤ) := by
    intro q
    rw [Int.floor_lt]
    unfold margin
    push_cast
    constructor <;> intro hq <;> linarith
  simp only [winner, key k, key k', h]

/-- Hence every komi is equivalent, for the purpose of `winner`, to the
half-integer just above its floor — so the tie convention never bites at the
score. -/
lemma winner_eq_halfInteger (b : Position m n) (k : ℚ) :
    winner b k = winner b ((⌊k⌋ : ℚ) + 1/2) := by
  refine winner_eq_of_floor_eq b k _ ?_
  rw [Int.floor_intCast_add]
  norm_num

/-- The komi clamp, at the score. The area difference is at most `m * n`, so a
komi whose floor reaches `m * n` hands every board to White, and only the
`2 * m * n + 2` floors in `[-(m·n)-1, m·n]` can distinguish anything.

The lift of this statement from `winner` to `WinsFor` — that the *game* is
constant outside the window — is open (C-35). -/
lemma winner_const_of_floor_ge (b : Position m n) (k : ℚ)
    (h : (m * n : ℤ) ≤ ⌊k⌋) : winner b k = Color.white := by
  have hb := area_add_area_le b
  have key : ¬ (0 < margin b k) := by
    have h1 : (m * n : ℚ) ≤ k := by
      have := Int.le_floor.mpr (le_refl ⌊k⌋)
      have h2 : ((⌊k⌋ : ℚ)) ≤ k := Int.floor_le k
      have h3 : ((m * n : ℤ) : ℚ) ≤ ((⌊k⌋ : ℤ) : ℚ) := by exact_mod_cast h
      push_cast at h3 ⊢
      linarith
    unfold margin
    have : (area b Color.black : ℚ) ≤ (m * n : ℚ) := by
      have : area b Color.black ≤ m * n := by omega
      exact_mod_cast this
    linarith [Nat.cast_nonneg (α := ℚ) (area b Color.white)]
  unfold winner
  simp [key]

/-! ## The game sees only the floor

C-29's decider consumes `⌊komi⌋`, so the `BlackWins`-level clauses are rewrites
on its statement rather than inductions on `WinsFor`. -/

/-- `winner` sees komi only through its floor, proved a second time through the
decider's integer leaf. Independent of `winner_eq_of_floor_eq`, which proves the
same statement from `Int.floor_lt`; the two routes agree. -/
theorem winner_congr_floor (b : Position m n) (k₁ k₂ : ℚ) (h : ⌊k₁⌋ = ⌊k₂⌋) :
    winner b k₁ = winner b k₂ := by
  rw [← winnerZ_eq_winner, ← winnerZ_eq_winner, h]

/-- And so does `BlackWins` — C-27's `WinsFor`-level clause, obtained from C-29
by rewriting rather than by induction on the `WinsFor` derivation. -/
theorem blackWins_congr_floor (b : Position m n) (k₁ k₂ : ℚ) (h : ⌊k₁⌋ = ⌊k₂⌋) :
    BlackWins m n b k₁ ↔ BlackWins m n b k₂ := by
  rw [← C29_decideWins_iff_blackWins, ← C29_decideWins_iff_blackWins, h]

/-- **C-27.** Every komi is equivalent to a half-integer one, so the tie
convention of `Defs.lean`'s `winner` (OPEN-3) is unobservable for `BlackWins`.

At a half-integer komi no tie arises, so the question `BlackWins m n b komi`
never depends on who a tie is awarded to. Under a color swap at margin 0 the
convention *is* observable, and nothing here says otherwise. -/
theorem C27_blackWins_iff_halfInteger (b : Position m n) (komi : ℚ) :
    BlackWins m n b komi ↔ BlackWins m n b ((⌊komi⌋ : ℚ) + 1/2) := by
  refine blackWins_congr_floor b komi _ ?_
  symm
  rw [Int.floor_eq_iff]
  refine ⟨by linarith, by linarith⟩

end Superko
