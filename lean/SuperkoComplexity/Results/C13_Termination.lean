/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Basic

/-!
# C-13 — play under superko terminates

The whole content of the claim is that one natural number strictly drops on
every legal move. Two quantities shrink and neither alone suffices:

* a **play** must reach a situation the history does not hold, so it enlarges
  `State.seen`, which is bounded by the finitely many situations — but a play
  also resets the pass counter, which makes the second summand *larger*;
* a **pass** leaves `seen` weakly larger and advances the pass counter, and two
  passes end the game, so no run of passes exceeds two.

`playMeasure` weights the first summand by two, which is exactly enough to
absorb the reset: a play spends at least one unit of the `2 * (unvisited)`
budget and is handed back at most two units of pass budget, and `¬ Ended`
leaves at least one of those two already spent at the source.

`docs/plans/first-results-plan.md` proposes a lexicographic measure on
(unvisited situations, pass counter). The weighting flattens that to `ℕ`, which
costs a factor of two in the length bound and saves the well-founded-order
plumbing. The factor is charged on every play though it is needed only to
absorb one reset, so a lexicographic measure would give a smaller constant;
`2 * Fintype.card (Situation m n)` is what this measure supports, exactly and
with nothing to spare.

## What this file does not establish

* Not determinacy. `C13_terminates` gives the well-founded relation a
  determinacy proof would recurse on; it is not that proof.
* Not that play must *reach* an ended state — only that it cannot go on
  forever. Nothing here says a legal move always exists.
* Nothing about Go beyond what `Defs.lean` says Go is. `Defs.lean` records its
  own definitions as plausible rather than validated, and the independent
  agreement argument of `docs/trusted-base.md` has not been run.

`C13_length_bound` bounds runs rooted at `start b c`, which is encoding (C) of
`docs/formal-model.md` §5. `playMeasure_add_le` gives the corresponding bound
from an arbitrary state if a later claim wants one.
-/

namespace Superko

/-! ## The measure -/

/-- Twice the number of situations still unvisited, plus the passes still
available.

`noncomputable` because `Set.ncard` is: `State.seen` is a `Set`, not a
`Finset`. A certificate checker cannot evaluate this, and any future `computed`
claim about measure values will need a decidable counterpart and a bridging
lemma, in the pattern `Basic.lean` uses for `clear'` and `area'`. -/
noncomputable def playMeasure {m n : ℕ} (st : State m n) : ℕ :=
  2 * (Fintype.card (Situation m n) - st.seen.ncard) + (2 - st.passes)

/-- Every legal move costs at least one unit of the measure. -/
theorem playMeasure_lt_of_follows {m n : ℕ} {L : Repetition m n}
    (hL : ExcludesRepeats L) {st st' : State m n} (h : Follows L st' st) :
    playMeasure st' < playMeasure st := by
  obtain ⟨hend, mv, hmv, rfl⟩ := h
  have hpass : st.passes < 2 := passes_lt_two_of_not_ended hend
  have hle : st.seen.ncard ≤ Fintype.card (Situation m n) := ncard_seen_le_card st
  cases mv with
  | pass =>
      have hmono : st.seen.ncard ≤ (step st Move.pass).seen.ncard :=
        ncard_seen_le_of_step st Move.pass
      have hle' : (step st Move.pass).seen.ncard ≤ Fintype.card (Situation m n) :=
        ncard_seen_le_card _
      have hp : (step st Move.pass).passes = st.passes + 1 := passes_step_pass st
      unfold playMeasure
      omega
  | play p =>
      have hgrow : (step st (Move.play p)).seen.ncard = st.seen.ncard + 1 :=
        ncard_seen_lt_of_play st p (hL st p hmv)
      have hle' : (step st (Move.play p)).seen.ncard ≤ Fintype.card (Situation m n) :=
        ncard_seen_le_card _
      have hp : (step st (Move.play p)).passes = 0 := passes_step_play st p
      unfold playMeasure
      omega

/-! ## Termination -/

/-- Termination for any repetition rule that excludes repeats.

Stated separately from the two superko rules because it is what makes the
result robust to OPEN-1 (claim C-18): any variant rule that ends the game on
two passes and forbids a play from recreating a seen situation terminates by
this theorem, whatever it does about passes. -/
theorem wellFounded_follows {m n : ℕ} {L : Repetition m n} (hL : ExcludesRepeats L) :
    WellFounded (Follows L) :=
  Subrelation.wf (fun h => playMeasure_lt_of_follows hL h)
    (InvImage.wf playMeasure Nat.lt_wfRel.wf)

/-- **C-13.** Play under situational superko terminates. -/
theorem C13_terminates {m n : ℕ} : WellFounded (Follows (SSK (m := m) (n := n))) :=
  wellFounded_follows ssk_excludesRepeats

/-- The same under positional superko. -/
theorem C13_terminates_psk {m n : ℕ} : WellFounded (Follows (PSK (m := m) (n := n))) :=
  wellFounded_follows psk_excludesRepeats

/-! ## How long a game can be -/

/-- Along a run of legal moves the measure pays for the moves made: each step
costs at least one, so the number of steps taken is bounded by the drop. -/
lemma playMeasure_add_le {m n : ℕ} {L : Repetition m n} (hL : ExcludesRepeats L)
    (f : ℕ → State m n) (k : ℕ) (hf : ∀ i < k, Follows L (f (i + 1)) (f i)) :
    ∀ i ≤ k, playMeasure (f i) + i ≤ playMeasure (f 0) := by
  intro i
  induction i with
  | zero => intro _; omega
  | succ j ih =>
      intro hj
      have h1 := ih (by omega)
      have h2 := playMeasure_lt_of_follows hL (hf j (by omega))
      omega

/-- No infinite sequence of legal moves exists. -/
theorem C13_no_infinite_play {m n : ℕ} (L : Repetition m n) (hL : ExcludesRepeats L)
    (f : ℕ → State m n) (hf : ∀ i, Follows L (f (i + 1)) (f i)) : False := by
  have h := playMeasure_add_le hL f (playMeasure (f 0) + 1) (fun i _ => hf i)
    (playMeasure (f 0) + 1) le_rfl
  omega

/-- At the root of play one situation has been seen and no pass has been made,
so the whole budget is twice the number of situations. -/
lemma playMeasure_start {m n : ℕ} (b : Position m n) (c : Color) :
    playMeasure (start b c) = 2 * Fintype.card (Situation m n) := by
  have h1 : (start b c).seen.ncard = 1 := Set.ncard_singleton _
  have h2 : (start b c).passes = 0 := rfl
  have h3 : 0 < Fintype.card (Situation m n) := Fintype.card_pos_iff.mpr ⟨⟨b, c⟩⟩
  unfold playMeasure
  omega

/-- **C-13, quantitative.** A game begun from a position as the root of play
lasts at most `2 * Fintype.card (Situation m n)` moves. -/
theorem C13_length_bound {m n : ℕ} (L : Repetition m n) (hL : ExcludesRepeats L)
    (b : Position m n) (c : Color) (f : ℕ → State m n) (k : ℕ)
    (h0 : f 0 = start b c) (hf : ∀ i < k, Follows L (f (i + 1)) (f i)) :
    k ≤ 2 * Fintype.card (Situation m n) := by
  have h := playMeasure_add_le hL f k hf k le_rfl
  rw [h0, playMeasure_start] at h
  omega

/-- **C-13, quantitative and explicit.** The same bound with the cardinality
computed: a game lasts at most `4 * 3 ^ (m * n)` moves.

This is the form the complexity question consumes. A game is singly exponential
in the board size, so a machine that plays one out needs exponentially many
moves and — if it carries the archive that superko requires — exponential
space. It says nothing about whether that space is necessary, which is the
open part (C-3, C-14). -/
theorem C13_length_bound_explicit {m n : ℕ} (L : Repetition m n)
    (hL : ExcludesRepeats L) (b : Position m n) (c : Color) (f : ℕ → State m n)
    (k : ℕ) (h0 : f 0 = start b c) (hf : ∀ i < k, Follows L (f (i + 1)) (f i)) :
    k ≤ 4 * 3 ^ (m * n) := by
  have h := C13_length_bound L hL b c f k h0 hf
  rw [card_situation] at h
  omega

end Superko
