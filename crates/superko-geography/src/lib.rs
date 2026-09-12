//! Geography variants — the comparator for the folklore upper-bound argument.
//!
//! Demaine and Hearn observe that if all dynamical state lives in kos, as it
//! does in Robson's construction, the game is an instance of undirected vertex
//! geography, which is polynomial in the graph size; the graph being
//! exponential, the construction stays in EXPTIME (C-6).
//!
//! That is folklore about *one construction*, and this crate exists so the
//! claim becomes testable rather than quotable.
//!
//! # Two corrections to the sentence this file used to carry
//!
//! It said a Go game under superko is a self-avoiding walk in situation space.
//! That is C-7, and it is the **positional** statement. Under SSK a pass is
//! exempt from the repetition rule (C-18) and re-enters a situation already
//! seen, so the walk avoids itself on play edges only (C-8).
//!
//! It also proposed the experiment "build the situation graph of a small
//! position and ask whether it is undirected". That question now has an answer
//! and the answer is uninformative: any stone the opponent cannot capture is a
//! one-way edge, so no board's graph is undirected. `superko-graph::scc`
//! censused the coarser structure instead and found the graph to be one
//! mutually reachable mass above the empty board on every board through
//! `m · n = 12` (C-45, `computed`).
//!
//! # What is left to ask
//!
//! Two refinements that the census does not answer:
//!
//! 1. How many edges *inside* the giant component are one-way? That is the
//!    quantitative form of `docs/open-questions.md` §5's question, and C-6's
//!    mechanism needs the answer to be small rather than the component to be.
//! 2. Does the matching argument behind C-21 survive free pass edges and a
//!    terminal scored by area? C-21 is a normal-play theorem and Go is not a
//!    normal-play game; that gap is C-49, `open`, and no source addresses it.
//!
//! Not trusted.
//!
//! # Status
//!
//! Empty. The census that would have been this crate's first experiment lives
//! in `superko-graph::scc`, because it is a statement about the situation
//! graph rather than about geography.
