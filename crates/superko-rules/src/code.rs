//! Two ways to name a position: an internal integer code, and the string form
//! that crosses a file boundary.
//!
//! [`PosCode`] is a base-3 index over **all** `3^(m·n)` positions, legal and
//! illegal alike, with digit `row · n + col` and digit values 0 empty, 1
//! black, 2 white. It is the index of [`crate::table`] and of
//! [`crate::archive`], it is internal, and nothing writes one to a file: a
//! code is meaningless without the board it was taken on, and a file that
//! carried codes would be a file whose meaning depended on a convention no
//! reader could check.
//!
//! What crosses a file boundary is [`render`]: rows of `.`, `X` and `O`,
//! row-major, separated by `/`. [`parse`] takes the dimensions explicitly and
//! refuses anything else — a parser that inferred the board from the text
//! would accept a 1×4 board as a 2×2 one, and the two have different counts.

use crate::config::{Dims, Point};
use crate::reference::{Color, Position};
use core::fmt;

/// The largest board a [`PosCode`] indexes: `3^20` is below `u32::MAX`, and
/// `3^21` is not.
pub const MAX_CODED_POINTS: usize = 20;

/// A base-3 index over all `3^(m·n)` positions of a board.
///
/// Digit `row · n + col` holds 0 for empty, 1 for black, 2 for white. Every
/// position has a code and every code below `3^(m·n)` names a position, so a
/// dense table indexed by code needs no occupancy map.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PosCode(pub u32);

impl fmt::Display for PosCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The number of codes on this board, `3^(m·n)`.
///
/// # Panics
///
/// Panics when `m · n` exceeds [`MAX_CODED_POINTS`].
#[must_use]
pub fn code_space(dims: Dims) -> u32 {
    let points = dims.point_count();
    assert!(
        points <= MAX_CODED_POINTS,
        "a position code covers at most {MAX_CODED_POINTS} points, not {points}"
    );
    3u32.pow(u32::try_from(points).expect("point count fits a u32"))
}

/// The code of a position.
///
/// # Panics
///
/// Panics when the board exceeds [`MAX_CODED_POINTS`] points.
#[must_use]
pub fn encode(b: &Position) -> PosCode {
    let dims = b.dims();
    let _ = code_space(dims);
    let mut code = 0u32;
    for (i, cell) in b.cells().iter().enumerate() {
        let digit = match cell {
            None => 0,
            Some(Color::Black) => 1,
            Some(Color::White) => 2,
        };
        code += digit * 3u32.pow(u32::try_from(i).expect("point index fits a u32"));
    }
    PosCode(code)
}

/// The position a code names.
///
/// # Panics
///
/// Panics when the board exceeds [`MAX_CODED_POINTS`] points, or when the code
/// is not below `3^(m·n)`.
#[must_use]
pub fn decode(dims: Dims, code: PosCode) -> Position {
    assert!(
        code.0 < code_space(dims),
        "code {code} is not a position of the {dims} board"
    );
    let mut rest = code.0;
    let mut cells = vec![None; dims.point_count()];
    for cell in &mut cells {
        *cell = match rest % 3 {
            0 => None,
            1 => Some(Color::Black),
            _ => Some(Color::White),
        };
        rest /= 3;
    }
    Position::from_cells(dims, &cells)
}

/// The row-major string form: rows of `.`, `X` (black) and `O` (white),
/// separated by `/`.
#[must_use]
pub fn render(b: &Position) -> String {
    let dims = b.dims();
    let mut out = String::with_capacity(dims.point_count() + dims.rows);
    for row in 0..dims.rows {
        if row > 0 {
            out.push('/');
        }
        for col in 0..dims.cols {
            out.push(match b.get(Point::new(row, col)) {
                None => '.',
                Some(c) => c.glyph(),
            });
        }
    }
    out
}

/// Why a string is not a position of the given board.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The text has a different number of rows than the board.
    RowCount {
        /// Rows the board has.
        expected: usize,
        /// Rows the text has.
        found: usize,
    },
    /// A row has a different length than the board's width — a ragged board.
    RowLength {
        /// The row, counted from zero.
        row: usize,
        /// Columns the board has.
        expected: usize,
        /// Columns the row has.
        found: usize,
    },
    /// A character that is not `.`, `X` or `O`.
    BadChar {
        /// The row, counted from zero.
        row: usize,
        /// The column, counted from zero.
        col: usize,
        /// The offending character.
        found: char,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RowCount { expected, found } => {
                write!(f, "expected {expected} rows, found {found}")
            }
            Self::RowLength {
                row,
                expected,
                found,
            } => write!(f, "row {row}: expected {expected} columns, found {found}"),
            Self::BadChar { row, col, found } => {
                write!(f, "row {row} column {col}: {found:?} is not one of . X O")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// The position a string names on the given board.
///
/// Rows are separated by `/` or by a newline; every row's length and the row
/// count are checked against `dims`, so a ragged board is a refusal and never
/// a silently reshaped one.
///
/// Whitespace around the whole text and around each row is ignored, and
/// nothing else is. An empty row is a row of length zero and is counted as
/// one: `"XX//.."` is three rows on any board and is refused, rather than
/// becoming the two rows `"XX"` and `".."` with a separator dropped. A parser
/// that discarded empty rows would accept a text with a row missing and
/// silently score a different position.
///
/// # Errors
///
/// Returns a [`ParseError`] naming the first row, column and character that
/// does not fit the board.
pub fn parse(dims: Dims, text: &str) -> Result<Position, ParseError> {
    let rows: Vec<&str> = text.trim().split(['/', '\n']).map(str::trim).collect();
    if rows.len() != dims.rows {
        return Err(ParseError::RowCount {
            expected: dims.rows,
            found: rows.len(),
        });
    }
    let mut cells = vec![None; dims.point_count()];
    for (r, row) in rows.iter().enumerate() {
        let glyphs: Vec<char> = row.chars().collect();
        if glyphs.len() != dims.cols {
            return Err(ParseError::RowLength {
                row: r,
                expected: dims.cols,
                found: glyphs.len(),
            });
        }
        for (c, ch) in glyphs.into_iter().enumerate() {
            let cell = match ch {
                '.' => None,
                'X' => Some(Color::Black),
                'O' => Some(Color::White),
                found => {
                    return Err(ParseError::BadChar {
                        row: r,
                        col: c,
                        found,
                    });
                }
            };
            cells[dims.index(Point::new(r, c))] = cell;
        }
    }
    Ok(Position::from_cells(dims, &cells))
}
