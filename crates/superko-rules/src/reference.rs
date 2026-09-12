//! The transliteration of `lean/SuperkoComplexity/Defs.lean`.
//!
//! One item per `Defs.lean` item, in that file's order, each carrying a
//! `**Mirrors**` doc line that names the `Superko` declaration it mirrors and
//! that [`tools/check-mirror.sh`](../../../../tools/check-mirror.sh) greps.
//! Two items come instead from `Decide.lean`, the computable layer, and say so
//! in the same line.
//!
//! This module is written for a reader holding the Lean open, not for speed.
//! Sets are `BTreeSet`, chains are flood fills, `clear` sweeps the whole board
//! and allocates a fresh position. The fast path is [`crate::table`], whose
//! only constructor calls the functions here; there is no second engine.
//!
//! Three items have no counterpart in `Defs.lean`. Two are registered in
//! [`crate::divergence`] and carry a marker: [`resolve_remove_own`]
//! (`suicide-remove-own`) and [`winner_z`] (`winner-via-floor-komi`). The
//! third, [`legality`], is not a divergence and has no slug: it decomposes the
//! conjunction [`permits`] already computes, and the contract
//! `legality(..).is_ok() == permits(..)` is asserted on every state the walk in
//! `tests/rules_walk.rs` reaches, so it changes no verdict. Two more items,
//! [`Dims`] and [`Point`], live in [`crate::config`] because every item here
//! takes them.
//!
//! Nothing in this module establishes anything about Go. `Defs.lean` says what
//! the rules are; this file is a second, untrusted reading of it, and the only
//! evidence that the two readings agree is the test suite.

use core::cmp::Ordering;
use core::fmt;
use std::collections::BTreeSet;

pub use crate::config::{Dims, Point, Repetition, Suicide};

/// The two colors of stone.
///
/// **Mirrors** `Superko.Color`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Color {
    /// Black, who moves first from the root of play.
    Black,
    /// White.
    White,
}

impl Color {
    /// The opposing color.
    ///
    /// **Mirrors** `Superko.Color.other`.
    #[must_use]
    pub const fn other(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }

    /// The character this color takes in a position's string form.
    #[must_use]
    pub const fn glyph(self) -> char {
        match self {
            Self::Black => 'X',
            Self::White => 'O',
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Black => "black",
            Self::White => "white",
        })
    }
}

// `Point` — **Mirrors** `Superko.Point`. It is declared in `crate::config`,
// next to `Dims`, because the board geometry is what every item below takes.

/// Two points are **adjacent** when they agree on one coordinate and differ by
/// one on the other: the four orthogonal neighbors of the Go board. Diagonals
/// are not adjacency.
///
/// **Mirrors** `Superko.Adj`.
#[must_use]
pub fn adj(p: Point, q: Point) -> bool {
    (p.row == q.row && (p.col + 1 == q.col || q.col + 1 == p.col))
        || (p.col == q.col && (p.row + 1 == q.row || q.row + 1 == p.row))
}

/// The points adjacent to `p` that lie on the board, in row-major order.
///
/// A derived convenience, not a `Defs.lean` item: Lean quantifies over all
/// points and tests `Adj`, and the test `adj_agrees_with_neighbors` checks
/// that this enumeration is that quantification.
#[must_use]
pub fn neighbors(dims: Dims, p: Point) -> Vec<Point> {
    assert!(dims.contains(p), "point off the board");
    let mut out = Vec::with_capacity(4);
    if p.row > 0 {
        out.push(Point::new(p.row - 1, p.col));
    }
    if p.col > 0 {
        out.push(Point::new(p.row, p.col - 1));
    }
    if p.col + 1 < dims.cols {
        out.push(Point::new(p.row, p.col + 1));
    }
    if p.row + 1 < dims.rows {
        out.push(Point::new(p.row + 1, p.col));
    }
    out
}

/// A **position** gives each point of the board a stone or leaves it empty.
///
/// **Mirrors** `Superko.Position`.
///
/// Lean's `Position m n` is a function `Point m n → Option Color`, and two
/// positions are equal when they agree at every point. Equality here is that
/// agreement, on a board that is part of the value: `dims` is carried because
/// Lean's `m` and `n` are runtime values in this crate, and two positions of
/// *different* boards are unequal rather than compared cell by cell. Lean has
/// no such comparison at all — `Position 1 4` and `Position 2 2` are different
/// types — and the four empty cells of the 1×4 board are not the four empty
/// cells of the 2×2 board, whose game count differs.
// DIVERGENCE: dims-are-runtime
#[derive(Clone, Debug)]
pub struct Position {
    dims: Dims,
    cells: Box<[Option<Color>]>,
}

impl Position {
    /// The empty board.
    #[must_use]
    pub fn empty(dims: Dims) -> Self {
        Self {
            dims,
            cells: vec![None; dims.point_count()].into_boxed_slice(),
        }
    }

    /// A position from its cells, row-major.
    ///
    /// # Panics
    ///
    /// Panics when the cell count is not `dims.point_count()`.
    #[must_use]
    pub fn from_cells(dims: Dims, cells: &[Option<Color>]) -> Self {
        assert_eq!(
            cells.len(),
            dims.point_count(),
            "cell count does not match the board"
        );
        Self {
            dims,
            cells: cells.to_vec().into_boxed_slice(),
        }
    }

    /// The board this position is on.
    #[must_use]
    pub const fn dims(&self) -> Dims {
        self.dims
    }

    /// The cells, row-major.
    #[must_use]
    pub fn cells(&self) -> &[Option<Color>] {
        &self.cells
    }

    /// What stands at `p`.
    ///
    /// # Panics
    ///
    /// Panics when `p` is off the board.
    #[must_use]
    pub fn get(&self, p: Point) -> Option<Color> {
        self.cells[self.dims.index(p)]
    }

    /// Put `v` at `p`, in place.
    ///
    /// # Panics
    ///
    /// Panics when `p` is off the board.
    pub fn set(&mut self, p: Point, v: Option<Color>) {
        let i = self.dims.index(p);
        self.cells[i] = v;
    }

    /// A fresh position agreeing with this one except at `p`.
    ///
    /// **Mirrors** `Function.update`, which is what `Superko.resolve` applies
    /// to place the played stone.
    #[must_use]
    pub fn update(&self, p: Point, v: Option<Color>) -> Self {
        let mut out = self.clone();
        out.set(p, v);
        out
    }

    /// How many stones of `c` stand on the board.
    #[must_use]
    pub fn stone_count(&self, c: Color) -> usize {
        self.cells.iter().filter(|v| **v == Some(c)).count()
    }
}

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        self.dims == other.dims && self.cells == other.cells
    }
}

impl Eq for Position {}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> Ordering {
        self.dims
            .cmp(&other.dims)
            .then_with(|| self.cells.cmp(&other.cells))
    }
}

/// Adjacent points carrying stones of the same color — the relation whose
/// connected components are chains.
///
/// **Mirrors** `Superko.Joined`.
#[must_use]
pub fn joined(b: &Position, p: Point, q: Point) -> bool {
    adj(p, q) && b.get(p) == b.get(q) && b.get(p).is_some()
}

/// The **chain** containing `p`: the stones connected to `p` through
/// same-colored neighbors.
///
/// **Mirrors** `Superko.chain`.
///
/// Lean takes the reflexive-transitive closure of `Joined`, so an empty `p`
/// gives the singleton `{p}` — the relation is empty there, and no rule
/// consults that case.
#[must_use]
pub fn chain(b: &Position, p: Point) -> BTreeSet<Point> {
    let mut seen = BTreeSet::new();
    seen.insert(p);
    let mut stack = vec![p];
    while let Some(q) = stack.pop() {
        for r in neighbors(b.dims(), q) {
            if joined(b, q, r) && seen.insert(r) {
                stack.push(r);
            }
        }
    }
    seen
}

/// A chain has a **liberty** when some point adjacent to some stone of the
/// chain is empty. A chain without a liberty is captured.
///
/// **Mirrors** `Superko.HasLiberty`.
#[must_use]
pub fn has_liberty(b: &Position, p: Point) -> bool {
    chain(b, p).into_iter().any(|q| {
        neighbors(b.dims(), q)
            .into_iter()
            .any(|r| b.get(r).is_none())
    })
}

/// Remove every `c` stone whose chain has no liberty.
///
/// **Mirrors** `Superko.clear`.
///
/// Lean's `clear` is a function of the original board at every point, so the
/// removals do not see each other and the sweep order cannot matter
/// (`clear_is_order_independent`). This returns a fresh position for the same
/// reason.
///
/// Removing in place while sweeping is a different function, and not only in
/// principle: on the 2x2 board of four black stones it leaves three of them
/// standing, because the first removal gives the rest a liberty. That the two
/// agree is `refuted`, with `clear_reads_the_original_board` as the witness.
#[must_use]
pub fn clear(b: &Position, c: Color) -> Position {
    let mut out = b.clone();
    for q in b.dims().points() {
        if b.get(q) == Some(c) && !has_liberty(b, q) {
            out.set(q, None);
        }
    }
    out
}

/// The position after `c` places a stone at `p` and every opposing chain left
/// without a liberty is removed.
///
/// **Mirrors** `Superko.resolve`.
///
/// Capture is resolved for the opponent only. A move that leaves the played
/// stone's own chain without a liberty is suicide, and [`playable_at`] forbids
/// it rather than resolving it.
#[must_use]
pub fn resolve(b: &Position, c: Color, p: Point) -> Position {
    clear(&b.update(p, Some(c)), c.other())
}

/// The position after `c` plays at `p` under the suicide-permitting reading:
/// the opponent's dead chains come off first, then the player's own.
///
/// This has **no counterpart in `Defs.lean`**. A number produced under
/// [`Suicide::RemoveOwn`] is a number about this crate and not about the audit
/// target; the divergence is unlicensed, and
/// [`crate::divergence::Divergence::SuicideRemoveOwn`] is what a witness
/// header prints to say so.
// DIVERGENCE: suicide-remove-own
#[must_use]
pub fn resolve_remove_own(b: &Position, c: Color, p: Point) -> Position {
    clear(&clear(&b.update(p, Some(c)), c.other()), c)
}

/// The position after `c` plays at `p` under the chosen suicide convention.
///
/// Under [`Suicide::Forbid`] this is exactly `Superko.resolve`.
#[must_use]
pub fn resolve_under(b: &Position, c: Color, p: Point, suicide: Suicide) -> Position {
    match suicide {
        Suicide::Forbid => resolve(b, c, p),
        // DIVERGENCE: suicide-remove-own
        Suicide::RemoveOwn => resolve_remove_own(b, c, p),
    }
}

/// A **situation**: a position together with the player to move.
///
/// **Mirrors** `Superko.Situation`.
///
/// The repetition rules are stated over situations because that is what
/// distinguishes situational from positional superko — SSK forbids recreating
/// a situation, PSK a position.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Situation {
    /// The stones on the board.
    pub board: Position,
    /// Whose turn it is.
    pub to_move: Color,
}

/// A **move** is a pass or a play at a point.
///
/// **Mirrors** `Superko.Move`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Move {
    /// A pass.
    Pass,
    /// A play at a point.
    Play(Point),
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pass => f.write_str("pass"),
            Self::Play(p) => write!(f, "play {p}"),
        }
    }
}

/// The situation a move leads to. Legality is a separate question; this is the
/// board mechanics alone.
///
/// **Mirrors** `Superko.Situation.after`.
///
/// Lean's `Situation.after` takes no suicide parameter, because `Defs.lean`
/// has one convention. Passing [`Suicide::Forbid`] is the literal reading.
#[must_use]
pub fn situation_after(s: &Situation, mv: Move, suicide: Suicide) -> Situation {
    match mv {
        Move::Pass => Situation {
            board: s.board.clone(),
            to_move: s.to_move.other(),
        },
        Move::Play(p) => Situation {
            board: resolve_under(&s.board, s.to_move, p, suicide),
            to_move: s.to_move.other(),
        },
    }
}

/// A **state of play**: the situation now, every situation that has occurred,
/// and how many passes immediately precede.
///
/// **Mirrors** `Superko.State`.
///
/// The fields are private and [`State::start`] is the only constructor. There
/// is deliberately no second way to build a state: the root seed is
/// observable — on 1×1 under [`Suicide::RemoveOwn`] and positional superko it
/// is what holds the game count at 1 — and two construction paths that
/// disagreed about it would be invisible in every count they produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct State {
    now: Situation,
    seen: BTreeSet<Situation>,
    passes: u32,
}

impl State {
    /// The state in which play begins from a given position.
    ///
    /// **Mirrors** `Superko.start`.
    ///
    /// This is encoding (C) of `docs/formal-model.md` §5: the position is the
    /// root of play, nothing is forbidden yet, and the root situation is
    /// already in the archive.
    #[must_use]
    pub fn start(b: &Position, c: Color) -> Self {
        let now = Situation {
            board: b.clone(),
            to_move: c,
        };
        let mut seen = BTreeSet::new();
        seen.insert(now.clone());
        Self {
            now,
            seen,
            passes: 0,
        }
    }

    /// The situation now.
    #[must_use]
    pub const fn now(&self) -> &Situation {
        &self.now
    }

    /// Every situation that has occurred, `now` included.
    #[must_use]
    pub const fn seen(&self) -> &BTreeSet<Situation> {
        &self.seen
    }

    /// How many passes immediately precede.
    #[must_use]
    pub const fn passes(&self) -> u32 {
        self.passes
    }
}

/// The state after a move. Total: applying it to an illegal move gives a
/// state, which no rule below reaches.
///
/// **Mirrors** `Superko.step`.
///
/// The successor situation is inserted on every move, passes included. The
/// pass count is an ordinary count and is not clamped at 2: `Superko.step`
/// writes `st.passes + 1`, and [`ended`] reads `2 ≤ passes`.
///
/// # Panics
///
/// Panics when the pass count overflows, which cannot happen within the
/// archive bound but is not silently wrapped.
#[must_use]
pub fn step(st: &State, mv: Move, suicide: Suicide) -> State {
    let now = situation_after(&st.now, mv, suicide);
    let mut seen = st.seen.clone();
    seen.insert(now.clone());
    let passes = match mv {
        Move::Pass => st.passes.checked_add(1).expect("pass count overflowed"),
        Move::Play(_) => 0,
    };
    State { now, seen, passes }
}

/// The game has **ended**: two consecutive passes.
///
/// **Mirrors** `Superko.Ended`.
#[must_use]
pub const fn ended(st: &State) -> bool {
    st.passes >= 2
}

// `Repetition` — Lean's `Superko.Repetition` is the *type* of a repetition
// rule, `State m n → Move m n → Prop`, and it appears as a parameter of
// `WinsFor`. The crate's `config::Repetition` names one of the two instances
// instead; the type-level item belongs with the solver.

/// The board conditions every play must meet, whatever the repetition rule:
/// the point is empty, and the resulting chain has a liberty. The second
/// conjunct is the prohibition on suicide.
///
/// **Mirrors** `Superko.PlayableAt`.
///
/// Lean's `PlayableAt` reads the board and the player to move out of a
/// `State`; this takes them directly, as `Basic.lean`'s computable
/// `PlayableAt'` does. Under [`Suicide::RemoveOwn`] the second conjunct is
/// dropped and the emptiness test stands alone.
#[must_use]
pub fn playable_at(b: &Position, c: Color, p: Point, suicide: Suicide) -> bool {
    match suicide {
        Suicide::Forbid => b.get(p).is_none() && has_liberty(&resolve(b, c, p), p),
        // DIVERGENCE: suicide-remove-own
        Suicide::RemoveOwn => b.get(p).is_none(),
    }
}

/// **Situational superko** — AGA Rule 6. A play may not recreate a situation
/// that has occurred: a position with the same player to move.
///
/// **Mirrors** `Superko.SSK`.
///
/// A pass is always legal. That reading is OPEN-1 in `docs/formal-model.md`
/// and claim C-18, `cited`; it is what lets a pass re-enter a situation no
/// play could reach.
#[must_use]
pub fn ssk(st: &State, mv: Move, suicide: Suicide) -> bool {
    match mv {
        Move::Pass => true,
        Move::Play(p) => {
            playable_at(&st.now().board, st.now().to_move, p, suicide)
                && !st.seen().contains(&situation_after(st.now(), mv, suicide))
        }
    }
}

/// **Positional superko**. A play may not recreate a position that has
/// occurred, whoever was to move.
///
/// **Mirrors** `Superko.PSK`.
#[must_use]
pub fn psk(st: &State, mv: Move, suicide: Suicide) -> bool {
    match mv {
        Move::Pass => true,
        Move::Play(p) => {
            let after = situation_after(st.now(), mv, suicide);
            playable_at(&st.now().board, st.now().to_move, p, suicide)
                && st.seen().iter().all(|s| s.board != after.board)
        }
    }
}

/// Whether `mv` is legal in `st` under the named repetition rule.
///
/// This is the dispatch `Superko.WinsFor` performs through its `L` parameter,
/// and nothing more; the rules themselves are [`ssk`] and [`psk`].
#[must_use]
pub fn permits(st: &State, mv: Move, rep: Repetition, suicide: Suicide) -> bool {
    match rep {
        Repetition::Ssk => ssk(st, mv, suicide),
        Repetition::Psk => psk(st, mv, suicide),
    }
}

/// Why a move was refused.
///
/// Legality is a conjunction, so a count reproduced by accident — two
/// conventions swapped and cancelling — looks exactly like a count reproduced
/// correctly. The decomposed verdict exists to catch that.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Refusal {
    /// The point already carries a stone.
    Occupied,
    /// The play would leave its own chain without a liberty, and the
    /// convention in force forbids that.
    Suicide,
    /// The play would recreate a situation (SSK) or a position (PSK) that has
    /// occurred.
    Repetition,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Occupied => "occupied",
            Self::Suicide => "suicide",
            Self::Repetition => "repetition",
        })
    }
}

/// The diagnostic form of legality: which conjunct refused the move, in the
/// fixed priority `Occupied` before `Suicide` before `Repetition`.
///
/// This has no counterpart in `Defs.lean`, which states legality as a
/// conjunction and never asks which half failed. The contract is
/// `legality(..).is_ok() == permits(..)`, and the test `legality_agrees` is
/// the only reason to believe it.
///
/// # Errors
///
/// Returns the first refusal in priority order. A pass is never refused.
pub fn legality(st: &State, mv: Move, rep: Repetition, suicide: Suicide) -> Result<(), Refusal> {
    let Move::Play(p) = mv else { return Ok(()) };
    let b = &st.now().board;
    let c = st.now().to_move;
    if b.get(p).is_some() {
        return Err(Refusal::Occupied);
    }
    // DIVERGENCE: suicide-remove-own
    if matches!(suicide, Suicide::Forbid) && !has_liberty(&resolve(b, c, p), p) {
        return Err(Refusal::Suicide);
    }
    let after = situation_after(st.now(), mv, suicide);
    let repeats = match rep {
        Repetition::Ssk => st.seen().contains(&after),
        Repetition::Psk => st.seen().iter().any(|s| s.board == after.board),
    };
    if repeats {
        return Err(Refusal::Repetition);
    }
    Ok(())
}

/// An empty point **reaches** color `c` when a path of adjacent empty points
/// leads from it to a `c` stone.
///
/// **Mirrors** `Superko.Reaches`.
///
/// Lean's relation is the reflexive-transitive closure of "adjacent, and the
/// *source* is empty", so the walk may step out of the empty region exactly
/// once, onto the stone it reaches.
#[must_use]
pub fn reaches(b: &Position, p: Point, c: Color) -> bool {
    let mut seen = BTreeSet::new();
    seen.insert(p);
    let mut stack = vec![p];
    while let Some(x) = stack.pop() {
        if b.get(x) == Some(c) {
            return true;
        }
        if b.get(x).is_none() {
            for y in neighbors(b.dims(), x) {
                if seen.insert(y) {
                    stack.push(y);
                }
            }
        }
    }
    false
}

/// **Area score**: stones of the color, plus empty points reaching that color
/// and not the other.
///
/// **Mirrors** `Superko.area`.
///
/// This is mechanical scoring in the Tromp–Taylor formulation. There is no
/// dead-stone determination; that the mechanical score coincides with the
/// agreed score under optimal play is claim C-16, which is `open`.
#[must_use]
pub fn area(b: &Position, c: Color) -> usize {
    b.dims()
        .points()
        .filter(|&p| {
            b.get(p) == Some(c)
                || (b.get(p).is_none() && reaches(b, p, c) && !reaches(b, p, c.other()))
        })
        .count()
}

/// An exact rational, `num / den` in lowest terms with `den > 0`.
///
/// Komi is a rational in `Defs.lean` (`ℚ`), and `Superko.margin` is a
/// subtraction in ℚ. Floating point would make the tie case — which
/// `Superko.winner` gives to White — depend on rounding, so this crate has no
/// floating point in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rat {
    num: i64,
    den: i64,
}

impl Rat {
    /// Zero.
    pub const ZERO: Self = Self { num: 0, den: 1 };

    /// The rational `num / den`, in lowest terms.
    ///
    /// # Panics
    ///
    /// Panics when `den` is zero, or when normalization overflows `i64`.
    #[must_use]
    pub const fn new(num: i64, den: i64) -> Self {
        assert!(den != 0, "a rational needs a nonzero denominator");
        let (mut num, mut den) = if den < 0 {
            (
                num.checked_neg().expect("rational overflowed"),
                den.checked_neg().expect("rational overflowed"),
            )
        } else {
            (num, den)
        };
        let g = gcd(num.unsigned_abs(), den.unsigned_abs());
        if g > 1 {
            num /= g as i64;
            den /= g as i64;
        }
        Self { num, den }
    }

    /// The rational `n / 1`.
    #[must_use]
    pub const fn from_int(n: i64) -> Self {
        Self { num: n, den: 1 }
    }

    /// The numerator, in lowest terms.
    #[must_use]
    pub const fn num(self) -> i64 {
        self.num
    }

    /// The denominator, in lowest terms; always positive.
    #[must_use]
    pub const fn den(self) -> i64 {
        self.den
    }

    /// Whether the rational is strictly positive — the test
    /// `Superko.winner` makes on the margin.
    #[must_use]
    pub const fn is_positive(self) -> bool {
        self.num > 0
    }

    /// The greatest integer at most this rational, Lean's `Int.floor`.
    #[must_use]
    pub const fn floor(self) -> i64 {
        self.num.div_euclid(self.den)
    }
}

const fn gcd(a: u64, b: u64) -> u64 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    if a == 0 { 1 } else { a }
}

const fn gcd_i128(a: u128, b: u128) -> u128 {
    let (mut a, mut b) = (a, b);
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    if a == 0 { 1 } else { a }
}

impl core::ops::Sub for Rat {
    type Output = Self;

    /// The difference `self - other`.
    ///
    /// # Panics
    ///
    /// Panics on overflow rather than wrapping.
    fn sub(self, other: Self) -> Self {
        let num = i128::from(self.num) * i128::from(other.den)
            - i128::from(other.num) * i128::from(self.den);
        let den = i128::from(self.den) * i128::from(other.den);
        let g = gcd_i128(num.unsigned_abs(), den.unsigned_abs());
        let (num, den) = if g > 1 {
            (num / g as i128, den / g as i128)
        } else {
            (num, den)
        };
        Self {
            num: i64::try_from(num).expect("rational overflowed"),
            den: i64::try_from(den).expect("rational overflowed"),
        }
    }
}

impl PartialOrd for Rat {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Rat {
    fn cmp(&self, other: &Self) -> Ordering {
        (i128::from(self.num) * i128::from(other.den))
            .cmp(&(i128::from(other.num) * i128::from(self.den)))
    }
}

impl fmt::Display for Rat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.den == 1 {
            write!(f, "{}", self.num)
        } else {
            write!(f, "{}/{}", self.num, self.den)
        }
    }
}

/// Black's margin: Black's area less White's, less komi.
///
/// **Mirrors** `Superko.margin`.
///
/// # Panics
///
/// Panics when the area counts do not fit an `i64`, which no board this crate
/// admits can produce.
#[must_use]
pub fn margin(b: &Position, komi: Rat) -> Rat {
    let black = i64::try_from(area(b, Color::Black)).expect("area overflowed");
    let white = i64::try_from(area(b, Color::White)).expect("area overflowed");
    Rat::from_int(black) - Rat::from_int(white) - komi
}

/// The winner at a finished game. A tie goes to White.
///
/// **Mirrors** `Superko.winner`.
///
/// With half-integer komi no tie arises and the convention is invisible
/// (OPEN-3 in `docs/formal-model.md`). It is observable where a color swap is
/// attempted, so nothing here may call it unobservable without that
/// qualification.
#[must_use]
pub fn winner(b: &Position, komi: Rat) -> Color {
    if margin(b, komi).is_positive() {
        Color::Black
    } else {
        Color::White
    }
}

/// Who won, decided over ℤ: Black wins when `⌊komi⌋` is below the difference
/// of the two area scores, and a tie goes to White as in [`winner`].
///
/// **Mirrors (Decide.lean)** `Superko.winnerZ`.
///
/// That the integer test at `⌊komi⌋` computes [`winner`] at `komi` is
/// `Superko.winnerZ_eq_winner`, `proved`; this crate's use of it is registered
/// as [`crate::divergence::Divergence::WinnerViaFloorKomi`].
///
/// # Panics
///
/// Panics when the area counts do not fit an `i64`.
// DIVERGENCE: winner-via-floor-komi
#[must_use]
pub fn winner_z(b: &Position, komi_floor: i64) -> Color {
    let black = i64::try_from(area(b, Color::Black)).expect("area overflowed");
    let white = i64::try_from(area(b, Color::White)).expect("area overflowed");
    if komi_floor < black - white {
        Color::Black
    } else {
        Color::White
    }
}

/// Every move available on the board, legal or not: the pass first, then a
/// play at each point in row-major order.
///
/// **Mirrors (Decide.lean)** `Superko.allMoves`.
///
/// Every count this workspace produces depends on this order, so the test
/// `all_moves_order_is_pinned` fixes it against a literal list rather than
/// against a second computation of it.
#[must_use]
pub fn all_moves(dims: Dims) -> Vec<Move> {
    let mut out = Vec::with_capacity(dims.point_count() + 1);
    out.push(Move::Pass);
    out.extend(dims.points().map(Move::Play));
    out
}
