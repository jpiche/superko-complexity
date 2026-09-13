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
//! suicide conventions (C-55). Floors outside that range are not tested. At
//! such a floor every leaf's winner test gives what it gives at the nearer end
//! floor of the range, because the area difference lies between `-(m·n)` and
//! `m·n` (read from `winner_at` and `RuleTable::area`, not tested); that the
//! searches return the same verdict there as well is not checked.
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
//! colors' verdicts at every komi floor from `-(m·n) - 1` to `m·n + 1`, at every
//! root position, on every board with `m · n <= 3` under both rules and both
//! suicide conventions; on
//! 1×1, 1×2, 1×3, 1×4 and 2×2 the fast engine's verdicts satisfy determinacy
//! (C-28, `proved`) and the threshold agreement (C-55); and the 1×n empty-board
//! scores under positional superko reproduce the transcribed table of C-24 for
//! n ≤ 6.
//!
//! The sweep of [`separate`] can spread its roots over threads
//! (`separate::Options::threads`, one by default): each thread owns its own
//! solvers over the shared transition table, and one fold adds the per-root
//! results up in root order. That the sweep and its body are the same at one,
//! two and fourteen threads is `computed` by `tests/threads.rs` on the boards
//! it names. The thread count is a performance choice and changes no body.
//! The two symmetry features below are performance features too, but a run
//! that uses them is under two unlicensed divergences and prints a different
//! body, and they are off by default.
//!
//! The symmetric sweep of [`separate`] transports values along orbits of the
//! board's symmetries and the color swap. That a value is unchanged by a
//! board symmetry and negated by the swap, and that a verdict at komi floor
//! `k` moves to the other color at floor `-k - 1` under the swap, is
//! `computed` under both rules — for verdicts, at every komi floor of
//! `tests/agreement.rs`'s range whose transported floor stays in that range —
//! at every root of every board of at most four points, 2×2 among them, under
//! both suicide conventions, and of 1×5 and 5×1 under
//! the no-suicide rule; on 1×5 and 5×1 with suicide removing its own stones it
//! is `computed` only at the roots resolved within the budgets
//! `tests/symmetry.rs` names, which leave 1 628 of 2 916 value pairs and
//! 28 136 of 71 928 verdict comparisons on each uncompared. It is not proved.
//! The sweep with symmetry agrees with the sweep without it on the same boards
//! — on 1×5 and 5×1 with suicide removing its own stones only at the 72 of 486
//! roots each that resolve within the budget that file names — and with the
//! naive engine's value at every root of every board of at most three points.
//! On 2×3 and 3×2 under the no-suicide rule, whose group has the four elements
//! of a non-square rectangle, the values of the sweeps with canonical roots,
//! with mirrored moves and with both are `computed` equal to the plain sweep's
//! only at the roots and rules both resolved within 10⁵ nodes a search: 1 504
//! of 2 916 on each board and setting, the other 1 412 uncompared, and no
//! verdict compared
//! (`tests/symmetry.rs`, ignored in debug).
//!
//! The mirrored moves of [`search`] skip a play at a state a board symmetry
//! fixes, archive included, when the symmetry maps an earlier play onto it.
//! That the value and both colors' verdicts at every komi floor from
//! `-(m·n) - 1` to `m·n + 1` are the same with the skip as without it is
//! `computed` under both rules and both suicide conventions at every root of
//! 1×1, 1×2, 2×1, 1×3, 3×1, 1×4, 4×1 and 2×2 (`tests/mirrored.rs`), and of 1×5
//! and 5×1 under the no-suicide rule; with suicide removing its own stones on
//! 1×5 and 5×1 it is `computed` only where both searches resolved within the
//! budgets that file names (440 of 972 values and 15 976 of 25 272 verdicts on
//! each). The fast engine with mirrored moves agrees with the naive engine on
//! every value and verdict of every board of at most three points
//! (`tests/agreement.rs`), and reproduces the published 1×n values for n ≤ 6
//! (`tests/published.rs`). On 2×3 and 3×2 only values are compared, at the
//! resolved roots of the paragraph above. The witness verdicts of a sweep are
//! handed the board's maps exactly when mirrored moves are on: `tests/mirrored.rs`
//! checks it through `Sweep::lines_with_verdicts`, which `Sweep::lines` wraps,
//! on a record put by hand into a 1×4 sweep, and under all four settings. It is
//! not proved; the fact it would follow from is the `board-symmetry` divergence.
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
