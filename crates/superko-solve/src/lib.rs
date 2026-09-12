//! Exact solvers over history-carrying states: who wins, and by how much.
//!
//! Three jobs, of which two are done:
//!
//! 1. **Decide.** [`naive::wins_for`] and [`search::decide`] answer the
//!    question `Superko.WinsFor` asks — does a named color have a winning
//!    strategy from this state under this repetition rule — by the same
//!    recursion `Superko.decideWins` uses.
//!
//! 2. **Score.** [`naive::value`] and [`search::solve`] return the minimax
//!    **area difference** of the finished game: Black maximizes Black's area
//!    less White's, White minimizes it. `Defs.lean` has no counterpart —
//!    `WinsFor` is a two-valued game at a fixed komi — so the score is a
//!    notion of this crate, and what ties it back to `Defs.lean` is the
//!    threshold agreement of the next paragraph.
//!
//! 3. **Quotient.** The Myhill-Nerode congruence on histories, claim C-15.
//!    Not started.
//!
//! # The threshold agreement, and its status
//!
//! Black wins at komi floor `k` exactly when the minimax score exceeds `k`.
//! That is a short induction over the game tree and it is **not proved here
//! or in Lean**: it is `computed`, checked by the fast engine in
//! `tests/agreement.rs` at every komi floor from `-(m·n) - 1` to `m·n + 1`, at
//! every root position of 1×1, 1×2, 1×3, 1×4 and 2×2, under both rules and both
//! suicide conventions (C-55).
//!
//! A separating **witness** does not rest on it: it is stated as two verdicts
//! at one named komi, each produced by the decision recursion of job 1, which
//! mirrors `Superko.decideWins` and needs no threshold argument. A report of
//! **no** separation, and the minimality of a witness, do rest on it: the sweep
//! compares values and reaches winners only through the agreement, so on boards
//! outside the range above they are statements about values alone.
//!
//! # Two engines, one of them the arbiter
//!
//! [`naive`] recurses over `superko_rules::reference::State`, cloning a state
//! and its archive at every node, running the rule predicates themselves and
//! pruning nothing. It is the arbiter, and it is unusable past about four
//! points.
//!
//! [`search`] is the fast path: alpha-beta over the transition table with make
//! and unmake on a dense archive, sharing no control flow with the naive one.
//! Alpha-beta returns the minimax value of the tree it searches whatever order
//! it visits moves in, so the value is engine-independent and the node count
//! is not.
//!
//! There is **no transposition table and no memoization**, deliberately. Two
//! states with the same position and different archives do not have the same
//! continuations, so a table keyed on less than the whole archive is the
//! graph-history-interaction error, and a table keyed on the whole archive is
//! about the size of the tree it would prune (`superko_graph::enumerate`).
//!
//! # Not trusted
//!
//! Nothing here establishes anything. A number this crate produces is a
//! `computed` ledger row; it becomes `proved` when Lean checks a certificate,
//! which for a verdict means kernel evaluation of `Superko.decideWins` at the
//! same position, rule and komi.
//!
//! # Status
//!
//! The two engines and the separation sweep exist. What the tests establish is
//! `computed` and bounded: the two engines agree on the value and on both
//! colors' verdicts at every komi floor, at every root position, on every
//! board with `m · n <= 3` under both rules and both suicide conventions; on
//! 1×1, 1×2, 1×3, 1×4 and 2×2 the fast engine's verdicts satisfy determinacy
//! (C-28, `proved`) and the threshold agreement (C-55); and the 1×n empty-board
//! scores under positional superko reproduce the transcribed table of C-24 for
//! n ≤ 6.
//!
//! What they do not establish: agreement with `Defs.lean` on any board those
//! tests do not reach, or anything at all about a board the sweep did not
//! resolve within its node budget — a budget the sweep reports rather than
//! absorbs.

pub mod naive;
pub mod search;
pub mod separate;

pub use search::{Decision, MoveOrder, Solution, decide, solve};
pub use separate::{Separating, Sweep, sweep, sweep_above};
