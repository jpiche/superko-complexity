//! The symmetries of a board, and the exchange of the two colors, as
//! permutations of points and of position codes.
//!
//! # The two maps
//!
//! A **board symmetry** is a map of the `m × n` grid onto itself taken from
//! the eight symmetries of the square: the identity, the reversal of the rows,
//! the reversal of the columns, the half-turn, and — only when `m = n` — the
//! transposition and the three maps it composes to (the other diagonal
//! reflection and the two quarter turns). On a given board several of the
//! eight can coincide as maps of the grid, and [`Symmetries::new`] keeps one
//! of each, so the group it holds has
//!
//! | board | order |
//! |---|---|
//! | 1×1 | 1 |
//! | 1×n and n×1, n ≥ 2 | 2: the identity and the reversal |
//! | m×n, m ≠ n, m, n ≥ 2 | 4: the identity, both reversals, the half-turn |
//! | n×n, n ≥ 2 | 8 |
//!
//! A symmetry moves a position by moving its stones: the image of a position
//! `b` under `s` has at `s(p)` what `b` has at `p`. The **color swap**
//! exchanges the color of every stone and moves none.
//!
//! # What a search may do with them, and its status
//!
//! Two facts would let a search skip work. First, the rules commute with every
//! board symmetry: the successor of `s(b)` under a play by `x` at `s(p)` is
//! `s` of the successor of `b` under `x` at `p`, the play is playable in one
//! exactly when in the other, and the two positions have the same area scores.
//! Second, they commute with the color swap: the successor of `swap(b)` under
//! `x.other` at `p` is `swap` of the successor of `b` under `x` at `p`,
//! playability likewise, and `x.other`'s area on `swap(b)` is `x`'s on `b`.
//!
//! Neither is stated in `Defs.lean`, and neither is proved here or in Lean.
//! Both are `computed`: `tests/symmetry.rs` checks them against the transition
//! table at every code, color and point on every board of at most six points,
//! and on 3×3 and 3×4 in an ignored release test. So they are registered as
//! the divergences `board-symmetry` and `color-swap` in [`crate::divergence`],
//! each `unlicensed`, and a result produced by a search that relies on one
//! names it in its `divergences=` line.
//!
//! # Why this module is in this crate
//!
//! `tools/check-mirror.sh` requires every slug in the divergence register to
//! have its `DIVERGENCE:` marker under `crates/superko-rules/src`, and scans
//! nowhere else, so the maps that carry the two markers live here. That is a
//! departure from `docs/plans/solver-plan.md` §4, which lists only the
//! divergence register and a pass-alive table as additions this crate may
//! take. The maps call the reference functions and nothing else — [`decode`],
//! a permutation of cells or an exchange of colors, [`encode`] — once per code
//! at construction, as [`crate::table::RuleTable::build`] does, so they are a
//! memo of a readable definition and not a second engine.
//!
//! Nothing here establishes anything about Go.

use crate::code::{PosCode, code_space, decode, encode};
use crate::config::{Dims, Point};
use crate::reference::Position;
use crate::table::{MAX_TABLE_POINTS, TooLarge};

/// One of the eight symmetries of the square, described by what it does to a
/// point `(row, col)`: reverse the rows, reverse the columns, and then, when
/// `transpose` is set, exchange the two coordinates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Transform {
    /// Exchange row and column, after the reversals. Only a square board has
    /// such a symmetry.
    pub transpose: bool,
    /// Send row `r` to row `m - 1 - r`.
    pub flip_rows: bool,
    /// Send column `c` to column `n - 1 - c`.
    pub flip_cols: bool,
}

impl Transform {
    /// The identity.
    pub const IDENTITY: Self = Self {
        transpose: false,
        flip_rows: false,
        flip_cols: false,
    };

    /// The image of a point of the board.
    ///
    /// # Panics
    ///
    /// Panics when the point is off the board, or when the transform
    /// transposes and the board is not square.
    #[must_use]
    pub const fn apply(self, dims: Dims, p: Point) -> Point {
        assert!(dims.contains(p), "point off the board");
        assert!(
            !self.transpose || dims.rows == dims.cols,
            "only a square board has a transposing symmetry"
        );
        let row = if self.flip_rows {
            dims.rows - 1 - p.row
        } else {
            p.row
        };
        let col = if self.flip_cols {
            dims.cols - 1 - p.col
        } else {
            p.col
        };
        if self.transpose {
            Point::new(col, row)
        } else {
            Point::new(row, col)
        }
    }

    /// A short name: `identity`, or the operations joined by `+`.
    #[must_use]
    pub fn name(self) -> String {
        let mut parts = Vec::new();
        if self.flip_rows {
            parts.push("flip-rows");
        }
        if self.flip_cols {
            parts.push("flip-cols");
        }
        if self.transpose {
            parts.push("transpose");
        }
        if parts.is_empty() {
            "identity".to_string()
        } else {
            parts.join("+")
        }
    }
}

/// The symmetry group of one board and the color swap, as lookup tables.
///
/// Elements are numbered from zero, and element 0 is the identity. The order
/// of the rest is fixed — the transform candidates are tried with `transpose`
/// outermost, then `flip_rows`, then `flip_cols`, false before true, and a
/// candidate is kept when its map of the grid differs from every one kept
/// before — so an element's number means the same thing on every run.
#[derive(Clone, Debug)]
pub struct Symmetries {
    dims: Dims,
    transforms: Vec<Transform>,
    /// `points[g · point_count + i]`: the row-major index of the image of point
    /// `i` under element `g`.
    points: Vec<usize>,
    /// The number of position codes of the board, `3^(m·n)`.
    code_count: usize,
    /// `codes[g · code_count + c]`: the code of the image of position `c` under
    /// element `g`.
    codes: Vec<u32>,
    /// `swap[c]`: the code of position `c` with every stone's color exchanged.
    swap: Vec<u32>,
    /// `compose[a][b]`: the element that applies `b` and then `a`.
    compose: Vec<Vec<usize>>,
    /// `inverse[g]`: the element that undoes `g`.
    inverse: Vec<usize>,
}

impl Symmetries {
    /// The symmetry group of a board, with every element's permutation of
    /// codes and the color swap's, built by decoding every code once.
    ///
    /// # Errors
    ///
    /// Refuses a board of more than [`MAX_TABLE_POINTS`] points, the bound the
    /// transition table has: the code maps are one `u32` per code per element.
    pub fn new(dims: Dims) -> Result<Self, TooLarge> {
        if dims.point_count() > MAX_TABLE_POINTS {
            return Err(TooLarge {
                dims,
                limit: MAX_TABLE_POINTS,
            });
        }
        let (transforms, points) = group_of(dims);
        let order = transforms.len();

        let compose: Vec<Vec<usize>> = (0..order)
            .map(|a| {
                (0..order)
                    .map(|b| {
                        let composed: Vec<usize> =
                            points[b].iter().map(|&i| points[a][i]).collect();
                        points
                            .iter()
                            .position(|perm| *perm == composed)
                            .expect("the symmetries of a board are closed under composition")
                    })
                    .collect()
            })
            .collect();
        let inverse: Vec<usize> = (0..order)
            .map(|g| {
                (0..order)
                    .find(|&h| compose[g][h] == 0)
                    .expect("every symmetry of a board has an inverse")
            })
            .collect();

        let code_count = code_space(dims) as usize;
        let mut codes = vec![0u32; order * code_count];
        let mut swap = vec![0u32; code_count];
        for raw in 0..code_count {
            let code = PosCode(u32::try_from(raw).expect("a code fits a u32"));
            let b = decode(dims, code);
            for (g, perm) in points.iter().enumerate() {
                codes[g * code_count + raw] = encode(&moved(&b, perm)).0;
            }
            swap[raw] = encode(&swapped(&b)).0;
        }

        Ok(Self {
            dims,
            transforms,
            points: points.concat(),
            code_count,
            codes,
            swap,
            compose,
            inverse,
        })
    }

    /// The board.
    #[must_use]
    pub const fn dims(&self) -> Dims {
        self.dims
    }

    /// The number of board symmetries, the identity included.
    #[must_use]
    pub fn order(&self) -> usize {
        self.transforms.len()
    }

    /// What element `g` does to a point.
    ///
    /// # Panics
    ///
    /// Panics when `g` is not an element.
    #[must_use]
    pub fn transform(&self, g: usize) -> Transform {
        self.transforms[g]
    }

    /// The image of a point under element `g`.
    ///
    /// # Panics
    ///
    /// Panics when `g` is not an element or the point is off the board.
    #[must_use]
    pub fn point(&self, g: usize, p: Point) -> Point {
        assert!(g < self.order(), "not an element");
        self.dims
            .point_at(self.points[g * self.dims.point_count() + self.dims.index(p)])
    }

    /// The image of a position code under element `g`.
    ///
    /// # Panics
    ///
    /// Panics when `g` is not an element or the code is not of this board.
    #[must_use]
    pub fn code(&self, g: usize, code: PosCode) -> PosCode {
        assert!(g < self.order(), "not an element");
        let c = code.0 as usize;
        assert!(c < self.code_count, "code is not a position of this board");
        PosCode(self.codes[g * self.code_count + c])
    }

    /// The number of position codes of the board, `3^(m·n)`: the stride of
    /// [`Symmetries::code_table`].
    #[must_use]
    pub const fn code_count(&self) -> usize {
        self.code_count
    }

    /// Every element's code map in one table: entry `g · code_count + c` is
    /// the raw code [`Symmetries::code`] returns for element `g` and code `c`.
    /// For a search that reads it in its inner loop without a call per entry.
    #[must_use]
    pub fn code_table(&self) -> &[u32] {
        &self.codes
    }

    /// Every element's point map in one table: entry `g · point_count + i` is
    /// the row-major index of the image of the point of row-major index `i`
    /// under element `g`, the point [`Symmetries::point`] returns.
    #[must_use]
    pub fn point_table(&self) -> &[usize] {
        &self.points
    }

    /// A position code with every stone's color exchanged.
    ///
    /// # Panics
    ///
    /// Panics when the code is not of this board.
    #[must_use]
    pub fn swap(&self, code: PosCode) -> PosCode {
        PosCode(self.swap[code.0 as usize])
    }

    /// The element that applies `b` and then `a`.
    ///
    /// # Panics
    ///
    /// Panics when either is not an element.
    #[must_use]
    pub fn compose(&self, a: usize, b: usize) -> usize {
        self.compose[a][b]
    }

    /// The element that undoes `g`.
    ///
    /// # Panics
    ///
    /// Panics when `g` is not an element.
    #[must_use]
    pub fn inverse(&self, g: usize) -> usize {
        self.inverse[g]
    }
}

/// The distinct maps of the grid among the eight symmetries of the square,
/// the identity first, each with its permutation of row-major indices.
fn group_of(dims: Dims) -> (Vec<Transform>, Vec<Vec<usize>>) {
    let square = dims.rows == dims.cols;
    let mut transforms = Vec::new();
    let mut points: Vec<Vec<usize>> = Vec::new();
    for transpose in [false, true] {
        if transpose && !square {
            continue;
        }
        for flip_rows in [false, true] {
            for flip_cols in [false, true] {
                let t = Transform {
                    transpose,
                    flip_rows,
                    flip_cols,
                };
                let perm: Vec<usize> = dims
                    .points()
                    .map(|p| dims.index(t.apply(dims, p)))
                    .collect();
                if !points.contains(&perm) {
                    transforms.push(t);
                    points.push(perm);
                }
            }
        }
    }
    (transforms, points)
}

/// The position with each stone moved from point `i` to point `perm[i]`.
// DIVERGENCE: board-symmetry
fn moved(b: &Position, perm: &[usize]) -> Position {
    let dims = b.dims();
    let mut cells = vec![None; dims.point_count()];
    for (i, cell) in b.cells().iter().enumerate() {
        cells[perm[i]] = *cell;
    }
    Position::from_cells(dims, &cells)
}

/// The position with every stone's color exchanged.
// DIVERGENCE: color-swap
fn swapped(b: &Position) -> Position {
    let cells: Vec<_> = b
        .cells()
        .iter()
        .map(|cell| cell.map(|c| c.other()))
        .collect();
    Position::from_cells(b.dims(), &cells)
}
