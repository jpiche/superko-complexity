/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Results.C13_Termination

/-!
# C-28 — exactly one color wins

From any state of a superko game, one of the two colors has a winning strategy,
and not both. In Go terms: there is no position-plus-history from which play
can be steered away from a decision. Whoever is to move either has a move into
a state they win, or every move they have hands the opponent a state the
opponent wins — and the recursion bottoms out because the game ends.

The two halves have different costs.

**Existence needs a rule that excludes repeats.** `C28_determined` recurses on
`wellFounded_follows`, the well-founded relation C-13 supplies: a legal move
strictly drops `playMeasure`, so the case split "the mover has a winning move,
or every move loses for them" is a legitimate induction. Without such a rule
play can cycle and the recursion has nothing to stand on. The hypothesis is
`ExcludesRepeats L` rather than SSK or PSK, so the theorem survives whichever
way OPEN-1 falls (C-18), exactly as C-13 does.

**Exclusivity needs no hypothesis at all.** `C28_not_both` is an induction on
the Black derivation, not on the game: at every state the three `WinsFor`
constructors disagree about something decidable — whether the game has ended,
or whose turn it is — except in the case where both colors wait, and there the
two `toMove` disequalities are contradictory on a two-element type. Nothing
about repetition, termination, or the board enters. This matters for hardness
reductions: the `←` direction of a correctness statement
`X x ↔ BlackWins (map x)` argued through a White strategy needs only this
lemma, and so carries no rules hypothesis with it.

Together these discharge the promise `Defs.lean` makes in the `WinsFor`
docstring — "that exactly one color wins from each state is determinacy — a
theorem, not part of this definition, and it rests on termination (C-13)" — and
the "Not determinacy" caveat in `C13_Termination.lean`, which said C-13 gives
the relation a determinacy proof would recurse on without being that proof.
This file is that proof.

## What this file does not establish

* Nothing about Go beyond what `Defs.lean` says Go is. `Defs.lean` records its
  own definitions as plausible rather than validated, and the independent
  agreement argument of `docs/trusted-base.md` has not been run. Determinacy
  here is determinacy of the formalized game.
* No strategy is produced. `C28_determined` is a classical disjunction about
  the existence of a `WinsFor` derivation; it says nothing about computing the
  winner, which is C-29's business.
* Nothing about which color wins, from any particular position or in general.
-/

namespace Superko

variable {m n : ℕ} {L : Repetition m n} {komi : ℚ}

/-! ## Existence -/

/-- **C-28, existence.** From every state one of the two colors has a winning
strategy, under any repetition rule that excludes repeats.

The induction is on C-13's well-founded `Follows` relation: at a state that has
not ended, either the player to move has a legal move into a state they win, or
every legal move leads to a state the waiting player wins by the inductive
hypothesis. -/
theorem C28_determined (hL : ExcludesRepeats L) (st : State m n) :
    WinsFor m n L komi .black st ∨ WinsFor m n L komi .white st := by
  induction st using (wellFounded_follows hL).induction with
  | _ st ih =>
    by_cases hE : Ended st
    · rcases hw : winner st.now.board komi with _ | _
      · exact .inl (.ended hE hw)
      · exact .inr (.ended hE hw)
    · rcases hd : st.now.toMove with _ | _
      · by_cases h : ∃ mv, L st mv ∧ WinsFor m n L komi .black (step st mv)
        · obtain ⟨mv, hmv, hwin⟩ := h
          exact .inl (.mover hE hd hmv hwin)
        · push Not at h
          refine .inr (.waiter hE (by rw [hd]; decide) fun mv hmv => ?_)
          exact (ih _ ⟨hE, mv, hmv, rfl⟩).resolve_left (h mv hmv)
      · by_cases h : ∃ mv, L st mv ∧ WinsFor m n L komi .white (step st mv)
        · obtain ⟨mv, hmv, hwin⟩ := h
          exact .inr (.mover hE hd hmv hwin)
        · push Not at h
          refine .inl (.waiter hE (by rw [hd]; decide) fun mv hmv => ?_)
          exact (ih _ ⟨hE, mv, hmv, rfl⟩).resolve_right (h mv hmv)

/-- **C-28** under situational superko. -/
theorem C28_determined_ssk (st : State m n) :
    WinsFor m n SSK komi .black st ∨ WinsFor m n SSK komi .white st :=
  C28_determined ssk_excludesRepeats st

/-- **C-28** under positional superko. -/
theorem C28_determined_psk (st : State m n) :
    WinsFor m n PSK komi .black st ∨ WinsFor m n PSK komi .white st :=
  C28_determined psk_excludesRepeats st

/-! ## Exclusivity -/

/-- **C-28, exclusivity.** The two colors cannot both have a winning strategy
from one state.

No hypothesis on `L`: this is an induction on the Black derivation rather than
on the game, and it holds for any repetition rule, including ones under which
play never terminates. -/
theorem C28_not_both {st : State m n} (hb : WinsFor m n L komi .black st) :
    WinsFor m n L komi .white st → False := by
  induction hb with
  | @ended st hend hwin =>
    intro hw
    cases hw with
    | ended _ hwin' => exact absurd (hwin.symm.trans hwin') (by decide)
    | mover hne _ _ _ => exact hne hend
    | waiter hne _ _ => exact hne hend
  | @mover st mv hne hto hleg _ ih =>
    intro hw
    cases hw with
    | ended hend _ => exact hne hend
    | mover _ hto' _ _ => exact absurd (hto.symm.trans hto') (by decide)
    | waiter _ _ hall => exact ih (hall _ hleg)
  | @waiter st hne hto _ ih =>
    intro hw
    cases hw with
    | ended hend _ => exact hne hend
    | mover _ hto' hleg' hw' => exact ih _ hleg' hw'
    | waiter _ hto' _ =>
      cases hc : st.now.toMove with
      | black => exact hto hc
      | white => exact hto' hc

/-! ## Determinacy -/

/-- **C-28.** Exactly one color wins from each state, under any repetition rule
that excludes repeats. The existence half carries the hypothesis; the
exclusivity half does not. -/
theorem C28_exactly_one (hL : ExcludesRepeats L) (st : State m n) :
    (WinsFor m n L komi .black st ∨ WinsFor m n L komi .white st) ∧
      ¬ (WinsFor m n L komi .black st ∧ WinsFor m n L komi .white st) :=
  ⟨C28_determined hL st, fun ⟨hb, hw⟩ => C28_not_both hb hw⟩

end Superko
