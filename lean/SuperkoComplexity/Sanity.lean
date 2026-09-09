/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Basic

/-!
# Sanity checks on the definitions

Concrete positions whose Go meaning is not in doubt, asserted against the
definitions in `Defs.lean`. Every check here is proved by `decide`, so the
kernel verifies it — these are not `#eval` observations to be read and trusted.

## What this is and is not

This is **not** the definitional validation the project relies on. That is
`docs/trusted-base.md`'s independent-agreement argument: reproducing counts
that other people computed, above all the 2×2 game count under positional
superko. These checks are hand-computed, so they can only catch a definition
that is wrong in a way the author already thought to test.

They are still worth having. Every one of them would have caught a real error
class — diagonal adjacency, chains merging across colors, capture resolved in
the wrong order, suicide misjudged when the move captures, dame miscounted —
and they run in the build rather than in a scratch file.

Each check routes through the computable counterparts in `Basic.lean` and their
bridging lemmas, which is what makes it a statement about `Defs.lean`.
-/

namespace Superko
namespace Sanity

/-! ## Fixtures -/

private def B : Option Color := some Color.black
private def W : Option Color := some Color.white
private def E : Option Color := none

private def mk22 (p00 p01 p10 p11 : Option Color) : Position 2 2 := fun p =>
  if p = (0,0) then p00 else if p = (0,1) then p01
  else if p = (1,0) then p10 else p11

private def mk33 (f : Fin 3 → Fin 3 → Option Color) : Position 3 3 :=
  fun p => f p.1 p.2

/-! ## Board geometry

Adjacency is orthogonal, irreflexive and symmetric. Diagonals are not
adjacency — the error that would silently make every chain too large. -/

theorem adj_orthogonal : Adj ((0,0) : Point 2 2) (0,1) := by decide

theorem adj_not_diagonal : ¬ Adj ((0,0) : Point 2 2) (1,1) := by decide

theorem adj_irreflexive : ¬ Adj ((0,0) : Point 2 2) (0,0) := by decide

theorem adj_symmetric : Adj ((0,1) : Point 2 2) (0,0) := by decide

/-- A 3×3 center point has four neighbors, a corner two. -/
theorem nbr_count_center :
    (Finset.univ.filter (fun q : Point 3 3 => Adj (1,1) q)).card = 4 := by decide

theorem nbr_count_corner :
    (Finset.univ.filter (fun q : Point 3 3 => Adj (0,0) q)).card = 2 := by decide

/-! ## Chains

A chain is same-colored and orthogonally connected. -/

/-- Adjacent stones of one color form one chain. -/
theorem chain_joins_adjacent :
    ((0,1) : Point 2 2) ∈ chain (mk22 B B E E) (0,0) := by decide

/-- Diagonal stones do not. -/
theorem chain_skips_diagonal :
    ((1,1) : Point 2 2) ∉ chain (mk22 B E E B) (0,0) := by decide

/-- Stones of different colors do not. -/
theorem chain_respects_color :
    ((0,1) : Point 2 2) ∉ chain (mk22 B W E E) (0,0) := by decide

/-! ## Liberties -/

/-- A stone whose every neighbor is an enemy stone has no liberty. -/
theorem no_liberty_when_surrounded :
    ¬ HasLiberty (mk22 B W W E) (0,0) := by decide

/-- One empty neighbor suffices. -/
theorem liberty_from_one_empty_neighbor :
    HasLiberty (mk22 B W E E) (0,0) := by decide

/-- Liberties belong to the whole chain, not to the stone: a two-stone chain
lives while either stone breathes. -/
theorem liberty_is_shared_across_chain :
    HasLiberty (mk33 (fun r c =>
      if (r,c) = (0,0) ∨ (r,c) = (0,1) then B
      else if (r,c) = (0,2) ∨ (r,c) = (1,0) then W else E)) (0,0) := by decide

/-! ## Playing a move -/

/-- Playing the last liberty of an enemy stone removes it. -/
theorem play_captures :
    resolve' (mk33 (fun r c =>
        if (r,c) = (0,0) then W else if (r,c) = (0,1) then B else E))
      Color.black (1,0) (0,0) = none := by decide

/-- Suicide is illegal: a point whose only neighbors are enemy stones, where
the move captures nothing. -/
theorem suicide_is_illegal :
    ¬ PlayableAt' (mk33 (fun r c =>
        if (r,c) = (0,1) ∨ (r,c) = (1,0) then W else E)) Color.black (0,0) := by
  decide

/-- The same point is legal for the other color, which joins a living chain. -/
theorem same_point_legal_for_owner :
    PlayableAt' (mk33 (fun r c =>
      if (r,c) = (0,1) ∨ (r,c) = (1,0) then W else E)) Color.white (0,0) := by
  decide

/-- Capture is resolved before suicide is judged, so a move that would
otherwise be suicide is legal when it captures. -/
theorem capture_precedes_suicide_test :
    PlayableAt' (mk33 (fun r c =>
      if (r,c) = (0,1) then W
      else if (r,c) = (0,2) ∨ (r,c) = (1,1) then B
      else if (r,c) = (1,0) then W else E)) Color.black (0,0) := by decide

/-- An occupied point is never playable. -/
theorem occupied_not_playable :
    ¬ PlayableAt' (mk22 B E E E) Color.black (0,0) := by decide

/-! ## Area scoring -/

/-- An empty board scores nothing for either color. -/
theorem area_empty_board :
    area' (mk22 E E E E) Color.black = 0 ∧
    area' (mk22 E E E E) Color.white = 0 := by decide

/-- A lone stone takes the whole board. -/
theorem area_lone_stone :
    area' (mk22 B E E E) Color.black = 4 ∧
    area' (mk22 B E E E) Color.white = 0 := by decide

/-- An empty point reaching both colors is dame and counts for neither. -/
theorem area_dame_counts_for_neither :
    area' (mk22 B E E W) Color.black = 1 ∧
    area' (mk22 B E E W) Color.white = 1 := by decide

/-- An eye counts for the color enclosing it, and the areas partition the
board when there is no dame. -/
theorem area_eye_and_partition :
    area' (mk33 (fun r c =>
      if (r,c) = (0,1) ∨ (r,c) = (1,0) then B
      else if (r,c) = (0,0) then E else W)) Color.black = 3 ∧
    area' (mk33 (fun r c =>
      if (r,c) = (0,1) ∨ (r,c) = (1,0) then B
      else if (r,c) = (0,0) then E else W)) Color.white = 6 := by decide

end Sanity
end Superko
