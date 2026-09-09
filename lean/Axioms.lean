/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity

/-!
# Axiom record

Not part of the library — `check-lean.sh` elaborates this file directly and
diffs its output against `results/axioms.txt`. Every top-level theorem in the
development is listed here.

A theorem acquiring `Lean.ofReduceBool` has picked up `native_decide` and put
the Lean compiler in the trusted base; the diff is how that gets noticed.
See `docs/trusted-base.md`.
-/

open Superko

-- Bridging lemmas: what makes a computed check a statement about Defs.lean.
#print axioms Superko.mem_chain_iff_reachable
#print axioms Superko.reaches_iff
#print axioms Superko.reaches_iff_occupied
#print axioms Superko.clear'_eq_clear
#print axioms Superko.resolve'_eq_resolve
#print axioms Superko.playableAt'_iff
#print axioms Superko.area'_eq_area

-- Sanity checks.
#print axioms Superko.Sanity.adj_orthogonal
#print axioms Superko.Sanity.adj_not_diagonal
#print axioms Superko.Sanity.adj_irreflexive
#print axioms Superko.Sanity.adj_symmetric
#print axioms Superko.Sanity.nbr_count_center
#print axioms Superko.Sanity.nbr_count_corner
#print axioms Superko.Sanity.chain_joins_adjacent
#print axioms Superko.Sanity.chain_skips_diagonal
#print axioms Superko.Sanity.chain_respects_color
#print axioms Superko.Sanity.no_liberty_when_surrounded
#print axioms Superko.Sanity.liberty_from_one_empty_neighbor
#print axioms Superko.Sanity.liberty_is_shared_across_chain
#print axioms Superko.Sanity.play_captures
#print axioms Superko.Sanity.suicide_is_illegal
#print axioms Superko.Sanity.same_point_legal_for_owner
#print axioms Superko.Sanity.capture_precedes_suicide_test
#print axioms Superko.Sanity.occupied_not_playable
#print axioms Superko.Sanity.area_empty_board
#print axioms Superko.Sanity.area_lone_stone
#print axioms Superko.Sanity.area_dame_counts_for_neither
#print axioms Superko.Sanity.area_eye_and_partition

-- C-13, termination (Results/C13_Termination.lean).
#print axioms Superko.ssk_excludesRepeats
#print axioms Superko.psk_excludesRepeats
#print axioms Superko.playMeasure_lt_of_follows
#print axioms Superko.wellFounded_follows
#print axioms Superko.C13_terminates
#print axioms Superko.C13_terminates_psk
#print axioms Superko.playMeasure_add_le
#print axioms Superko.C13_no_infinite_play
#print axioms Superko.playMeasure_start
#print axioms Superko.C13_length_bound
#print axioms Superko.C13_length_bound_explicit

-- Finiteness of the situation space (Basic.lean).
#print axioms Superko.card_color
#print axioms Superko.card_position
#print axioms Superko.card_situation
#print axioms Superko.ncard_seen_le_card
