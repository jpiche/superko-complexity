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

-- The computable decider and the bridges to Defs.lean (Decide.lean).
#print axioms Superko.mem_allMoves
#print axioms Superko.afterC_eq_after
#print axioms Superko.step'_toState
#print axioms Superko.ended'_iff
#print axioms Superko.winnerZ_eq_winner
#print axioms Superko.decideWins_sound
#print axioms Superko.playMeasure'_eq
#print axioms Superko.decideWins_mono
#print axioms Superko.decideWins_complete_aux
#print axioms Superko.start'_toState
#print axioms Superko.ssk'_faithful
#print axioms Superko.psk'_faithful
#print axioms Superko.playMeasure'_start

-- C-31, the input encoding (Encoding.lean).
#print axioms Superko.Enc.natOf_bits
#print axioms Superko.Enc.length_dbl
#print axioms Superko.Enc.length_sd
#print axioms Superko.Enc.readSd_append
#print axioms Superko.Enc.readSd_sd
#print axioms Superko.Enc.readSd_length_lt
#print axioms Superko.Enc.readSd_len
#print axioms Superko.Enc.cellOf_bits
#print axioms Superko.Enc.cellFlat_idx
#print axioms Superko.Enc.idx_lt
#print axioms Superko.Enc.length_boardBits
#print axioms Superko.Enc.boardBits_even
#print axioms Superko.Enc.boardBits_odd
#print axioms Superko.Enc.cellAt_boardBits
#print axioms Superko.Enc.boardOf_boardBits
#print axioms Superko.Enc.length_komiBits
#print axioms Superko.Enc.readKomi_ne_nil
#print axioms Superko.Enc.readKomi_komiBits
#print axioms Superko.Enc.colorOf_colorBit
#print axioms Superko.Enc.dec_enc
#print axioms Superko.Enc.enc_inj_dims
#print axioms Superko.Enc.enc_inj
#print axioms Superko.Enc.enc_inj_full
#print axioms Superko.Enc.num_halfInteger
#print axioms Superko.Enc.den_halfInteger
#print axioms Superko.Enc.length_enc
#print axioms Superko.Enc.C31_enc_length_lower
#print axioms Superko.Enc.enc_length_upper
#print axioms Superko.Enc.size_le_self
#print axioms Superko.Enc.length_enc_halfInteger
#print axioms Superko.Enc.length_enc_halfInteger_poly
#print axioms Superko.Enc.four_pow_eq
#print axioms Superko.Enc.three_pow_le_two_pow_length
#print axioms Superko.Enc.card_situation_le_two_pow_length
#print axioms Superko.Enc.card_situation_mul_le
#print axioms Superko.Enc.lengthBound_le_two_pow
#print axioms Superko.Enc.length_encSparse_empty
#print axioms Superko.Enc.sparse_encoding_refuted
#print axioms Superko.Enc.blackWinsFrom_black
#print axioms Superko.Enc.decStrict_enc
#print axioms Superko.Enc.enc_of_decStrict
#print axioms Superko.Enc.goLangBlack_subset
#print axioms Superko.Enc.goLang_eq_decStrict
#print axioms Superko.Enc.two_mul_le_length_of_decStrict
#print axioms Superko.Enc.two_mul_le_length_of_dec
#print axioms Superko.Enc.margin_ne_zero_halfInteger
#print axioms Superko.Enc.winner_swap_iff

-- C-27, komi through its floor (Results/C27_Komi.lean).
#print axioms Superko.area_add_area_le
#print axioms Superko.winner_eq_of_floor_eq
#print axioms Superko.winner_eq_halfInteger
#print axioms Superko.winner_const_of_floor_ge
#print axioms Superko.winner_congr_floor
#print axioms Superko.blackWins_congr_floor
#print axioms Superko.C27_blackWins_iff_halfInteger

-- C-28, determinacy (Results/C28_Determinacy.lean).
#print axioms Superko.C28_determined
#print axioms Superko.C28_determined_ssk
#print axioms Superko.C28_determined_psk
#print axioms Superko.C28_not_both
#print axioms Superko.C28_exactly_one

-- C-29, the decider decides BlackWins (Results/C29_DeciderCorrect.lean).
#print axioms Superko.C29_decideWins_iff_blackWins
#print axioms Superko.C29_decideWins_iff_blackWinsPSK
#print axioms Superko.C29_decideWins_iff_winsFor_black
#print axioms Superko.not_blackWins_11_half
#print axioms Superko.blackWins_11_neg_half
#print axioms Superko.not_blackWins_12_six_half
#print axioms Superko.blackWins_12_neg_half
#print axioms Superko.not_blackWinsPSK_12_half

-- C-41 to C-44, what the value sees of the archive (Compress.lean).
#print axioms Superko.resolve_self
#print axioms Superko.Compress.winsFor_transfer
#print axioms Superko.Compress.winsFor_cone_congr
#print axioms Superko.Compress.winsFor_seen_inter_cone
#print axioms Superko.Compress.winsFor_insert_unreachable
#print axioms Superko.Compress.decideWins_iff_winsFor
#print axioms Superko.Compress.sitStep_empty_board

-- C-50, C-51, what separates the two superko rules (Results/C50_Mechanism.lean).
#print axioms Superko.now_mem_seen_start
#print axioms Superko.now_mem_seen_step
#print axioms Superko.eq_or_eq_other
#print axioms Superko.pass_seen_both
#print axioms Superko.C50_pass_closes_board
#print axioms Superko.resolve_keeps_mover
#print axioms Superko.C51_play_changes_board
#print axioms Superko.C51_two_plays_by_one_color_change_board
