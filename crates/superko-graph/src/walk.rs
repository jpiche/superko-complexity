//! The naive enumerator, and the arbiter.
//!
//! This walks `superko_rules::reference::State` directly: a `BTreeSet` archive
//! of situations, a fresh state cloned at every node, and
//! `superko_rules::reference::legality` asked about every move. It shares no
//! control flow with [`crate::enumerate`] — not a helper, not a move loop, not
//! an archive — so that an agreement between the two is evidence about the
//! rules and not about one piece of code called twice.
//!
//! It is unusable past 1×3 at full depth. That is the point: the fast count is
//! quotable only where this one has agreed with it, which for 2×2 means
//! agreement to a depth cap deep enough that the archive is doing work.
//!
//! Nothing here is proved. An agreement between the two enumerators is
//! `computed`, on exactly the boards and depths the tests run.

use std::cmp::max;
use std::collections::BTreeSet;

use superko_rules::config::{Dims, Repetition, Suicide};
use superko_rules::reference::{Color, Position, Refusal, State, all_moves, ended, legality, step};

use crate::enumerate::Report;

/// The size of the archive the rule in force actually reads.
///
/// Situational superko reads situations, so the archive is `seen` itself.
/// Positional superko reads `seen` only through `.board`, so the archive it
/// reads is the set of positions — which is what [`crate::enumerate`] holds
/// under that rule, and the reason the two reports are comparable field by
/// field.
fn archive_size(st: &State, rep: Repetition) -> usize {
    match rep {
        Repetition::Ssk => st.seen().len(),
        Repetition::Psk => st
            .seen()
            .iter()
            .map(|s| &s.board)
            .collect::<BTreeSet<_>>()
            .len(),
    }
}

/// Add one, or panic rather than wrap.
fn inc(counter: &mut u64) {
    *counter = counter.checked_add(1).expect("a count overflowed u64");
}

struct Walk {
    dims: Dims,
    rep: Repetition,
    suicide: Suicide,
    depth_cap: Option<usize>,
    report: Report,
}

impl Walk {
    fn visit(&mut self, st: &State, depth: usize) {
        inc(&mut self.report.nodes);
        self.report.max_depth = max(self.report.max_depth, depth);
        self.report.max_archive = max(self.report.max_archive, archive_size(st, self.rep));

        if ended(st) {
            inc(&mut self.report.games);
            return;
        }

        if self.depth_cap == Some(depth) {
            inc(&mut self.report.truncated);
            return;
        }

        for mv in all_moves(self.dims) {
            match legality(st, mv, self.rep, self.suicide) {
                Ok(()) => {
                    let next = step(st, mv, self.suicide);
                    self.visit(&next, depth + 1);
                }
                Err(Refusal::Occupied) => inc(&mut self.report.refused_occupied),
                Err(Refusal::Suicide) => inc(&mut self.report.refused_suicide),
                Err(Refusal::Repetition) => inc(&mut self.report.refused_repetition),
            }
        }
    }
}

/// Count the games of a board by the naive walk, from the empty position with
/// Black to move.
///
/// `depth_cap` abandons a state that many moves from the root and counts it in
/// [`Report::truncated`], never in [`Report::games`].
///
/// # Panics
///
/// Panics when a count overflows `u64`.
#[must_use]
pub fn count_games(
    dims: Dims,
    rep: Repetition,
    suicide: Suicide,
    depth_cap: Option<usize>,
) -> Report {
    let mut walk = Walk {
        dims,
        rep,
        suicide,
        depth_cap,
        report: Report::default(),
    };
    let start = State::start(&Position::empty(dims), Color::Black);
    walk.visit(&start, 0);
    walk.report
}
