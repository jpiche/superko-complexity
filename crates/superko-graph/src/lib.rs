//! The situation graph: exhaustive enumeration, a naive arbiter, and the
//! position-graph census.
//!
//! Vertices are situations — a position with a player to move — and edges are
//! legal moves. Under positional superko a game is a *simple path* from the
//! empty position (claim C-7, `cited`); under situational superko it need not
//! be (C-8, `cited`). That distinction is this crate's reason to exist.
//!
//! # Two enumerators and a census
//!
//! [`enumerate`] is the fast path: a depth-first search with make and unmake
//! over a dense archive, whose inner loop reads the transition table of
//! `superko-rules` and never runs a rule. [`walk`] is the naive enumerator
//! over `superko_rules::reference::State`, cloning a state per node and
//! sharing no control flow with the fast one; it is unusable past 1×3 at full
//! depth, and it is the arbiter. A fast count is quotable only where the naive
//! one has agreed with it, which for 2×2 means agreement to a depth cap.
//!
//! [`census`] counts legal positions and the position graph — nodes, edges and
//! self-loops — by brute force over every position code. It touches no
//! history at all, so it exercises chains, liberties and capture against two
//! published tables without the repetition rule in the way.
//!
//! # Symmetry
//!
//! Symmetry is **off and unimplemented**, and the sentence this doc used to
//! carry — that canonicalization under the board's eight symmetries is what
//! makes enumeration feasible — was wrong twice over.
//!
//! It is not a compression. Two positions in one orbit can carry archives in
//! different orbits, so identifying search states by a canonical position is
//! unsound under a history-dependent rule; when symmetry comes it is a
//! root-only decomposition and a cross-check.
//!
//! There are not always eight symmetries either. The group is derived from
//! `(m, n)`: the dihedral group of order 8 on a square board, the Klein
//! four-group on a rectangle, order 2 on a line. `docs/plans/rules-mirror-plan.md`
//! records the 2×2 decomposition the group would license.
//!
//! # Not trusted
//!
//! Nothing here establishes anything, for the reason `superko-rules` states:
//! a number this crate produces is a `computed` ledger row and a certificate,
//! and becomes `proved` only when a Lean checker accepts that certificate.
//!
//! # Status
//!
//! The enumerators and the census exist and agree with each other on the
//! boards the test suite reaches. What that establishes is `computed` and
//! bounded: the game counts 1, 9 and 907 on 1×1 through 1×3 under positional
//! superko and both suicide conventions, the fast and naive enumerators
//! agreeing on the whole report there and on 2×2 to a depth cap, and the
//! legal-position and position-graph censuses of `docs/plans/rules-mirror-plan.md`.
//!
//! On 1×4 under positional superko the run of 2026-09-11 found 2 098 407 841
//! games under `Suicide::RemoveOwn` — Tromp–Farnebäck's published figure,
//! reproduced exactly — and 719 178 893 under `Suicide::Forbid`, which is
//! `Defs.lean`'s rule (both `computed`; `tests/published.rs` carries the
//! witness line and the reason). The suicide convention is therefore
//! observable in a game count from 1×4 upward, which experiment 004's
//! hypothesis expected it not to be.
//!
//! What it does not establish: anything about 2×2 at full depth, which no test
//! runs, and anything at all about `Defs.lean` beyond what `superko-rules`
//! tests already claim. In particular, reproducing a published count under the
//! publication's own convention is evidence that this workspace computes what
//! that publication computed — not that either is a correct account of Go.

pub mod census;
pub mod enumerate;
pub mod walk;

pub use census::{GraphCensus, legal_positions, position_graph};
pub use enumerate::{Board, Options, Report, count_games};
