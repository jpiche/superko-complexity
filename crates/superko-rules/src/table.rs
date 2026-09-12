//! The transition table: the fast path, built by calling the transliteration.
//!
//! [`RuleTable::build`] is the only constructor, and its body calls
//! [`crate::reference`]'s `resolve_under`, `playable_at` and `area` and
//! nothing else. There is no second implementation of the rules to drift from
//! the first: the table is a memo of this one, over every position, color and
//! point of the board.
//!
//! That is a divergence only in the sense that a memo is one — the table is
//! licensed by its own construction, and the test `table_is_its_generator`
//! re-derives every entry from the reference functions rather than trusting
//! that the constructor did what its body says.
// DIVERGENCE: rule-table-memo

use crate::code::{PosCode, code_space, decode, encode};
use crate::config::{Dims, Point, Suicide};
use crate::reference::{Color, Position, area, playable_at, resolve_under};
use core::fmt;

/// The largest board the table is built for by default: `3^12` codes × 2
/// colors × 12 points of successor, about 51 MB. Above it the build is
/// refused rather than attempted.
pub const MAX_TABLE_POINTS: usize = 12;

/// Why a table was not built.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TooLarge {
    /// The board asked for.
    pub dims: Dims,
    /// The point budget the build refuses to exceed.
    pub limit: usize,
}

impl fmt::Display for TooLarge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the {} board has {} points; the transition table covers at most {} \
             (3^{} codes of successor per color and point)",
            self.dims,
            self.dims.point_count(),
            self.limit,
            self.limit
        )
    }
}

impl std::error::Error for TooLarge {}

/// Every transition of a board, indexed by position code, color and point.
///
/// Holds, per `(code, color, point)`, the successor code and whether the play
/// is playable; and per `code`, the two area scores. An enumerator's inner
/// loop is then two array reads and a bit test, and runs no rule.
#[derive(Clone, Debug)]
pub struct RuleTable {
    dims: Dims,
    suicide: Suicide,
    points: usize,
    succ: Vec<u32>,
    playable: Vec<u64>,
    areas: Vec<[u8; 2]>,
}

impl RuleTable {
    /// Build the table for a board, by calling the reference functions at
    /// every position, color and point.
    ///
    /// Successors are defined at occupied points too, because
    /// `Superko.resolve` is total; whether the play is *allowed* there is the
    /// separate `playable` bit.
    ///
    /// # Errors
    ///
    /// Refuses a board of more than [`MAX_TABLE_POINTS`] points.
    ///
    /// # Panics
    ///
    /// Panics when an area score exceeds `u8::MAX`, which no board within the
    /// point budget can produce.
    pub fn build(dims: Dims, suicide: Suicide) -> Result<Self, TooLarge> {
        if dims.point_count() > MAX_TABLE_POINTS {
            return Err(TooLarge {
                dims,
                limit: MAX_TABLE_POINTS,
            });
        }
        let points = dims.point_count();
        let codes = code_space(dims) as usize;
        let slots = codes * 2 * points;
        let mut succ = vec![0u32; slots];
        let mut playable = vec![0u64; slots.div_ceil(64)];
        let mut areas = vec![[0u8; 2]; codes];

        for (code, areas_of_code) in areas.iter_mut().enumerate() {
            let b = decode(dims, PosCode(u32::try_from(code).expect("code fits a u32")));
            *areas_of_code = [
                u8::try_from(area(&b, Color::Black)).expect("area fits a u8"),
                u8::try_from(area(&b, Color::White)).expect("area fits a u8"),
            ];
            for (ci, c) in [Color::Black, Color::White].into_iter().enumerate() {
                for pi in 0..points {
                    let p = dims.point_at(pi);
                    let slot = (code * 2 + ci) * points + pi;
                    succ[slot] = encode(&resolve_under(&b, c, p, suicide)).0;
                    if playable_at(&b, c, p, suicide) {
                        playable[slot / 64] |= 1u64 << (slot % 64);
                    }
                }
            }
        }

        Ok(Self {
            dims,
            suicide,
            points,
            succ,
            playable,
            areas,
        })
    }

    /// The board this table covers.
    #[must_use]
    pub const fn dims(&self) -> Dims {
        self.dims
    }

    /// The suicide convention this table was built under.
    #[must_use]
    pub const fn suicide(&self) -> Suicide {
        self.suicide
    }

    /// The number of position codes, `3^(m·n)`.
    #[must_use]
    pub fn code_count(&self) -> usize {
        self.areas.len()
    }

    fn slot(&self, code: PosCode, c: Color, p: Point) -> usize {
        let code = code.0 as usize;
        assert!(
            code < self.areas.len(),
            "code is not a position of this board"
        );
        let ci = match c {
            Color::Black => 0,
            Color::White => 1,
        };
        (code * 2 + ci) * self.points + self.dims.index(p)
    }

    /// The code of the position after `c` plays at `p`, whether or not the
    /// play is allowed.
    ///
    /// # Panics
    ///
    /// Panics when the code or the point is not of this board.
    #[must_use]
    pub fn succ(&self, code: PosCode, c: Color, p: Point) -> PosCode {
        PosCode(self.succ[self.slot(code, c, p)])
    }

    /// Whether `c` may play at `p` on the board conditions alone — the
    /// repetition rule is the caller's business.
    ///
    /// # Panics
    ///
    /// Panics when the code or the point is not of this board.
    #[must_use]
    pub fn playable(&self, code: PosCode, c: Color, p: Point) -> bool {
        let slot = self.slot(code, c, p);
        self.playable[slot / 64] & (1u64 << (slot % 64)) != 0
    }

    /// The area score of `c` at this position.
    ///
    /// # Panics
    ///
    /// Panics when the code is not a position of this board.
    #[must_use]
    pub fn area(&self, code: PosCode, c: Color) -> u32 {
        let code = code.0 as usize;
        assert!(
            code < self.areas.len(),
            "code is not a position of this board"
        );
        let ci = match c {
            Color::Black => 0,
            Color::White => 1,
        };
        u32::from(self.areas[code][ci])
    }

    /// The position a code names, for a caller that has a code and wants the
    /// board back.
    ///
    /// # Panics
    ///
    /// Panics when the code is not a position of this board.
    #[must_use]
    pub fn position(&self, code: PosCode) -> Position {
        decode(self.dims, code)
    }
}
