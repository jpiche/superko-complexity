/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Results.C29_DeciderCorrect

/-!
# What the value sees of the archive

Derived notions, not part of the trusted core. Nothing here appears in the
statement of `BlackWins`, and `Defs.lean` is untouched.

The question these serve is the upper bound. The archive an EXPSPACE decider
carries holds up to `2 · 3 ^ (m * n)` situations (C-3, `folklore`), and the
standing intuition is that most of it cannot matter to the value. This file
fixes what "cannot matter" means and proves the first instance of it.

## What is here

`winsFor_transfer` is the general statement: a relation that preserves the
situation, the pass counter and legality, and survives `step`, preserves the
game value. It is parametric in the repetition rule, and every later
compression statement factors through it (C-41).

`Cone` is the forward reachable set in the situation graph — the moves the
board permits, with the repetition rule out of the way. `winsFor_cone_congr`
and its corollaries say the value reads the archive only inside that cone
(C-42): entries outside it may be dropped, and unreachable situations may be
added, without changing who wins.

`decideWins_iff_winsFor` lifts C-29 from `start'` to any state whose archive is
nonempty, which is every state a root of play reaches (C-43). C-29 is stated at
the root only, so without this no mid-game verdict transports to `WinsFor` at
all.

`sitStep_empty_board` is the structural fact that makes the cone worth
measuring and then explains why it does not pay: a play never produces the
empty board, because `resolve` clears the opponent's stones only
(`resolve_self`), so the empty situation has no incoming play edge (C-44).

## What this does not establish

**The cone is not a compression.** C-45 records the census: on every board it
reaches — the nine of `superko-graph`'s test suite, and 3×4, 2×6 and 1×12 at
the top of the range — the situation graph has exactly two strongly connected
components — the empty board's pass cycle, and everything else — so the cone of
any non-empty situation is the whole of that giant component, and the prune of
`winsFor_seen_inter_cone` removes at most the two empty situations (C-46,
`computed`). The theorems below are exact and their effect is nil. That is the
result: the subset-of-the-archive route to a smaller state is refuted by
measurement, and the refutation is recorded rather than the hope.

Nor does anything here bear on C-14. The decision problem takes a root position
alone (C-1, encoding (C)), so no algorithm is obliged to summarize an interior
history; and a bound on one summary scheme bounds no complexity class.
-/

open scoped Classical

namespace Superko
namespace Compress

variable {m n : ℕ}

/-! ## 0. Two one-liners the transfer lemma needs -/

lemma ended_congr {s t : State m n} (h : s.passes = t.passes) :
    Ended s ↔ Ended t := by unfold Ended; rw [h]

/-! ## 1. Value transfer along a simulation -/

/-- **Lemma 1.** If a relation `R` preserves the situation, the pass counter and
legality, and is preserved by `step` on legal moves, then it preserves the game
value. Every compression statement below is an instance. -/
theorem winsFor_transfer {L : Repetition m n} {komi : ℚ} {c : Color}
    (R : State m n → State m n → Prop)
    (hnow : ∀ {s t : State m n}, R s t → s.now = t.now)
    (hpass : ∀ {s t : State m n}, R s t → s.passes = t.passes)
    (hleg : ∀ {s t : State m n}, R s t → ∀ mv, L s mv ↔ L t mv)
    (hstep : ∀ {s t : State m n}, R s t → ∀ mv, L s mv → R (step s mv) (step t mv))
    {s : State m n} (hw : WinsFor m n L komi c s) :
    ∀ t : State m n, R s t → WinsFor m n L komi c t := by
  induction hw with
  | @ended st hend hwin =>
      intro t hR
      exact WinsFor.ended ((ended_congr (hpass hR)).mp hend)
        (by rw [← hnow hR]; exact hwin)
  | @mover st mv hend hto hmv _ ih =>
      intro t hR
      exact WinsFor.mover
        (fun hc => hend ((ended_congr (hpass hR)).mpr hc))
        (by rw [← hnow hR]; exact hto)
        ((hleg hR mv).mp hmv)
        (ih _ (hstep hR mv hmv))
  | @waiter st hend hto _ ih =>
      intro t hR
      refine WinsFor.waiter
        (fun hc => hend ((ended_congr (hpass hR)).mpr hc))
        (by rw [← hnow hR]; exact hto) (fun mv hmv => ?_)
      exact ih mv ((hleg hR mv).mpr hmv) _ (hstep hR mv ((hleg hR mv).mpr hmv))

/-! ## 2. The situation graph and the forward cone -/

/-- One board-legal move in situation space, superko ignored. The pass edge is
unconditional, because a pass is legal in every state under both rules. -/
def SitStep (s t : Situation m n) : Prop :=
  t = s.after .pass ∨ ∃ p : Point m n, PlayableAt' s.board s.toMove p ∧ t = s.after (.play p)

/-- Every situation still reachable from `s` by board-legal moves. -/
def Cone (s : Situation m n) : Set (Situation m n) :=
  {t | Relation.ReflTransGen SitStep s t}

lemma mem_cone_self (s : Situation m n) : s ∈ Cone s := Relation.ReflTransGen.refl

lemma cone_subset_of_sitStep {s t : Situation m n} (h : SitStep s t) :
    Cone t ⊆ Cone s := fun _ hx => Relation.ReflTransGen.head h hx

/-- A move legal under SSK is a board-legal move of the situation graph. -/
lemma sitStep_of_ssk {st : State m n} {mv : Move m n} (h : SSK st mv) :
    SitStep st.now (st.now.after mv) := by
  cases mv with
  | pass => exact Or.inl rfl
  | play p => exact Or.inr ⟨p, (playableAt'_iff st p).mpr h.1, rfl⟩

/-- The target of an SSK-legal play lies in the cone. -/
lemma mem_cone_of_ssk {st : State m n} {mv : Move m n} (h : SSK st mv) :
    st.now.after mv ∈ Cone st.now :=
  Relation.ReflTransGen.single (sitStep_of_ssk h)

/-! ## 3. The value sees the archive only through the forward cone -/

/-- **Lemma 2.** Two states with the same situation and pass counter whose
archives agree on the forward cone have the same value. -/
theorem winsFor_cone_congr {komi : ℚ} {c : Color} (s t : State m n)
    (hnow : s.now = t.now) (hpass : s.passes = t.passes)
    (hagree : ∀ x ∈ Cone s.now, (x ∈ s.seen ↔ x ∈ t.seen)) :
    WinsFor m n SSK komi c s ↔ WinsFor m n SSK komi c t := by
  -- the relation, symmetric in shape so one construction serves both directions
  let R : State m n → State m n → Prop := fun a b =>
    a.now = b.now ∧ a.passes = b.passes ∧ ∀ x ∈ Cone a.now, (x ∈ a.seen ↔ x ∈ b.seen)
  have hlegR : ∀ {a b : State m n}, R a b → ∀ mv, SSK a mv ↔ SSK b mv := by
    rintro a b ⟨hn, _, hag⟩ mv
    cases mv with
    | pass => exact Iff.rfl
    | play p =>
        have hpl : PlayableAt a p ↔ PlayableAt b p := by
          rw [← playableAt'_iff a p, ← playableAt'_iff b p, hn]
        constructor
        · rintro ⟨h1, h2⟩
          refine ⟨hpl.mp h1, fun hm => h2 ?_⟩
          have hx : a.now.after (.play p) ∈ Cone a.now :=
            mem_cone_of_ssk (st := a) (mv := .play p) ⟨h1, h2⟩
          exact (hag _ hx).mpr (by rw [hn]; exact hm)
        · rintro ⟨h1, h2⟩
          have h1' : PlayableAt a p := hpl.mpr h1
          refine ⟨h1', fun hm => h2 ?_⟩
          have hx : a.now.after (.play p) ∈ Cone a.now :=
            Relation.ReflTransGen.single
              (Or.inr ⟨p, (playableAt'_iff a p).mpr h1', rfl⟩)
          rw [← hn]
          exact (hag _ hx).mp hm
  have hstepR : ∀ {a b : State m n}, R a b → ∀ mv, SSK a mv → R (step a mv) (step b mv) := by
    rintro a b ⟨hn, hp, hag⟩ mv hmv
    refine ⟨by simp [step, hn], by simp [step, hp], ?_⟩
    intro x hx
    have hsub : Cone (step a mv).now ⊆ Cone a.now := by
      have : SitStep a.now (a.now.after mv) := sitStep_of_ssk hmv
      simpa [step] using cone_subset_of_sitStep this
    have hxa : x ∈ Cone a.now := hsub hx
    simp only [step, Set.mem_insert_iff, hn]
    exact or_congr Iff.rfl (hag x hxa)
  constructor
  · exact fun h => winsFor_transfer R (fun h => h.1) (fun h => h.2.1) hlegR hstepR h t
      ⟨hnow, hpass, hagree⟩
  · refine fun h => winsFor_transfer R (fun h => h.1) (fun h => h.2.1) hlegR hstepR h s
      ⟨hnow.symm, hpass.symm, ?_⟩
    intro x hx
    rw [← hnow] at hx
    exact (hagree x hx).symm

/-- **Corollary.** Pruning the archive to the forward cone changes no value. -/
theorem winsFor_seen_inter_cone {komi : ℚ} {c : Color} (st : State m n) :
    WinsFor m n SSK komi c st ↔
      WinsFor m n SSK komi c ⟨st.now, st.seen ∩ Cone st.now, st.passes⟩ :=
  winsFor_cone_congr _ _ rfl rfl (fun x hx => by simp [hx])

/-- **Corollary.** Adding an unreachable situation to the archive changes no
value. -/
theorem winsFor_insert_unreachable {komi : ℚ} {c : Color} (st : State m n)
    (u : Situation m n) (hu : u ∉ Cone st.now) :
    WinsFor m n SSK komi c st ↔
      WinsFor m n SSK komi c ⟨st.now, insert u st.seen, st.passes⟩ :=
  winsFor_cone_congr _ _ rfl rfl (by
    intro x hx
    simp only [Set.mem_insert_iff]
    exact ⟨fun h => Or.inr h, fun h => h.elim (fun he => absurd (he ▸ hx) hu) id⟩)

/-! ## 4. The decider at an arbitrary archive

C-29 is stated at `start'`. Everything measured mid-game needs it at an
arbitrary state, which costs only the fuel arithmetic. -/

theorem playMeasure'_le (s : State' m n) (hs : s.seen.Nonempty) :
    playMeasure' s ≤ 4 * 3 ^ (m * n) := by
  have h1 : 1 ≤ s.seen.card := Finset.card_pos.mpr hs
  have h2 : s.seen.card ≤ Fintype.card (Situation m n) := s.seen.card_le_univ
  have h3 := card_situation (m := m) (n := n)
  unfold playMeasure'
  omega

/-- **C-29 at an arbitrary archive.** The archive decider, run at the same fuel,
decides `WinsFor` from any state whose archive is nonempty — which is every
state reachable from a root of play. -/
theorem decideWins_iff_winsFor {L' : State' m n → Move m n → Bool} {L : Repetition m n}
    (hL : Faithful L' L) (hLr : ExcludesRepeats L)
    (s : State' m n) (hs : s.seen.Nonempty) (komi : ℚ) (c : Color) :
    decideWins ⌊komi⌋ c L' (4 * 3 ^ (m * n) + 1) s = true ↔
      WinsFor m n L komi c s.toState := by
  constructor
  · exact decideWins_sound hL _ _
  · intro h
    have hc := decideWins_complete_aux hL hLr s.toState h s rfl
    exact decideWins_mono _ _ (by have := playMeasure'_le s hs; omega) _ hc

/-! ## 5. The empty board has no incoming play edge

Why the condensation of the situation graph is not trivial, and — with the
census of C-45 — why it is not useful either. -/

/-- **A play never produces the empty board.** `resolve` clears `c.other` only,
so the point played to carries a stone afterwards (`resolve_self`). Hence the
only `SitStep` into an empty board comes from an empty board, by a pass: the
empty situation has in-degree zero under play edges, on every board. -/
theorem sitStep_empty_board {s t : Situation m n} (h : SitStep s t)
    (ht : t.board = fun _ => none) : s.board = fun _ => none := by
  rcases h with rfl | ⟨p, _, rfl⟩
  · simpa [Situation.after] using ht
  · have := congrFun ht p
    simp only [Situation.after] at this
    rw [resolve_self] at this
    exact absurd this (by simp)

end Compress
end Superko
