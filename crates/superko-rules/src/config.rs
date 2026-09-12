//! The board and the two variant axes.
//!
//! `Dims` and `Point` are the board geometry that every item of
//! [`crate::reference`] takes; `Suicide` and `Repetition` are the two axes
//! along which a run may depart from `Defs.lean`, and `RuleConfig` is the
//! bundle a caller passes around and a witness header prints.

use core::fmt;

use crate::reference::Rat;

/// The dimensions of the board: `rows` is Lean's `m`, `cols` is Lean's `n`.
///
/// Lean carries `m` and `n` as implicit type indices, so a `Point 2 2` cannot
/// be handed to a `Position 1 3`. Here they are runtime values, and the
/// mismatch that Lean's typing makes unrepresentable is caught by an
/// assertion instead.
// DIVERGENCE: dims-are-runtime
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dims {
    /// Rows — Lean's `m`.
    pub rows: usize,
    /// Columns — Lean's `n`.
    pub cols: usize,
}

impl Dims {
    /// A board of `rows` × `cols` points.
    ///
    /// # Panics
    ///
    /// Panics when either dimension is zero. Lean admits `m = 0`, where the
    /// board has no points at all; this crate refuses it, because every
    /// consumer indexes a non-empty board and a zero dimension has no
    /// experiment behind it.
    #[must_use]
    pub const fn new(rows: usize, cols: usize) -> Self {
        assert!(rows > 0 && cols > 0, "a board dimension must be positive");
        Self { rows, cols }
    }

    /// The number of points, `m · n`.
    #[must_use]
    pub const fn point_count(self) -> usize {
        self.rows * self.cols
    }

    /// Whether `p` lies on this board.
    #[must_use]
    pub const fn contains(self, p: Point) -> bool {
        p.row < self.rows && p.col < self.cols
    }

    /// The row-major index of `p`, Lean's digit position `row · n + col`.
    ///
    /// # Panics
    ///
    /// Panics when `p` is off the board.
    #[must_use]
    pub const fn index(self, p: Point) -> usize {
        assert!(self.contains(p), "point off the board");
        p.row * self.cols + p.col
    }

    /// The point at row-major index `i`, the inverse of [`Dims::index`].
    ///
    /// # Panics
    ///
    /// Panics when `i` is not an index of this board.
    #[must_use]
    pub const fn point_at(self, i: usize) -> Point {
        assert!(i < self.point_count(), "index off the board");
        Point {
            row: i / self.cols,
            col: i % self.cols,
        }
    }

    /// Every point, row-major. This is the order `all_moves` plays in and the
    /// order a position's string form is written in.
    pub fn points(self) -> impl Iterator<Item = Point> {
        let cols = self.cols;
        (0..self.rows).flat_map(move |row| (0..cols).map(move |col| Point { row, col }))
    }
}

impl fmt::Display for Dims {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}x{}", self.rows, self.cols)
    }
}

/// A point of the board.
///
/// **Mirrors** `Superko.Point`.
///
/// Lean's `Point m n` is `Fin m × Fin n`, so rows and columns are separate
/// coordinates that cannot be confused with each other or with a flat index.
/// This type keeps them separate for the same reason; the flat index is
/// [`Dims::index`] and is never the identity of a point.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Point {
    /// The row coordinate — Lean's `Fin m` component.
    pub row: usize,
    /// The column coordinate — Lean's `Fin n` component.
    pub col: usize,
}

impl Point {
    /// The point at `(row, col)`.
    #[must_use]
    pub const fn new(row: usize, col: usize) -> Self {
        Self { row, col }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({},{})", self.row, self.col)
    }
}

/// What happens to a play that leaves its own chain without a liberty.
///
/// `Forbid` is `Defs.lean`: `Superko.PlayableAt` refuses the move. `RemoveOwn`
/// is the Tromp–Taylor and New Zealand reading, where the move is legal and
/// the player's own stones come off; it has **no counterpart in `Defs.lean`**,
/// so a number produced under it is a number about this crate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suicide {
    /// Suicide is illegal — the AGA rule, and `Defs.lean`'s.
    #[default]
    Forbid,
    /// Suicide is legal and removes the player's own dead stones.
    RemoveOwn,
}

impl Suicide {
    /// The spelling the CLI accepts and a witness header prints.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Forbid => "forbid",
            Self::RemoveOwn => "remove-own",
        }
    }
}

impl fmt::Display for Suicide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

/// Which repetition rule is in force.
///
/// This names one of `Superko.SSK` and `Superko.PSK`; it is not Lean's
/// `Superko.Repetition`, which is the *type* of a repetition rule and belongs
/// with the `WinsFor` recursion in the solver crate. `Ssk` is the object of
/// the project; `Psk` is the comparison, and the rule the published counts are
/// under.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Repetition {
    /// Situational superko — a play may not recreate a seen situation.
    #[default]
    Ssk,
    /// Positional superko — a play may not recreate a seen position.
    Psk,
}

impl Repetition {
    /// The spelling the CLI accepts and a witness header prints.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Ssk => "ssk",
            Self::Psk => "psk",
        }
    }
}

impl fmt::Display for Repetition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

/// A resolved rule configuration: the board, the two variant axes and the
/// komi.
///
/// Every run prints this, so that a count is never quoted without the
/// conventions it was produced under.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuleConfig {
    /// The board.
    pub dims: Dims,
    /// What a self-capturing play does.
    pub suicide: Suicide,
    /// Which repetition rule is in force.
    pub repetition: Repetition,
    /// The komi, exactly, as a rational.
    pub komi: Rat,
}

impl RuleConfig {
    /// The configuration that mirrors `Defs.lean`: suicide forbidden,
    /// situational superko, komi zero.
    ///
    /// Komi zero is this crate's default and no rule text's. A tie goes to
    /// White (`Superko.winner`), so the tie convention is observable at an
    /// integer komi and invisible at a half-integer one.
    #[must_use]
    pub const fn new(dims: Dims) -> Self {
        Self {
            dims,
            suicide: Suicide::Forbid,
            repetition: Repetition::Ssk,
            komi: Rat::ZERO,
        }
    }

    /// The same configuration under a different repetition rule.
    #[must_use]
    pub const fn with_repetition(mut self, repetition: Repetition) -> Self {
        self.repetition = repetition;
        self
    }

    /// The same configuration under a different suicide convention.
    #[must_use]
    pub const fn with_suicide(mut self, suicide: Suicide) -> Self {
        self.suicide = suicide;
        self
    }

    /// The same configuration at a different komi.
    #[must_use]
    pub const fn with_komi(mut self, komi: Rat) -> Self {
        self.komi = komi;
        self
    }

    /// Whether this configuration is a literal reading of `Defs.lean` —
    /// that is, whether `Suicide::RemoveOwn`, which has no Lean counterpart,
    /// is out of play.
    #[must_use]
    pub const fn mirrors_defs(self) -> bool {
        matches!(self.suicide, Suicide::Forbid)
    }
}

impl fmt::Display for RuleConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "board {} rule {} suicide {} komi {}",
            self.dims, self.repetition, self.suicide, self.komi
        )
    }
}
