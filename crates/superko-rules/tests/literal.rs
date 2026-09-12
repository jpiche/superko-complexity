//! A second, deliberately literal reading of `Defs.lean`, checked exhaustively
//! against [`superko_rules::reference`].
//!
//! `reference` takes one shortcut that `Defs.lean` does not: where Lean writes
//! `∃ r, Adj q r ∧ ...` — a quantifier over every point of the board — the
//! crate enumerates the at most four on-board neighbors. `chain`, `has_liberty`
//! and `reaches` all rest on it, and `clear`, `resolve`, `playable_at` and
//! `area` all rest on those. The shortcut is sound only if the enumeration is
//! that quantification at every point of every board.
//!
//! The functions below take the quantifier literally: every closure is grown by
//! scanning all `m · n` points, and `Adj` is retyped from the Lean rather than
//! called. They are quadratic and are meant to be. Agreement with `reference`
//! is `computed` for the boards named in each test and for no others.

use std::collections::BTreeSet;

use superko_rules::PosCode;
use superko_rules::code::{code_space, decode, render};
use superko_rules::config::{Dims, Point, Suicide};
use superko_rules::reference::{
    Color, Position, adj, area, chain, clear, has_liberty, joined, neighbors, playable_at, reaches,
    resolve, resolve_remove_own, resolve_under,
};

// --- The literal reading ----------------------------------------------------

/// `Superko.Adj`, retyped from the Lean:
/// `(p.1 = q.1 ∧ (p.2+1 = q.2 ∨ q.2+1 = p.2)) ∨ (p.2 = q.2 ∧ (p.1+1 = q.1 ∨ q.1+1 = p.1))`,
/// with `p.1` the row and `p.2` the column.
fn adj_literal(p: Point, q: Point) -> bool {
    (p.row == q.row && (p.col + 1 == q.col || q.col + 1 == p.col))
        || (p.col == q.col && (p.row + 1 == q.row || q.row + 1 == p.row))
}

/// `Superko.Joined`.
fn joined_literal(b: &Position, p: Point, q: Point) -> bool {
    adj_literal(p, q) && b.get(p) == b.get(q) && b.get(p).is_some()
}

/// `Superko.chain`: the reflexive-transitive closure of `Joined b` from `p`,
/// grown by scanning every point rather than by walking neighbors.
fn chain_literal(b: &Position, p: Point) -> BTreeSet<Point> {
    let all: Vec<Point> = b.dims().points().collect();
    let mut set = BTreeSet::new();
    set.insert(p);
    loop {
        let mut grew = false;
        for q in set.iter().copied().collect::<Vec<Point>>() {
            for &r in &all {
                if joined_literal(b, q, r) && set.insert(r) {
                    grew = true;
                }
            }
        }
        if !grew {
            return set;
        }
    }
}

/// `Superko.HasLiberty`: `∃ q ∈ chain b p, ∃ r, Adj q r ∧ b r = none`, with the
/// inner quantifier over every point of the board.
fn has_liberty_literal(b: &Position, p: Point) -> bool {
    let all: Vec<Point> = b.dims().points().collect();
    chain_literal(b, p)
        .into_iter()
        .any(|q| all.iter().any(|&r| adj_literal(q, r) && b.get(r).is_none()))
}

/// `Superko.clear`, as the pointwise function Lean writes.
fn clear_literal(b: &Position, c: Color) -> Position {
    let cells: Vec<Option<Color>> = b
        .dims()
        .points()
        .map(|q| {
            if b.get(q) == Some(c) && !has_liberty_literal(b, q) {
                None
            } else {
                b.get(q)
            }
        })
        .collect();
    Position::from_cells(b.dims(), &cells)
}

/// `Superko.resolve`.
fn resolve_literal(b: &Position, c: Color, p: Point) -> Position {
    clear_literal(&b.update(p, Some(c)), c.other())
}

/// The crate's `resolve_remove_own`, which has no `Defs.lean` counterpart, in
/// the same literal style.
fn resolve_remove_own_literal(b: &Position, c: Color, p: Point) -> Position {
    clear_literal(&clear_literal(&b.update(p, Some(c)), c.other()), c)
}

/// `Superko.PlayableAt`, with the board and the mover passed directly.
fn playable_at_literal(b: &Position, c: Color, p: Point) -> bool {
    b.get(p).is_none() && has_liberty_literal(&resolve_literal(b, c, p), p)
}

/// `Superko.Reaches`: the closure of `fun x y => Adj x y ∧ b x = none` from `p`,
/// then a search for a `c` stone in it. The source of a step must be empty and
/// the target need not be, so the walk leaves the empty region exactly once.
fn reaches_literal(b: &Position, p: Point, c: Color) -> bool {
    let all: Vec<Point> = b.dims().points().collect();
    let mut set = BTreeSet::new();
    set.insert(p);
    loop {
        let mut grew = false;
        for x in set.iter().copied().collect::<Vec<Point>>() {
            if b.get(x).is_some() {
                continue;
            }
            for &y in &all {
                if adj_literal(x, y) && set.insert(y) {
                    grew = true;
                }
            }
        }
        if !grew {
            break;
        }
    }
    set.into_iter().any(|q| b.get(q) == Some(c))
}

/// `Superko.area`.
fn area_literal(b: &Position, c: Color) -> usize {
    b.dims()
        .points()
        .filter(|&p| {
            b.get(p) == Some(c)
                || (b.get(p).is_none()
                    && reaches_literal(b, p, c)
                    && !reaches_literal(b, p, c.other()))
        })
        .count()
}

// --- The comparison ---------------------------------------------------------

/// Every board with `m · n <= 6`, the same set `table_is_its_generator` covers.
fn small_boards() -> Vec<Dims> {
    let mut out = Vec::new();
    for rows in 1..=6 {
        for cols in 1..=6 {
            if rows * cols <= 6 {
                out.push(Dims::new(rows, cols));
            }
        }
    }
    out
}

/// `Adj` is the same relation both ways, and it is irreflexive and symmetric on
/// every board this compares over.
#[test]
fn adj_is_the_literal_relation() {
    for dims in small_boards() {
        for p in dims.points() {
            for q in dims.points() {
                assert_eq!(adj(p, q), adj_literal(p, q), "adj at {p} {q} on {dims}");
                assert_eq!(adj(p, q), adj(q, p), "adj is symmetric at {p} {q}");
            }
            assert!(!adj(p, p), "adj is irreflexive at {p}");
        }
    }
}

/// The neighbor enumeration is the quantification `∃ r, Adj q r ∧ ...` runs
/// over: on every board, the neighbors of a point are exactly the on-board
/// points `Adj` relates it to, with no repetition.
#[test]
fn neighbors_is_the_quantifier() {
    for dims in small_boards() {
        for p in dims.points() {
            let listed = neighbors(dims, p);
            let quantified: Vec<Point> = dims.points().filter(|&q| adj_literal(p, q)).collect();
            let as_set: BTreeSet<Point> = listed.iter().copied().collect();
            assert_eq!(as_set.len(), listed.len(), "a neighbor is listed twice");
            assert_eq!(
                as_set,
                quantified.into_iter().collect::<BTreeSet<Point>>(),
                "neighbors of {p} on {dims}"
            );
        }
    }
}

/// `joined`, `chain` and `has_liberty` agree with the literal closures at every
/// position, and a chain at an empty point is the singleton.
#[test]
fn chains_and_liberties_are_literal() {
    for dims in small_boards() {
        for code in 0..code_space(dims) {
            let b = decode(dims, PosCode(code));
            for p in dims.points() {
                for q in dims.points() {
                    assert_eq!(
                        joined(&b, p, q),
                        joined_literal(&b, p, q),
                        "joined at {p} {q} on {}",
                        render(&b)
                    );
                }
                let got = chain(&b, p);
                assert_eq!(got, chain_literal(&b, p), "chain at {p} on {}", render(&b));
                if b.get(p).is_none() {
                    assert_eq!(
                        got,
                        [p].into_iter().collect::<BTreeSet<Point>>(),
                        "the chain at an empty point is the singleton, at {p} on {}",
                        render(&b)
                    );
                }
                assert_eq!(
                    has_liberty(&b, p),
                    has_liberty_literal(&b, p),
                    "has_liberty at {p} on {}",
                    render(&b)
                );
            }
        }
    }
}

/// `clear`, `resolve`, `resolve_remove_own` and `playable_at` agree with the
/// literal reading at every position, color and point — occupied points
/// included, because `Superko.resolve` is total.
#[test]
fn clear_resolve_and_playable_are_literal() {
    for dims in small_boards() {
        for code in 0..code_space(dims) {
            let b = decode(dims, PosCode(code));
            for c in [Color::Black, Color::White] {
                assert_eq!(
                    clear(&b, c),
                    clear_literal(&b, c),
                    "clear {c} on {}",
                    render(&b)
                );
                for p in dims.points() {
                    assert_eq!(
                        resolve(&b, c, p),
                        resolve_literal(&b, c, p),
                        "resolve {c} at {p} on {}",
                        render(&b)
                    );
                    assert_eq!(
                        resolve_remove_own(&b, c, p),
                        resolve_remove_own_literal(&b, c, p),
                        "resolve_remove_own {c} at {p} on {}",
                        render(&b)
                    );
                    assert_eq!(
                        resolve_under(&b, c, p, Suicide::Forbid),
                        resolve(&b, c, p),
                        "Forbid is Superko.resolve"
                    );
                    assert_eq!(
                        playable_at(&b, c, p, Suicide::Forbid),
                        playable_at_literal(&b, c, p),
                        "playable_at {c} at {p} on {}",
                        render(&b)
                    );
                    assert_eq!(
                        playable_at(&b, c, p, Suicide::RemoveOwn),
                        b.get(p).is_none(),
                        "RemoveOwn keeps the emptiness conjunct alone, at {p} on {}",
                        render(&b)
                    );
                }
            }
        }
    }
}

/// `reaches` and `area` agree with the literal reading at every position, and a
/// stone reaches its own color by the reflexive case of the closure.
#[test]
fn reaches_and_area_are_literal() {
    for dims in small_boards() {
        for code in 0..code_space(dims) {
            let b = decode(dims, PosCode(code));
            for c in [Color::Black, Color::White] {
                for p in dims.points() {
                    assert_eq!(
                        reaches(&b, p, c),
                        reaches_literal(&b, p, c),
                        "reaches {c} from {p} on {}",
                        render(&b)
                    );
                }
                assert_eq!(
                    area(&b, c),
                    area_literal(&b, c),
                    "area {c} on {}",
                    render(&b)
                );
            }
            assert!(
                area(&b, Color::Black) + area(&b, Color::White) <= dims.point_count(),
                "the two areas do not overlap, on {}",
                render(&b)
            );
        }
    }
}

/// The reflexive case of `Reaches` is observable: a `c` stone reaches `c`, and
/// a point of the other color reaches neither by a step it cannot take.
#[test]
fn reaches_is_reflexive_at_a_stone() {
    let dims = Dims::new(1, 2);
    let b = decode(dims, PosCode(1 + 2 * 3)); // "XO"
    assert_eq!(render(&b), "XO");
    assert!(reaches(&b, Point::new(0, 0), Color::Black));
    assert!(!reaches(&b, Point::new(0, 0), Color::White));
    assert!(reaches(&b, Point::new(0, 1), Color::White));
    assert!(!reaches(&b, Point::new(0, 1), Color::Black));
}
