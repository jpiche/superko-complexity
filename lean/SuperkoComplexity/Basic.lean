/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Defs
import Mathlib.Combinatorics.SimpleGraph.Connectivity.Connected
import Mathlib.Combinatorics.SimpleGraph.Connectivity.Finite

/-!
# Derived notions and decidable counterparts

Everything that is *not* needed to state the main theorem. The kernel checks
this file, so a reader auditing the project need not read it — which is the
whole reason `Defs.lean` is kept small.

## What belongs here

* Lemmas about the core definitions.
* **Decidable counterparts.** `Defs.lean` is classical and noncomputable. Each
  decidable version defined here is accompanied by a lemma proving it equal to
  the core definition. Certificate checkers use the decidable version; the
  bridging lemma is what makes such a check say something about the definitions
  a reader audited. A checker not routed through a bridging lemma proves
  nothing about Go.

## Status

Compiles. Two groups of material, both exercised by the kernel: the decidable
counterparts of the core definitions with their bridging lemmas, and — added
for claim C-13 — the finiteness facts about `Situation` together with the
one-line consequences of `step` that a termination argument needs.

C-13 itself is proved in `Results/C13_Termination.lean`. Its measure is not the
lexicographic one on (unvisited situations, pass counter) that
`docs/plans/first-results-plan.md` proposed. Weighting the situation count by
two flattens the order to a single natural number, because two units of that
budget are exactly enough to absorb the reset of `passes` that a play performs.
One `omega` call per case then discharges the whole argument.
-/

namespace Superko

/-! ## Decidability

`Defs.lean` is classical, so nothing in it evaluates. Instance resolution
cannot even see into `Adj`, which is decidable on its face, because a `def` is
semireducible. The instances here restore evaluation for the parts that are
genuinely decidable, and are the first half of the bridge that lets a
certificate check say something about the core. -/

instance decAdj {m n : ℕ} (p q : Point m n) : Decidable (Adj p q) := by
  unfold Adj; infer_instance

instance decJoined {m n : ℕ} (b : Position m n) (p q : Point m n) :
    Decidable (Joined b p q) := by
  unfold Joined; infer_instance

lemma adj_symm {m n : ℕ} {p q : Point m n} (h : Adj p q) : Adj q p := by
  unfold Adj at h ⊢; aesop

lemma adj_irrefl {m n : ℕ} (p : Point m n) : ¬ Adj p p := by
  unfold Adj; omega

/-! ### Chains as graph components

`chain` is a transitive closure, which Mathlib gives no decidability for. But
`Joined` is symmetric and irreflexive, so it is the adjacency of a simple
graph, and Mathlib decides reachability in a finite simple graph. -/

/-- The graph whose connected components are the chains of `b`. -/
def stoneGraph {m n : ℕ} (b : Position m n) : SimpleGraph (Point m n) where
  Adj := Joined b
  symm := ⟨by
    intro p q h
    obtain ⟨hadj, hcol, hsome⟩ := h
    exact ⟨adj_symm hadj, hcol.symm, by rw [← hcol]; exact hsome⟩⟩
  loopless := ⟨by
    intro p h
    obtain ⟨hadj, -, -⟩ := h
    exact adj_irrefl p hadj⟩

instance {m n : ℕ} (b : Position m n) : DecidableRel (stoneGraph b).Adj :=
  fun p q => decJoined b p q

/-- **Bridge.** Membership of a chain is reachability in `stoneGraph`. -/
lemma mem_chain_iff_reachable {m n : ℕ} (b : Position m n) (p q : Point m n) :
    q ∈ chain b p ↔ (stoneGraph b).Reachable p q := by
  rw [SimpleGraph.reachable_iff_reflTransGen]
  exact Iff.rfl

instance decMemChain {m n : ℕ} (b : Position m n) (p q : Point m n) :
    Decidable (q ∈ chain b p) :=
  decidable_of_iff _ (mem_chain_iff_reachable b p q).symm

instance decHasLiberty {m n : ℕ} (b : Position m n) (p : Point m n) :
    Decidable (HasLiberty b p) := by
  unfold HasLiberty; infer_instance

/-! ### Reaching, as a graph component

`Reaches` uses a relation that is *not* symmetric — a step requires its source
empty, while the final point is a stone — so it is not a graph adjacency as it
stands. But the intermediate points of such a walk are all empty, being sources
of later steps, so the walk splits into a path through empty points followed by
one step onto a stone. That prefix *is* symmetric. -/

/-- Adjacency between empty points. -/
def emptyAdj {m n : ℕ} (b : Position m n) (p q : Point m n) : Prop :=
  Adj p q ∧ b p = none ∧ b q = none

instance decEmptyAdj {m n : ℕ} (b : Position m n) (p q : Point m n) :
    Decidable (emptyAdj b p q) := by unfold emptyAdj; infer_instance

/-- The graph on empty points. -/
def emptyGraph {m n : ℕ} (b : Position m n) : SimpleGraph (Point m n) where
  Adj := emptyAdj b
  symm := ⟨by
    intro p q h
    obtain ⟨hadj, hp, hq⟩ := h
    exact ⟨adj_symm hadj, hq, hp⟩⟩
  loopless := ⟨by
    intro p h
    exact adj_irrefl p h.1⟩

instance {m n : ℕ} (b : Position m n) : DecidableRel (emptyGraph b).Adj :=
  fun p q => decEmptyAdj b p q

/-- The step relation of `Reaches`: adjacency out of an empty point. -/
private def stepR {m n : ℕ} (b : Position m n) (x y : Point m n) : Prop :=
  Adj x y ∧ b x = none

/-- An `emptyAdj` walk from an empty point stays on empty points. -/
private lemma empty_of_reflTransGen_emptyAdj {m n : ℕ} (b : Position m n)
    {p x : Point m n} (h : Relation.ReflTransGen (emptyAdj b) p x) (hp : b p = none) :
    b x = none := by
  induction h with
  | refl => exact hp
  | tail _ hyx _ => exact hyx.2.2

/-- A `stepR` walk ending on an empty point is an `emptyAdj` walk. -/
private lemma emptyAdj_of_stepR {m n : ℕ} (b : Position m n) {p x : Point m n}
    (h : Relation.ReflTransGen (stepR b) p x) (hx : b x = none) :
    Relation.ReflTransGen (emptyAdj b) p x := by
  revert hx
  induction h with
  | refl => intro _; exact Relation.ReflTransGen.refl
  | tail hpy hyx ih =>
      intro hxn
      exact (ih hyx.2).tail ⟨hyx.1, hyx.2, hxn⟩

/-- Conversely, every `emptyAdj` walk is a `stepR` walk. -/
private lemma stepR_of_emptyAdj {m n : ℕ} (b : Position m n) {p x : Point m n}
    (h : Relation.ReflTransGen (emptyAdj b) p x) :
    Relation.ReflTransGen (stepR b) p x := by
  induction h with
  | refl => exact Relation.ReflTransGen.refl
  | tail _ hyx ih => exact ih.tail ⟨hyx.1, hyx.2.1⟩

/-- **Bridge.** From an empty point, reaching a color is: walk to some empty
point through empty points, then step onto a stone of that color. -/
lemma reaches_iff {m n : ℕ} (b : Position m n) (p : Point m n) (c : Color)
    (hp : b p = none) :
    Reaches b p c ↔
      ∃ x, (emptyGraph b).Reachable p x ∧ ∃ q, Adj x q ∧ b q = some c := by
  constructor
  · rintro ⟨q, hq, hwalk⟩
    rcases Relation.ReflTransGen.cases_tail hwalk with heq | ⟨x, hpx, hxq⟩
    · exact absurd (heq ▸ hq) (by rw [hp]; simp)
    · refine ⟨x, ?_, q, hxq.1, hq⟩
      rw [SimpleGraph.reachable_iff_reflTransGen]
      exact emptyAdj_of_stepR b hpx hxq.2
  · rintro ⟨x, hreach, q, hxq, hq⟩
    rw [SimpleGraph.reachable_iff_reflTransGen] at hreach
    refine ⟨q, hq, (stepR_of_emptyAdj b hreach).tail ⟨hxq, ?_⟩⟩
    exact empty_of_reflTransGen_emptyAdj b hreach hp

/-- No `stepR` walk leaves an occupied point: a step requires its source
empty. -/
private lemma eq_of_stepR_of_occupied {m n : ℕ} (b : Position m n)
    {p x : Point m n} (h : Relation.ReflTransGen (stepR b) p x) (hp : b p ≠ none) :
    x = p := by
  induction h with
  | refl => rfl
  | tail _ hyx ih => exact absurd (ih ▸ hyx.2) hp

/-- **Bridge.** From an occupied point, `Reaches` degenerates to the point's
own color — no walk can leave an occupied point. -/
lemma reaches_iff_occupied {m n : ℕ} (b : Position m n) (p : Point m n)
    (c : Color) (hp : b p ≠ none) : Reaches b p c ↔ b p = some c := by
  constructor
  · rintro ⟨q, hq, hwalk⟩
    rw [eq_of_stepR_of_occupied b hwalk hp] at hq
    exact hq
  · intro h
    exact ⟨p, h, Relation.ReflTransGen.refl⟩

instance decReaches {m n : ℕ} (b : Position m n) (p : Point m n) (c : Color) :
    Decidable (Reaches b p c) :=
  if hp : b p = none then decidable_of_iff _ (reaches_iff b p c hp).symm
  else decidable_of_iff _ (reaches_iff_occupied b p c hp).symm

/-! ### Move application, computably

`clear` and `resolve` were elaborated under `open scoped Classical`, so their
`if` carries the classical instance and they do not reduce. These counterparts
differ from them only in that instance. -/

/-- The computable counterpart of `clear`. -/
def clear' {m n : ℕ} (b : Position m n) (c : Color) : Position m n :=
  fun q => if b q = some c ∧ ¬ HasLiberty b q then none else b q

/-- **Bridge.** `clear'` computes `clear`. -/
lemma clear'_eq_clear {m n : ℕ} (b : Position m n) (c : Color) :
    clear' b c = clear b c := by
  funext q; unfold clear' clear; congr 1

/-- The computable counterpart of `resolve`. -/
def resolve' {m n : ℕ} (b : Position m n) (c : Color) (p : Point m n) :
    Position m n :=
  clear' (Function.update b p (some c)) c.other

/-- **Bridge.** `resolve'` computes `resolve`. -/
lemma resolve'_eq_resolve {m n : ℕ} (b : Position m n) (c : Color)
    (p : Point m n) : resolve' b c p = resolve b c p := by
  unfold resolve' resolve; exact clear'_eq_clear _ _

/-- The played stone survives its own move. `resolve` clears `c.other` only, so
whatever else a play removes, the point played to carries a `c` stone after it.

This is the whole reason the empty position has no incoming play edge, which is
what `Compress.sitStep_empty_board` records and what makes the situation graph's
condensation nontrivial (C-45). -/
lemma resolve_self {m n : ℕ} (b : Position m n) (c : Color) (p : Point m n) :
    resolve b c p p = some c := by
  have hne : c ≠ c.other := by cases c <;> simp [Color.other]
  unfold resolve clear
  simp [hne]

/-- The computable counterpart of `PlayableAt`. -/
def PlayableAt' {m n : ℕ} (b : Position m n) (c : Color) (p : Point m n) : Prop :=
  b p = none ∧ HasLiberty (resolve' b c p) p

instance {m n : ℕ} (b : Position m n) (c : Color) (p : Point m n) :
    Decidable (PlayableAt' b c p) := by unfold PlayableAt'; infer_instance

/-- **Bridge.** `PlayableAt'` decides `PlayableAt`. -/
lemma playableAt'_iff {m n : ℕ} (st : State m n) (p : Point m n) :
    PlayableAt' st.now.board st.now.toMove p ↔ PlayableAt st p := by
  unfold PlayableAt' PlayableAt
  rw [resolve'_eq_resolve]

/-! ### Area, computably -/

/-- The computable counterpart of `area`. -/
def area' {m n : ℕ} (b : Position m n) (c : Color) : ℕ :=
  (Finset.univ.filter (fun p : Point m n =>
    b p = some c ∨ (b p = none ∧ Reaches b p c ∧ ¬ Reaches b p c.other))).card

/-- **Bridge.** `area'` computes `area`. -/
lemma area'_eq_area {m n : ℕ} (b : Position m n) (c : Color) :
    area' b c = area b c := by
  unfold area' area
  rw [Set.ncard_eq_toFinset_card']
  congr 1
  ext p
  simp

/-! ## Finiteness

The board is finite, so the situations are, and every history is a subset of a
finite set. That is what lets a termination measure be a natural number which
can run out.

`Color` gets its `Fintype` instance here rather than through a `deriving`
clause in `Defs.lean`. An instance is a derived notion, and the trusted core is
not enlarged for one. -/

instance : Fintype Color :=
  ⟨{Color.black, Color.white}, fun c => by cases c <;> decide⟩

lemma card_color : Fintype.card Color = 2 := rfl

/-- A situation is nothing but its board and its player to move. -/
def situationEquiv (m n : ℕ) : Situation m n ≃ Position m n × Color where
  toFun s := (s.board, s.toMove)
  invFun x := ⟨x.1, x.2⟩
  left_inv _ := rfl
  right_inv _ := rfl

instance instFintypeSituation (m n : ℕ) : Fintype (Situation m n) :=
  Fintype.ofEquiv (Position m n × Color) (situationEquiv m n).symm

/-- Each of the `m * n` points carries a black stone, a white stone or
nothing. -/
lemma card_position (m n : ℕ) : Fintype.card (Position m n) = 3 ^ (m * n) := by
  simp [Position, Point, Fintype.card_option, card_color]

/-- **The count every exponential in this project comes from.** A situation is a
position together with a player to move, so there are `2 * 3 ^ (m * n)` of
them — singly exponential in the board size. -/
lemma card_situation (m n : ℕ) :
    Fintype.card (Situation m n) = 2 * 3 ^ (m * n) := by
  rw [Fintype.card_congr (situationEquiv m n), Fintype.card_prod, card_position,
    card_color]
  omega

-- Two counts a reader can check by hand: the 1×1 board has three positions and
-- two players to move, the 2×2 board eighty-one and two.
example : Fintype.card (Situation 1 1) = 6 := by rw [card_situation]; decide
example : Fintype.card (Situation 2 2) = 162 := by rw [card_situation]; decide

/-- A history never holds more situations than there are situations. -/
lemma ncard_seen_le_card {m n : ℕ} (st : State m n) :
    st.seen.ncard ≤ Fintype.card (Situation m n) := by
  have h : st.seen.ncard ≤ (Set.univ : Set (Situation m n)).ncard :=
    Set.ncard_le_ncard (Set.subset_univ _) (Set.toFinite _)
  simpa [Set.ncard_univ, Nat.card_eq_fintype_card] using h

/-! ## How a move moves the state

Each of these is a one-line consequence of what `step` does, named so that a
change to `Defs.lean` breaks a small site rather than the interior of a
theorem. `start` is the encoding question C-1, so it is the field most likely
to change. -/

/-- A live game has a pass still to spare. -/
lemma passes_lt_two_of_not_ended {m n : ℕ} {st : State m n} (h : ¬ Ended st) :
    st.passes < 2 :=
  Nat.lt_of_not_le h

/-- No move ever forgets a situation. -/
lemma seen_subset_of_step {m n : ℕ} (st : State m n) (mv : Move m n) :
    st.seen ⊆ (step st mv).seen :=
  Set.subset_insert _ _

/-- Hence no move shrinks the history. -/
lemma ncard_seen_le_of_step {m n : ℕ} (st : State m n) (mv : Move m n) :
    st.seen.ncard ≤ (step st mv).seen.ncard :=
  Set.ncard_le_ncard (seen_subset_of_step st mv) (Set.toFinite _)

/-- A play whose target the history does not hold enlarges it by exactly one. -/
lemma ncard_seen_lt_of_play {m n : ℕ} (st : State m n) (p : Point m n)
    (h : st.now.after (Move.play p) ∉ st.seen) :
    (step st (Move.play p)).seen.ncard = st.seen.ncard + 1 :=
  Set.ncard_insert_of_notMem h (Set.toFinite _)

/-- A play resets the pass counter. -/
lemma passes_step_play {m n : ℕ} (st : State m n) (p : Point m n) :
    (step st (Move.play p)).passes = 0 := rfl

/-- A pass advances the pass counter. -/
lemma passes_step_pass {m n : ℕ} (st : State m n) :
    (step st Move.pass).passes = st.passes + 1 := rfl

/-! ## The play relation

What a repetition rule must do for play to terminate, and the relation whose
well-foundedness says that it does. -/

/-- A repetition rule **excludes repeats** when no legal play may recreate a
situation the history already holds.

Both superko rules have this property, and it is the only property of them a
termination argument uses. In particular the pass exemption — OPEN-1 in
`docs/formal-model.md`, claim C-18 — does not bear on termination: what bounds a
run of passes is the pass counter, not the history. -/
def ExcludesRepeats {m n : ℕ} (L : Repetition m n) : Prop :=
  ∀ (st : State m n) (p : Point m n), L st (Move.play p) →
    st.now.after (Move.play p) ∉ st.seen

theorem ssk_excludesRepeats {m n : ℕ} : ExcludesRepeats (SSK (m := m) (n := n)) :=
  fun _ _ h => h.2

theorem psk_excludesRepeats {m n : ℕ} : ExcludesRepeats (PSK (m := m) (n := n)) := by
  intro st p h hmem
  exact h.2 _ hmem rfl

/-- `Follows L st' st`: the game is live at `st`, and some `L`-legal move leads
from `st` to `st'`.

The successor comes first because that is the order `WellFounded` reads —
`Acc r x` looks at every `y` with `r y x` — so `WellFounded (Follows L)` says
no play goes on forever, rather than the reverse. -/
def Follows {m n : ℕ} (L : Repetition m n) (st' st : State m n) : Prop :=
  ¬ Ended st ∧ ∃ mv, L st mv ∧ st' = step st mv

end Superko
