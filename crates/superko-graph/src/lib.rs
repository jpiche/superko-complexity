//! The situation graph: construction, canonicalization, exhaustive
//! enumeration.
//!
//! Vertices are situations — a position with a player to move — and edges are
//! legal moves. Under positional superko a game is a *simple path* from the
//! empty position (claim C-7); under situational superko it need not be
//! (C-8). That distinction is this crate's reason to exist.
//!
//! Canonicalization under the board's eight symmetries is what makes
//! enumeration feasible, and is also a place a subtle bug would go unnoticed:
//! symmetry reduction interacts with the history set, since two positions in
//! the same orbit may have incomparable histories.
//!
//! Not trusted. See `superko-rules`.
//!
//! # Status
//!
//! Empty.
