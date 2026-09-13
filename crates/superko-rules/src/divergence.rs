//! Every way this workspace departs from a literal reading of `Defs.lean`.
//!
//! The enum is closed, and [`tools/check-mirror.sh`](../../../../tools/check-mirror.sh)
//! requires the set of `DIVERGENCE:` markers in the crate's source to equal
//! the set of slugs below. A departure that is not in this list is either a
//! bug or an unrecorded change of meaning, and the gate cannot tell which, so
//! it fails on both.
//!
//! A **license** is a Lean theorem that makes the departure invisible in the
//! result. One of the seven has one, `winner-via-floor-komi`; the other six
//! have no `lean_witness`, and each one's consequence sentence says what stands
//! in its place. Every witness header prints the consequence sentence of each
//! divergence the run was under, so a number is never quoted without them.
//!
//! `board-symmetry` and `color-swap` differ from the other five in kind: they
//! are not readings of a `Defs.lean` item but facts about the rules a search
//! relies on to skip work, and a run is under them only when it asked for that
//! (`superko separate --symmetry on`, under both, and `superko solve --symmetry
//! on`, under `board-symmetry` alone). Their markers are in
//! [`crate::symmetry`].

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
    /// A search takes values from the image of a root, or of a play at a state
    /// the symmetry fixes, under a symmetry of the board.
    BoardSymmetry,
    /// A search takes values, negated, from the root with the colors
    /// exchanged.
    ColorSwap,
}

impl Divergence {
    /// Every divergence, in the order a witness header prints them.
    pub const ALL: [Self; 7] = [
        Self::DimsAreRuntime,
        Self::SuicideRemoveOwn,
        Self::PskArchiveProjection,
        Self::RuleTableMemo,
        Self::WinnerViaFloorKomi,
        Self::BoardSymmetry,
        Self::ColorSwap,
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
            Self::BoardSymmetry => "board-symmetry",
            Self::ColorSwap => "color-swap",
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
            | Self::RuleTableMemo
            | Self::BoardSymmetry
            | Self::ColorSwap => None,
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
            Self::BoardSymmetry => {
                "Some roots take their values from the root a symmetry of the board maps \
                 them to instead of being searched, or some plays at a state a symmetry of \
                 the board fixes, archive included, are not searched because the symmetry \
                 maps an earlier play onto them; that the transition table commutes with \
                 every symmetry of the board is `computed` for boards of at most six points \
                 and for 3x3 and 3x4; that values are invariant under it is `computed` at \
                 every root of every board of at most five points except 1x5 and 5x1 with \
                 suicide removing its own stones, where it is `computed` only at the roots \
                 resolved within the node budgets `superko-solve`'s tests/symmetry.rs names \
                 (1288 of 2916 pairs of a root and any of its images, both rules, on each); \
                 that values and verdicts with those plays skipped equal those without is \
                 `computed` at every root, and at komi floors -(m*n)-1 to m*n+1, of every \
                 board of at most five points except 1x5 and 5x1 with suicide removing its \
                 own stones, where it is `computed` only where both searches resolved within \
                 the budgets of `superko-solve`'s tests/mirrored.rs (440 of 972 values and \
                 15976 of 25272 verdicts, both rules, on each); and none of it is proved."
            }
            Self::ColorSwap => {
                "Some roots take their values, negated, from the root with every stone's \
                 color exchanged and the other color to move instead of being searched; that \
                 the transition table commutes with the exchange is `computed` for boards of \
                 at most six points and for 3x3 and 3x4; that values negate under it is \
                 `computed` at every root of every board of at most five points except 1x5 \
                 and 5x1 with suicide removing its own stones, where it is `computed` only at \
                 the roots resolved within the node budgets `superko-solve`'s \
                 tests/symmetry.rs names; and neither is proved."
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
