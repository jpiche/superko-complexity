//! Counting positions, and counting the position graph.
//!
//! Neither count here has any history in it, so both exercise chains,
//! liberties and capture with the repetition rule out of the way. Both are
//! brute force over every position code of the board and call
//! `superko_rules::reference` directly, never the transition table: this module
//! is a second, slower reading, and a cross-check on the table rather than a
//! consumer of it.
//!
//! # What is counted
//!
//! A position is **legal** when no chain on it lacks a liberty. That is the
//! notion Tromp–Farnebäck's `L(m, n)` counts (C-23, `cited`); `Defs.lean` has
//! no predicate for it, because the rules reach only positions that satisfy it.
//!
//! The **position graph** has the legal positions as its nodes. An edge
//! `p -> q` is a play of either color that the convention in force permits and
//! that carries `p` to a different position `q`; edges are counted as distinct
//! pairs, so two plays reaching the same `q` from the same `p` are one edge. On
//! every board censused below the two countings coincide — no two plays from
//! one legal position reach the same different position (`computed`, the run of
//! 2026-09-11) — so the published figures do not settle which counting their
//! author meant, and the distinct-pair reading is this module's choice.
//!
//! A play that returns `p` itself is a **self-loop**, and self-loops *are*
//! counted per play: on 1×3 under the suicide-permitting convention the figure
//! 4 counts two self-capturing White plays into `.X.` and two Black plays into
//! `.O.`, where the distinct-pair counting would give 2. That is the one place
//! the published figures do settle the question.
//!
//! # The figures these reproduce
//!
//! Tromp–Farnebäck's Figures 1 and 2 are drawn under a suicide-permitting rule
//! with self-loops excluded: 5 nodes and 12 edges on 1×2, 15 and 42 on 1×3
//! (`cited`). Under `Defs.lean`'s suicide-forbidding rule the same census is 5
//! and 8, and 15 and 36 (scratch enumeration of 2026-09-11, `computed`). Both
//! are checked, each under its own convention, in `tests/census.rs`.

use std::collections::BTreeSet;

use superko_rules::code::{PosCode, code_space, decode, encode};
use superko_rules::config::{Dims, Suicide};
use superko_rules::reference::{Color, Position, has_liberty, playable_at, resolve_under};

/// Whether no chain of this position lacks a liberty.
#[must_use]
pub fn is_legal(b: &Position) -> bool {
    b.dims()
        .points()
        .all(|p| b.get(p).is_none() || has_liberty(b, p))
}

/// How many positions of the board are legal — Tromp–Farnebäck's `L(m, n)`.
///
/// # Panics
///
/// Panics when the board is too large for a position code (`m · n > 20`).
#[must_use]
pub fn legal_positions(dims: Dims) -> u64 {
    (0..code_space(dims))
        .filter(|&code| is_legal(&decode(dims, PosCode(code))))
        .count()
        .try_into()
        .expect("a legal-position count fits a u64")
}

/// The size of the position graph.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GraphCensus {
    /// Legal positions.
    pub nodes: u64,
    /// Distinct pairs `p -> q` with `q != p` that one play of either color
    /// reaches.
    pub edges: u64,
    /// Plays that return the position they were made from. Counted per play:
    /// on 1×3 under the suicide-permitting convention the two self-capturing
    /// White plays into `.X.` are two self-loops, and that is the counting the
    /// published figure of 4 agrees with.
    pub self_loops: u64,
}

impl GraphCensus {
    /// The body lines a results file carries, in a fixed order.
    #[must_use]
    pub fn lines(&self) -> Vec<String> {
        vec![
            format!("nodes={}", self.nodes),
            format!("edges={}", self.edges),
            format!("self-loops={}", self.self_loops),
        ]
    }
}

/// Census the position graph under a suicide convention.
///
/// # Panics
///
/// Panics when the board is too large for a position code (`m · n > 20`).
#[must_use]
pub fn position_graph(dims: Dims, suicide: Suicide) -> GraphCensus {
    let mut census = GraphCensus::default();
    for code in 0..code_space(dims) {
        let b = decode(dims, PosCode(code));
        if !is_legal(&b) {
            continue;
        }
        census.nodes += 1;
        let mut successors = BTreeSet::new();
        for c in [Color::Black, Color::White] {
            for p in dims.points() {
                if !playable_at(&b, c, p, suicide) {
                    continue;
                }
                let q = resolve_under(&b, c, p, suicide);
                if q == b {
                    census.self_loops += 1;
                } else {
                    successors.insert(encode(&q));
                }
            }
        }
        census.edges += u64::try_from(successors.len()).expect("an edge count fits a u64");
    }
    census
}
