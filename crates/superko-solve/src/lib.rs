//! Exact solvers over history-carrying states, and the history congruence.
//!
//! Two jobs:
//!
//! 1. **Solve.** Minimax over states that carry their history, under both
//!    repetition rules. Weninger and Hayward's Positional Linear Go solver
//!    carries exactly this state — (player to move, position, set of earlier
//!    positions, whether the previous move was a pass) — and reaches 1x9.
//!    Their published table is an acceptance test, and its disputed 1x9 entry
//!    (C-11) is an early target.
//!
//! 2. **Quotient.** Compute the Myhill-Nerode congruence on histories: two
//!    histories are equivalent when no continuation distinguishes them. Its
//!    index `H(n)` is claim C-15, and the growth rate decides which bound is
//!    worth attacking — see `docs/open-questions.md` §4 for the threshold,
//!    fixed in advance.
//!
//! The graph-history-interaction problem is unavoidable here: a transposition
//! table keyed on position alone is unsound when legality depends on history.
//! MIGOS handles it by putting situational properties in the hash. Whatever
//! this crate does, it does deliberately and documents.
//!
//! Not trusted. A solver result is `computed`; it becomes `proved` when a
//! strategy certificate checks in Lean.
//!
//! # Status
//!
//! Empty.
