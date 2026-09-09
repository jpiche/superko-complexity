//! Geography variants — the comparator for the folklore upper-bound argument.
//!
//! Demaine and Hearn observe that if all dynamical state lives in kos, as it
//! does in Robson's construction, the game is an instance of undirected vertex
//! geography, which is polynomial in the graph size; the graph being
//! exponential, the construction stays in EXPTIME (C-6).
//!
//! That is folklore about *one construction*. This crate exists so the claim
//! becomes testable rather than quotable: build the situation graph of a small
//! position and ask whether it is in fact undirected.
//!
//! The distinction is exactly what decides the reachability core — undirected
//! vertex geography is polynomial, while directed vertex, directed edge and
//! undirected edge geography are PSPACE-complete. Under superko a Go game is a
//! self-avoiding walk in situation space, which makes these the right
//! analogues.
//!
//! The open question this serves: **characterize exactly when a Go move
//! creates a directed edge in situation space.** Ko toggles are locally
//! reversible; captures in general are not.
//!
//! Not trusted.
//!
//! # Status
//!
//! Empty.
