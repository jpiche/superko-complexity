//! Position codes and the string form.

use superko_rules::code::{
    MAX_CODED_POINTS, ParseError, PosCode, code_space, decode, encode, parse, render,
};
use superko_rules::config::{Dims, Point};
use superko_rules::reference::{Color, Position};

#[test]
fn poscode_round_trips_over_the_whole_space() {
    for (rows, cols) in [(1usize, 1usize), (1, 4), (2, 2), (2, 3)] {
        let dims = Dims::new(rows, cols);
        let span = code_space(dims);
        assert_eq!(span, 3u32.pow(u32::try_from(rows * cols).expect("fits")));
        for raw in 0..span {
            let b = decode(dims, PosCode(raw));
            assert_eq!(encode(&b), PosCode(raw), "code {raw} on {dims}");
            assert_eq!(b.dims(), dims);
        }
    }
}

#[test]
fn poscode_digit_is_row_major() {
    let dims = Dims::new(2, 3);
    let mut b = Position::empty(dims);
    assert_eq!(encode(&b), PosCode(0));
    b.set(Point::new(0, 0), Some(Color::Black));
    assert_eq!(encode(&b), PosCode(1));
    let mut b = Position::empty(dims);
    b.set(Point::new(0, 1), Some(Color::White));
    assert_eq!(encode(&b), PosCode(2 * 3));
    let mut b = Position::empty(dims);
    b.set(Point::new(1, 0), Some(Color::Black));
    assert_eq!(encode(&b), PosCode(3u32.pow(3)));
}

#[test]
fn max_coded_points_is_the_largest_that_fits_a_u32() {
    assert!(3u64.pow(u32::try_from(MAX_CODED_POINTS).expect("fits")) <= u64::from(u32::MAX));
    assert!(3u64.pow(u32::try_from(MAX_CODED_POINTS + 1).expect("fits")) > u64::from(u32::MAX));
}

#[test]
fn string_form_round_trips() {
    let dims = Dims::new(2, 3);
    for raw in 0..code_space(dims) {
        let b = decode(dims, PosCode(raw));
        let text = render(&b);
        assert_eq!(parse(dims, &text).expect("its own rendering parses"), b);
    }
    assert_eq!(render(&Position::empty(Dims::new(2, 3))), ".../...");
}

#[test]
fn parser_rejects_ragged_rows() {
    let dims = Dims::new(2, 2);
    assert_eq!(
        parse(dims, "XX/X"),
        Err(ParseError::RowLength {
            row: 1,
            expected: 2,
            found: 1
        })
    );
    assert_eq!(
        parse(dims, "XXX/.."),
        Err(ParseError::RowLength {
            row: 0,
            expected: 2,
            found: 3
        })
    );
}

#[test]
fn parser_rejects_the_wrong_row_count() {
    let dims = Dims::new(2, 2);
    assert_eq!(
        parse(dims, "XX"),
        Err(ParseError::RowCount {
            expected: 2,
            found: 1
        })
    );
    assert_eq!(
        parse(dims, "XX/../.."),
        Err(ParseError::RowCount {
            expected: 2,
            found: 3
        })
    );
    // The same text is a legal 1x4 board and an illegal 2x2 one, which is why
    // the parser takes the dimensions rather than inferring them.
    assert!(parse(Dims::new(1, 4), "X.O.").is_ok());
    assert_eq!(
        parse(Dims::new(2, 2), "X.O."),
        Err(ParseError::RowCount {
            expected: 2,
            found: 1
        })
    );
}

/// An empty row is a row, not a separator to be discarded. A parser that
/// dropped it would read `"XX//.."` as the two rows of a 2x2 board, so a text
/// with a row missing would be scored as a different position with no error —
/// and a position crosses a file boundary as exactly this text.
#[test]
fn parser_rejects_an_empty_row() {
    let dims = Dims::new(2, 2);
    for text in ["XX//..", "/XX/..", "XX/../", "XX\n\n.."] {
        assert_eq!(
            parse(dims, text),
            Err(ParseError::RowCount {
                expected: 2,
                found: 3
            }),
            "{text:?} has three rows, one of them empty"
        );
    }
    assert_eq!(
        parse(Dims::new(1, 2), "//"),
        Err(ParseError::RowCount {
            expected: 1,
            found: 3
        })
    );
    assert_eq!(
        parse(Dims::new(1, 2), ""),
        Err(ParseError::RowLength {
            row: 0,
            expected: 2,
            found: 0
        })
    );
}

#[test]
fn parser_rejects_a_stray_character() {
    assert_eq!(
        parse(Dims::new(2, 2), "X./.x"),
        Err(ParseError::BadChar {
            row: 1,
            col: 1,
            found: 'x'
        })
    );
}

#[test]
fn parser_accepts_newline_separated_rows() {
    let dims = Dims::new(2, 2);
    assert_eq!(
        parse(dims, "XO\n.X").expect("newlines separate rows"),
        parse(dims, "XO/.X").expect("slashes separate rows")
    );
}
