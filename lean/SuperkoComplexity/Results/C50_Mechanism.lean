/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Basic

/-!
# What separates the two superko rules (C-50, C-51)

`Superko.PSK` refuses a play whose board has stood with *either* color to move;
`Superko.SSK` refuses it only when that board has stood with *this* color to
move. So the two rules differ at exactly the plays whose target board has
occurred at the opposite parity and never at this one, and the question of how
a game reaches such a play is the question of how a board comes to stand with
both colors to move.

This file proves the two local facts that answer it.

**C-50: a pass closes a board to both rules.** A pass archives the situation it
leads to, and `now ∈ seen` holds already, so a pass made while the board is `X`
puts *both* of `X`'s situations into the history at once. From then on no play
anywhere in the game may recreate `X` — under either rule. A pass made at a
board therefore cannot be what lets *that* board recur at the opposite parity;
it is what forbids it. A pass made at some other board can still sit inside a
return walk and contribute to its parity, so what this rules out is a pass at
the recurring board, not passes altogether.

**C-51: a play never undoes a play of the same color.** `resolve` clears the
opponent's chains only, so the mover's own stones survive every move the mover
makes. A single play therefore never recreates the board it was played on, and
two plays by one color never do either. A board can only come back after a play
by each color — a ko brings one back in two — and a return that swaps the color
to move needs at least three plays (C-52).

Together these are the local content of claim C-52, which is that the return
walk behind a legality gap holds at least three plays and no pass at the
recurring board. That statement is
about walks and is proved by hand in `proofs/C-52.md`, not here; the two facts
below are the ones it turns on, and they are what makes it a correction rather
than a guess:

> `docs/formal-model.md` §OPEN-1 and the `SSK` docstring in `Defs.lean` used to
> say that the pass exemption is the parity resource separating the two rules,
> and that it is the mechanism behind "sending two, returning one". C-50 says a
> pass at the recurring board does the opposite, and the pass-free 1×3 line in
> `crates/superko-graph/tests/containment.rs` exhibits the separation with no
> pass at all. The separation does not depend on OPEN-1 or on C-18.

## Status

`proved`, machine-checked, on the three standard axioms. Nothing here bears on
whether the *value* of a position differs between the rules: that is C-17, and
it is open. A legality gap is necessary for a value gap and nowhere near
sufficient — the sweep of `superko-solve` finds no value gap on any board with
`m · n ≤ 5` and finds legality gaps all over those same boards (C-53).
-/

namespace Superko

variable {m n : ℕ}

/-! ## The history holds the present

`now ∈ seen` is the invariant C-50 needs, and it costs nothing: `start` seeds
the history with the root situation and `step` inserts the successor, so it
holds at the root and after every move, with no hypothesis on the move at all.
-/

/-- At the root of play the history holds the root situation — encoding (C) of
`docs/formal-model.md` §5, which is what `start` seeds. -/
theorem now_mem_seen_start (b : Position m n) (c : Color) :
    (start b c).now ∈ (start b c).seen :=
  rfl

/-- After any move the history holds the situation reached. -/
theorem now_mem_seen_step (st : State m n) (mv : Move m n) :
    (step st mv).now ∈ (step st mv).seen :=
  Set.mem_insert _ _

/-! ## A pass closes a board to both rules (C-50) -/

/-- A color is either the one named or the other one. -/
theorem eq_or_eq_other (x c : Color) : x = c ∨ x = c.other := by
  cases x <;> cases c <;> simp [Color.other]

/-- **A pass archives both of the board's situations.** The pass leads to the
board standing now with the turn handed over, and `now ∈ seen` gives the same
board with the turn as it is, so after the pass the history holds that board
under each color. -/
theorem pass_seen_both (st : State m n) (h : st.now ∈ st.seen) (x : Color) :
    (⟨st.now.board, x⟩ : Situation m n) ∈ (step st Move.pass).seen := by
  rcases eq_or_eq_other x st.now.toMove with rfl | rfl
  · exact Set.mem_insert_of_mem _ h
  · exact Set.mem_insert _ _

/-- **C-50.** Once a pass has been made while the board is `st.now.board`, no
later play may recreate that board — under situational superko or under
positional superko.

`hsub` is the history-monotonicity `Basic.seen_subset_of_step` supplies along
any sequence of moves: a later state's history contains the one the pass left.
-/
theorem C50_pass_closes_board (st st' : State m n) (h : st.now ∈ st.seen)
    (hsub : (step st Move.pass).seen ⊆ st'.seen) (p : Point m n)
    (hb : (st'.now.after (Move.play p)).board = st.now.board) :
    ¬ SSK st' (Move.play p) ∧ ¬ PSK st' (Move.play p) := by
  have hmem : st'.now.after (Move.play p) ∈ st'.seen := by
    have : st'.now.after (Move.play p) =
        (⟨st.now.board, (st'.now.after (Move.play p)).toMove⟩ : Situation m n) := by
      rw [← hb]
    rw [this]
    exact hsub (pass_seen_both st h _)
  refine ⟨fun hssk => hssk.2 hmem, fun hpsk => ?_⟩
  -- Positional superko asks for a board, and `st.now` is in the history with
  -- the right one.
  exact hpsk.2 st.now (hsub (seen_subset_of_step st Move.pass h)) hb.symm

/-! ## A play never undoes a play of the same color (C-51) -/

/-- **A play never removes the mover's own stones.** `resolve` clears
`c.other`, so a `c` stone standing before a `c` play stands after it. -/
theorem resolve_keeps_mover (b : Position m n) (c : Color) (q r : Point m n)
    (h : b r = some c) : resolve b c q r = some c := by
  have hne : c ≠ c.other := by cases c <;> simp [Color.other]
  have hupd : Function.update b q (some c) r = some c := by
    by_cases hr : r = q
    · subst hr; simp
    · rwa [Function.update_of_ne hr]
  unfold resolve clear
  simp [hupd, hne]

/-- **C-51, one play.** A play never recreates the board it was played on: the
point played to was empty and carries the played stone afterwards. -/
theorem C51_play_changes_board (b : Position m n) (c : Color) (p : Point m n)
    (h : b p = none) : resolve b c p ≠ b := by
  intro heq
  have hself : resolve b c p p = some c := resolve_self b c p
  rw [heq, h] at hself
  simp at hself

/-- **C-51, two plays by one color.** Two plays by the same color never
recreate the board either, whatever they capture: the first play's stone
survives the second, so the point it stands on is occupied at the end and was
empty at the start.

This is the case that rules out the three-move return `play, pass, play`, whose
two plays are necessarily by the same color — see `proofs/C-52.md`. -/
theorem C51_two_plays_by_one_color_change_board (b : Position m n) (c : Color)
    (p q : Point m n) (hp : b p = none) :
    resolve (resolve b c p) c q ≠ b := by
  intro heq
  have hfirst : resolve b c p p = some c := resolve_self b c p
  have hsecond : resolve (resolve b c p) c q p = some c :=
    resolve_keeps_mover _ c q p hfirst
  rw [heq, hp] at hsecond
  simp at hsecond

end Superko
