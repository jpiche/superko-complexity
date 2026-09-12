//! Every way this workspace departs from a literal reading of `Defs.lean`.
//!
//! The enum is closed, and [`tools/check-mirror.sh`](../../../../tools/check-mirror.sh)
//! requires the set of `DIVERGENCE:` markers in the crate's source to equal
//! the set of slugs below. A departure that is not in this list is either a
//! bug or an unrecorded change of meaning, and the gate cannot tell which, so
//! it fails on both.
//!
//! A **license** is a Lean theorem that makes the departure invisible in the
//! result. Two of the five have none: one is licensed by construction and
//! tested, the other has no Lean counterpart at all. Every witness header
//! prints the consequence sentence of each divergence the run was under, so a
//! number is never quoted without them.

use core::fmt;

/// A departure from a literal reading of `lean/SuperkoComplexity/Defs.lean`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Divergence {
    /// The board's dimensions are runtime values, not type indices.
    DimsAreRuntime,
    /// Suicide is legal and removes the player's own dead stones.
    SuicideRemoveOwn,
    /// The positional-superko archive reads positions, not situations.
    PskArchiveProjection,
    /// Transitions are read from a memo of the reference functions.
    RuleTableMemo,
    /// The winner is decided over ℤ at the floor of komi.
    WinnerViaFloorKomi,
}

impl Divergence {
    /// Every divergence, in the order a witness header prints them.
    pub const ALL: [Self; 5] = [
        Self::DimsAreRuntime,
        Self::SuicideRemoveOwn,
        Self::PskArchiveProjection,
        Self::RuleTableMemo,
        Self::WinnerViaFloorKomi,
    ];

    /// The slug, which is what a `DIVERGENCE:` marker in the source names and
    /// what a witness header prints.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::DimsAreRuntime => "dims-are-runtime",
            Self::SuicideRemoveOwn => "suicide-remove-own",
            Self::PskArchiveProjection => "psk-archive-projection",
            Self::RuleTableMemo => "rule-table-memo",
            Self::WinnerViaFloorKomi => "winner-via-floor-komi",
        }
    }

    /// The Lean theorem that licenses the departure, where one exists.
    ///
    /// `None` is not an oversight to be filled in later without comment: it
    /// means a result produced under this divergence is a result about this
    /// crate until the theorem is written.
    #[must_use]
    pub const fn lean_witness(self) -> Option<&'static str> {
        match self {
            Self::DimsAreRuntime
            | Self::SuicideRemoveOwn
            | Self::PskArchiveProjection
            | Self::RuleTableMemo => None,
            Self::WinnerViaFloorKomi => Some("Superko.winnerZ_eq_winner"),
        }
    }

    /// One sentence, printed in every witness header of a run this divergence
    /// was in force for.
    #[must_use]
    pub const fn consequence(self) -> &'static str {
        match self {
            Self::DimsAreRuntime => {
                "The board's dimensions are runtime values rather than type indices: a point \
                 off the board is refused by an assertion rather than made unrepresentable, \
                 two positions of different boards are unequal rather than untypable, and a \
                 zero dimension, which `Defs.lean` admits, is refused outright."
            }
            Self::SuicideRemoveOwn => {
                "A self-capturing play is legal and removes the player's own dead stones; \
                 `Defs.lean` has no counterpart, so a number produced under this convention \
                 is a number about this crate and not about the audit target."
            }
            Self::PskArchiveProjection => {
                "Positional superko reads a position's two archive bits instead of \
                 quantifying over archived situations; the one-line `Basic.lean` lemma that \
                 `PSK` consults `seen` only through `.board` is not written, so the license \
                 is `open`."
            }
            Self::RuleTableMemo => {
                "Transitions are read from a table whose only constructor calls the \
                 reference functions; that the table equals its generator is `computed` for \
                 boards with `m * n <= 6` and is not proved."
            }
            Self::WinnerViaFloorKomi => {
                "The winner is decided by comparing the floor of komi against the difference \
                 of the two area scores rather than by a rational subtraction, which \
                 `Superko.winnerZ_eq_winner` proves is the same verdict."
            }
        }
    }

    /// Whether a Lean theorem licenses the departure.
    #[must_use]
    pub const fn licensed(self) -> bool {
        self.lean_witness().is_some()
    }
}

impl fmt::Display for Divergence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}
