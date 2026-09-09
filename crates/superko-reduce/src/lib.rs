//! Reduction gadgets and their verification.
//!
//! **A reduction is a program.** This crate takes an instance of a source
//! problem — a QBF, a geography graph, a formula game — and emits a Go
//! position together with the claimed correspondence. A checker then solves
//! both sides exhaustively on small instances and asserts the outcomes match.
//!
//! That turns "is my gadget right" from proof-reading into a test suite, which
//! is how gadget-based lower bounds are actually built. It is also the only
//! practical defense against the failure mode of this literature: a
//! construction that is nearly right, whose error is invisible in prose.
//!
//! First target is the Lichtenstein-Sipser reduction (C-2), which is not
//! novel — the point is to build the workbench against a reduction already
//! known to be correct, and to own the PSPACE-hardness bound for this
//! project's exact ruleset rather than inheriting it.
//!
//! Later, the harder ones: Robson's ko gadgets (C-4), and the Walraet-Tromp
//! Gray-code construction (C-10) made *steerable* so that players simulate a
//! no-repeat formula game (C-5). That last is the concrete route to
//! EXPSPACE-hardness and, as far as the literature shows, untried.
//!
//! Not trusted. Note the asymmetry: a verified gadget is evidence about the
//! gadget, and a hardness *theorem* needs the complexity infrastructure
//! Mathlib lacks (C-20).
//!
//! # Status
//!
//! Empty.
