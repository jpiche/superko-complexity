/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Results.C13_Termination
import Mathlib.Data.Rat.Floor

/-!
# The computable game layer

A decider for `Superko.BlackWins` that the kernel can run, assembled from
`Defs.lean`'s notions without touching them. Every declaration here is a
derived notion in the sense of `docs/trusted-base.md`: the audit target is
unchanged, and what a reader must add to their reading is the *statements* of
the lemmas marked **Bridge.** below. Each equates a computable twin to a core
definition, and a check that does not route through them says nothing about Go.

`Defs.lean`'s `State` carries `seen : Set (Situation m n)`, and `playMeasure`
counts that set with `Set.ncard`, so neither evaluates. `State'` carries a
`Finset` instead, `step'` inserts into it, `Ended'` is a `Bool`, and
`State'.toState` maps a computable state back to the one the rules are stated
over.

The decider is parametric in the repetition rule: it takes a `Bool`-valued
`L'` together with `Faithful L' L`, and is instantiated at both `SSK'` and
`PSK'`. The correctness proofs know nothing about which rule they are running.

## Why the leaf test is an integer

`winner` asks whether `margin b komi` is positive, and `margin` is a
subtraction in ℚ. `Rat.sub` is `@[irreducible]` in Lean core at this toolchain
(`Init/Data/Rat/Basic.lean:295`), as are `Rat.add` (:259), `Rat.mul` (:168),
`Rat.inv` (:191) and `Rat.ofScientific` (:132). A decider whose leaf computes
`margin` therefore does not reduce in the kernel at *any* komi — not at a
half-integer one, and not at `0` either, because the obstruction is the
subtraction and not the literal.

`winnerZ` asks the same question over ℤ, comparing `⌊komi⌋` against the
difference of the two areas, and `winnerZ_eq_winner` proves it computes
`winner`. It is the only leaf test in this file because it is the only
kernel-reducible one. The price appears in every statement downstream: the
decider is handed `⌊komi⌋` and concludes about `komi`. `Rat.inv` being
irreducible has a second consequence — a komi literal written `1/2` has no
kernel normal form either, so the integer the decider runs at must be produced
by a proof rather than by reduction.

## Why fuel rather than well-founded recursion

C-13 supplies a well-founded relation, and `decideWins` does not use it: a
definition by `WellFounded.fix` does not reduce in the kernel, which is the
whole point of this file. Recursion on a `ℕ` fuel argument reduces, and costs
two lemmas instead — `decideWins_mono`, to raise fuel already spent, and
`playMeasure'_start`, to show that C-26's bound supplies enough of it.

## What the kernel actually reaches

Reduction of this decider is exponential in the board size, and `List.any` and
`List.all` short-circuit, so how far it reaches depends on the board and the
komi and not on `m * n` alone. Measured on this toolchain (`computed`): the
checks in `Results/C29_DeciderCorrect.lean` run at `m * n ≤ 2` in under a
second, the empty 1×4 and 2×2 boards each decide in seconds, and the empty 2×3
board exceeds the default heartbeat budget and does not finish with that budget
lifted.

Deciding a winner is not counting games. The decider contributes nothing to the
validation of the published 2×2 game count (C-9): that count is a search for the
Rust side to produce and a certificate checker to verify, and no run of this
decider enumerates games at all.
-/

namespace Superko

variable {m n : ℕ}

/-! ## Decidable equality on positions and situations -/

/-- Two boards are equal when they carry the same stone at every point, which
is decidable because there are finitely many points. -/
instance decEqPosition : DecidableEq (Position m n) := Fintype.decidablePiFintype

/-- Two situations are equal when the boards agree and the same player is to
move. -/
instance decEqSituation : DecidableEq (Situation m n) := fun s t =>
  decidable_of_iff (s.board = t.board ∧ s.toMove = t.toMove)
    (by cases s; cases t; constructor <;> intro h <;> simp_all)

/-! ## Moves -/

/-- Every move available on the board, legal or not: the pass, and a play at
each of the `m * n` points. Legality is a separate question, asked by the
repetition rule. -/
def allMoves (m n : ℕ) : List (Move m n) :=
  Move.pass :: (List.finRange m ×ˢ List.finRange n).map Move.play

/-- The enumeration misses nothing: a branch over `allMoves` is a branch over
all moves. -/
theorem mem_allMoves (mv : Move m n) : mv ∈ allMoves m n := by
  cases mv with
  | pass => simp [allMoves]
  | play p => obtain ⟨i, j⟩ := p; simp [allMoves, List.mem_product]

/-- The situation a move leads to, computed: a pass hands the turn over, a play
places the stone and removes the opponent's chains that lose their last
liberty. -/
def afterC (s : Situation m n) : Move m n → Situation m n
  | .pass => ⟨s.board, s.toMove.other⟩
  | .play p => ⟨resolve' s.board s.toMove p, s.toMove.other⟩

/-- **Bridge.** `afterC` computes `Situation.after`. -/
theorem afterC_eq_after (s : Situation m n) (mv : Move m n) :
    afterC s mv = s.after mv := by
  cases mv <;> simp [afterC, Situation.after, resolve'_eq_resolve]

/-! ## The computable state of play -/

/-- A state of play whose archive is a `Finset`: the situation now, every
situation the game has passed through, and the number of passes immediately
preceding. This is what a machine playing under superko would actually carry. -/
structure State' (m n : ℕ) where
  /-- The situation now. -/
  now : Situation m n
  /-- Every situation that has occurred, `now` included — the superko archive. -/
  seen : Finset (Situation m n)
  /-- How many passes immediately precede. Two ends the game. -/
  passes : ℕ

/-- The state of play that a computable state represents. -/
def State'.toState (s : State' m n) : State m n := ⟨s.now, ↑s.seen, s.passes⟩

/-- Making a move: advance the situation, add it to the archive, and count the
pass or reset the pass counter. -/
def step' (s : State' m n) (mv : Move m n) : State' m n :=
  { now := afterC s.now mv
    seen := insert (afterC s.now mv) s.seen
    passes := match mv with | .pass => s.passes + 1 | .play _ => 0 }

/-- **Bridge.** `step'` computes `step`. -/
theorem step'_toState (s : State' m n) (mv : Move m n) :
    (step' s mv).toState = step s.toState mv := by
  cases mv <;>
    simp [State'.toState, step', step, afterC_eq_after, Finset.coe_insert]

/-- The game has ended: two consecutive passes. -/
def Ended' (s : State' m n) : Bool := decide (2 ≤ s.passes)

/-- **Bridge.** `Ended'` decides `Ended`. -/
theorem ended'_iff (s : State' m n) : Ended' s = true ↔ Ended s.toState := by
  simp [Ended', Ended, State'.toState]

/-! ## The leaf test -/

/-- Who won, decided over ℤ: Black wins when `⌊komi⌋` is below the difference
of the two area scores, and a tie goes to White as in `winner`.

`winner` compares a rational margin against zero, and a rational subtraction
does not reduce in the kernel; this comparison of integers does. -/
def winnerZ (b : Position m n) (k : ℤ) : Color :=
  if k < (area' b .black : ℤ) - (area' b .white : ℤ) then .black else .white

/-- **Bridge.** The integer leaf test at `⌊komi⌋` computes `winner` at `komi`:
`winner` sees the komi only through its floor. -/
theorem winnerZ_eq_winner (b : Position m n) (komi : ℚ) :
    winnerZ b (Int.floor komi) = winner b komi := by
  have key : (0 < margin b komi) ↔
      Int.floor komi < (area b .black : ℤ) - (area b .white : ℤ) := by
    rw [Int.floor_lt]; unfold margin; push_cast
    constructor <;> intro hq <;> linarith
  simp only [winnerZ, winner, area'_eq_area, key]

/-! ## The decider -/

/-- A `Bool`-valued repetition rule is **faithful** to `L` when it decides `L`
on the states it represents. This is what makes a run of the decider a
statement about the rule `Defs.lean` defines. -/
def Faithful (L' : State' m n → Move m n → Bool) (L : Repetition m n) : Prop :=
  ∀ (s : State' m n) (mv : Move m n), L' s mv = true ↔ L s.toState mv

/-- The archive decider: play out the game tree, carrying the superko archive,
to a depth of `fuel` moves. At an ended game the leaf test says whether `c`
won; otherwise `c` needs one legal move to a won state on its own turn, and
every legal move to lead to one on the opponent's. -/
def decideWins (komi : ℤ) (c : Color) (L' : State' m n → Move m n → Bool) :
    ℕ → State' m n → Bool
  | 0, _ => false
  | fuel + 1, s =>
      if Ended' s then decide (winnerZ s.now.board komi = c)
      else if s.now.toMove = c then
        (allMoves m n).any fun mv => L' s mv && decideWins komi c L' fuel (step' s mv)
      else
        (allMoves m n).all fun mv => !(L' s mv) || decideWins komi c L' fuel (step' s mv)

/-- **Soundness.** What the decider accepts, `c` really wins. No bound on the
fuel is needed: a decider run that returns `true` has exhibited a strategy. -/
theorem decideWins_sound {komi : ℚ} {c : Color} {L' : State' m n → Move m n → Bool}
    {L : Repetition m n} (hL : Faithful L' L) :
    ∀ (fuel : ℕ) (s : State' m n),
      decideWins ⌊komi⌋ c L' fuel s = true → WinsFor m n L komi c s.toState := by
  intro fuel
  induction fuel with
  | zero => intro s h; simp [decideWins] at h
  | succ k ih =>
      intro s h
      rw [decideWins] at h
      split at h
      · rename_i hend
        exact WinsFor.ended ((ended'_iff s).mp hend)
          (by simpa [winnerZ_eq_winner, State'.toState] using h)
      · rename_i hend
        split at h
        · rename_i hturn
          obtain ⟨mv, _, hmv⟩ := List.any_eq_true.mp h
          rw [Bool.and_eq_true] at hmv
          refine WinsFor.mover (fun hc => hend ((ended'_iff s).mpr hc)) hturn
            ((hL s mv).mp hmv.1) ?_
          have := ih (step' s mv) hmv.2
          rwa [step'_toState] at this
        · rename_i hturn
          refine WinsFor.waiter (fun hc => hend ((ended'_iff s).mpr hc)) hturn ?_
          intro mv hmv
          have hall := List.all_eq_true.mp h mv (mem_allMoves mv)
          rw [Bool.or_eq_true, Bool.not_eq_true'] at hall
          rcases hall with hneg | hpos
          · exact absurd ((hL s mv).mpr hmv) (by simp [hneg])
          · have := ih (step' s mv) hpos
            rwa [step'_toState] at this

/-! ## Enough fuel

Completeness needs a fuel figure, and C-13's measure supplies it once it is
made computable. -/

/-- Computable measure: twice the unvisited situations, plus passes remaining. -/
def playMeasure' (s : State' m n) : ℕ :=
  2 * (Fintype.card (Situation m n) - s.seen.card) + (2 - s.passes)

/-- The computable measure is C-13's measure. -/
theorem playMeasure'_eq (s : State' m n) :
    playMeasure' s = playMeasure s.toState := by
  simp [playMeasure', playMeasure, State'.toState]

/-- Fuel already sufficient stays sufficient. Needed because the recursive call
in the completeness proof is made at the measure of the successor state, which
is smaller than the fuel the statement hands it. -/
theorem decideWins_mono {komi : ℤ} {c : Color} {L' : State' m n → Move m n → Bool} :
    ∀ (f g : ℕ), f ≤ g → ∀ s : State' m n,
      decideWins komi c L' f s = true → decideWins komi c L' g s = true := by
  intro f
  induction f with
  | zero => intro g _ s h; simp [decideWins] at h
  | succ k ih =>
      intro g hg s h
      obtain ⟨j, rfl⟩ : ∃ j, g = j + 1 := ⟨g - 1, by omega⟩
      rw [decideWins] at h ⊢
      split at h <;> rename_i hc
      · simp only [hc, ite_true]; exact h
      · simp only [hc] at h ⊢
        split at h <;> rename_i ht
        · simp only [ht, ite_true] at h ⊢
          obtain ⟨mv, hmem, hmv⟩ := List.any_eq_true.mp h
          rw [Bool.and_eq_true] at hmv
          exact List.any_eq_true.mpr ⟨mv, hmem, by simp [hmv.1, ih j (by omega) _ hmv.2]⟩
        · simp only [ht, ite_false] at h ⊢
          refine List.all_eq_true.mpr fun mv hmem => ?_
          have hx := List.all_eq_true.mp h mv hmem
          rw [Bool.or_eq_true] at hx ⊢
          rcases hx with hn | hp
          · exact Or.inl hn
          · exact Or.inr (ih j (by omega) _ hp)

/-- **Completeness**, generalized so that the `WinsFor` index is a variable —
`induction` on the derivation is rejected at `s.toState`. -/
theorem decideWins_complete_aux {komi : ℚ} {c : Color}
    {L' : State' m n → Move m n → Bool} {L : Repetition m n}
    (hL : Faithful L' L) (hLr : ExcludesRepeats L) :
    ∀ (st : State m n), WinsFor m n L komi c st →
      ∀ s : State' m n, s.toState = st →
        decideWins ⌊komi⌋ c L' (playMeasure' s + 1) s = true := by
  intro st hw
  induction hw with
  | @ended st hend hwin =>
      intro s hs
      rw [decideWins]
      have he : Ended' s = true := (ended'_iff s).mpr (hs ▸ hend)
      simp only [he, ite_true, decide_eq_true_eq]
      rw [show s.now.board = st.now.board from by rw [← hs]; rfl] at *
      simpa [winnerZ_eq_winner] using hwin
  | @mover st mv hend hturn hmv hrec ih =>
      intro s hs
      rw [decideWins]
      have he : Ended' s = false := by
        simp only [Ended', decide_eq_false_iff_not]
        intro hc; exact hend (hs ▸ (show Ended s.toState from hc))
      have ht : s.now.toMove = c := by rw [show s.now = st.now from by rw [← hs]; rfl]; exact hturn
      simp only [he, Bool.false_eq_true, ite_false, ht, ite_true]
      refine List.any_eq_true.mpr ⟨mv, mem_allMoves mv, ?_⟩
      rw [Bool.and_eq_true]
      refine ⟨(hL s mv).mpr (hs ▸ hmv), ?_⟩
      have hstep : (step' s mv).toState = step st mv := by rw [step'_toState, hs]
      have hdrop : playMeasure' (step' s mv) < playMeasure' s := by
        rw [playMeasure'_eq, playMeasure'_eq, hstep, hs]
        exact playMeasure_lt_of_follows hLr ⟨hend, mv, hmv, rfl⟩
      exact decideWins_mono _ _ (by omega) _ (ih (step' s mv) hstep)
  | @waiter st hend hturn hall ih =>
      intro s hs
      rw [decideWins]
      have he : Ended' s = false := by
        simp only [Ended', decide_eq_false_iff_not]
        intro hc; exact hend (hs ▸ (show Ended s.toState from hc))
      have ht : ¬ (s.now.toMove = c) := by
        rw [show s.now = st.now from by rw [← hs]; rfl]; exact hturn
      simp only [he, Bool.false_eq_true, ite_false, ht]
      refine List.all_eq_true.mpr fun mv _ => ?_
      rw [Bool.or_eq_true]
      by_cases hmv : L' s mv = true
      · right
        have hmvL : L st mv := hs ▸ (hL s mv).mp hmv
        have hstep : (step' s mv).toState = step st mv := by rw [step'_toState, hs]
        have hdrop : playMeasure' (step' s mv) < playMeasure' s := by
          rw [playMeasure'_eq, playMeasure'_eq, hstep, hs]
          exact playMeasure_lt_of_follows hLr ⟨hend, mv, hmvL, rfl⟩
        exact decideWins_mono _ _ (by omega) _ (ih mv hmvL (step' s mv) hstep)
      · left; simp [Bool.eq_false_iff.mpr hmv]

/-! ## The root of play, and the two superko rules -/

/-- Play begins from a position with nothing forbidden but that position
itself — encoding (C) of `docs/formal-model.md` §5, computably. -/
def start' (b : Position m n) (c : Color) : State' m n :=
  { now := ⟨b, c⟩, seen := {⟨b, c⟩}, passes := 0 }

/-- **Bridge.** `start'` computes `start`. -/
theorem start'_toState (b : Position m n) (c : Color) :
    (start' b c).toState = start b c := by simp [State'.toState, start', start]

/-- Situational superko, computed: a pass is always legal, and a play must be
playable on the board and must not recreate an archived situation. -/
def SSK' (s : State' m n) (mv : Move m n) : Bool :=
  match mv with
  | .pass => true
  | .play p => decide (PlayableAt' s.now.board s.now.toMove p) &&
      decide (afterC s.now (.play p) ∉ s.seen)

/-- **Bridge.** `SSK'` decides `SSK`. -/
theorem ssk'_faithful : Faithful (SSK' (m := m) (n := n)) SSK := by
  intro s mv
  cases mv with
  | pass => simp [SSK', SSK]
  | play p =>
      have hp : PlayableAt' s.now.board s.now.toMove p ↔ PlayableAt s.toState p :=
        playableAt'_iff s.toState p
      simp only [SSK', SSK, Bool.and_eq_true, decide_eq_true_eq, State'.toState,
        afterC_eq_after, Finset.mem_coe]
      exact and_congr hp Iff.rfl

/-- Positional superko, computed: the same, except that the archive is consulted
for the board alone, whoever was to move. -/
def PSK' (s : State' m n) (mv : Move m n) : Bool :=
  match mv with
  | .pass => true
  | .play p => decide (PlayableAt' s.now.board s.now.toMove p) &&
      decide (∀ t ∈ s.seen, t.board ≠ (afterC s.now (.play p)).board)

/-- **Bridge.** `PSK'` decides `PSK`. -/
theorem psk'_faithful : Faithful (PSK' (m := m) (n := n)) PSK := by
  intro s mv
  cases mv with
  | pass => simp [PSK', PSK]
  | play p =>
      have hp : PlayableAt' s.now.board s.now.toMove p ↔ PlayableAt s.toState p :=
        playableAt'_iff s.toState p
      simp only [PSK', PSK, Bool.and_eq_true, decide_eq_true_eq, State'.toState,
        afterC_eq_after, Finset.mem_coe]
      exact and_congr hp Iff.rfl

/-- At the root of play the measure is C-26's bound: `4 * 3 ^ (m * n)`, the
fuel the decider needs. -/
theorem playMeasure'_start (b : Position m n) (c : Color) :
    playMeasure' (start' b c) = 4 * 3 ^ (m * n) := by
  rw [playMeasure'_eq, start'_toState, playMeasure_start, card_situation]
  omega

end Superko
