//! The superko archive: a dense bitset with two bits per position code.
//!
//! Bit `2·code` says the position has been seen with Black to move, bit
//! `2·code + 1` with White. Situational superko reads one bit; positional
//! superko reads the two together. One structure serves both rules, so a
//! difference between an SSK count and a PSK count is attributable to the rule
//! and not to two pieces of bookkeeping that differ elsewhere, and the
//! containment "PSK-legal implies SSK-legal" is a per-edge check that costs
//! nothing.
//!
//! No key is ever a hash. At the operation counts this workspace reaches, a
//! 64-bit digest is within birthday range, and a collision prunes a subtree
//! silently — an undercount with no symptom.

use crate::code::{PosCode, code_space};
use crate::config::{Dims, Repetition};
use crate::reference::Color;

/// A position together with the player to move — what a superko archive holds.
///
/// This is the index of a bit, not a situation: it is only meaningful on the
/// board it was taken on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArchiveKey {
    /// The position.
    pub code: PosCode,
    /// Whose turn it is in that position.
    pub to_move: Color,
}

impl ArchiveKey {
    /// The key for a position and a player to move.
    #[must_use]
    pub const fn new(code: PosCode, to_move: Color) -> Self {
        Self { code, to_move }
    }

    const fn bit(self) -> usize {
        let base = (self.code.0 as usize) * 2;
        match self.to_move {
            Color::Black => base,
            Color::White => base + 1,
        }
    }
}

/// Every situation a game has passed through, as two bits per position.
#[derive(Clone, Debug)]
pub struct Archive {
    dims: Dims,
    codes: usize,
    words: Vec<u64>,
    len: usize,
}

impl Archive {
    /// An empty archive over the whole code space of a board.
    ///
    /// # Panics
    ///
    /// Panics when the board is too large for a position code.
    #[must_use]
    pub fn new(dims: Dims) -> Self {
        let codes = code_space(dims) as usize;
        Self {
            dims,
            codes,
            words: vec![0u64; (codes * 2).div_ceil(64)],
            len: 0,
        }
    }

    /// The board this archive is over.
    #[must_use]
    pub const fn dims(&self) -> Dims {
        self.dims
    }

    /// How many situations are archived.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether nothing is archived. An archive reached from
    /// `reference::State::start` is never empty: the root is seeded.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn check(&self, key: ArchiveKey) -> usize {
        assert!(
            (key.code.0 as usize) < self.codes,
            "code is not a position of this board"
        );
        key.bit()
    }

    /// Archive a situation, and say whether it was new.
    ///
    /// The answer is the caller's obligation, not a convenience: `now ∈ seen`
    /// is an invariant of `Superko.State`, so a pass always inserts a key
    /// whose position is already archived under the other color's bit — or,
    /// after two passes, the very bit that is already set. A make and unmake
    /// that cleared unconditionally would forget a genuine earlier occurrence
    /// and undercount.
    ///
    /// # Panics
    ///
    /// Panics when the key is not of this board.
    #[must_use = "undo(key, was_new) needs this answer; clearing unconditionally undercounts"]
    pub fn insert(&mut self, key: ArchiveKey) -> bool {
        let bit = self.check(key);
        let mask = 1u64 << (bit % 64);
        let word = &mut self.words[bit / 64];
        let was_new = *word & mask == 0;
        if was_new {
            *word |= mask;
            self.len += 1;
        }
        was_new
    }

    /// Undo an [`Archive::insert`], clearing only what that insert set.
    ///
    /// # Panics
    ///
    /// Panics when the key is not of this board, or when `was_new` says a bit
    /// was set that is not set now — which means the make and unmake were not
    /// paired.
    ///
    /// The key is checked before `was_new` is read, so `undo` refuses a key of
    /// the wrong board on the same domain [`Archive::insert`] does; a
    /// make-and-unmake pair that had drifted onto another board would otherwise
    /// go unnoticed on exactly the moves that do not clear a bit.
    pub fn undo(&mut self, key: ArchiveKey, was_new: bool) {
        let bit = self.check(key);
        if !was_new {
            return;
        }
        let mask = 1u64 << (bit % 64);
        let word = &mut self.words[bit / 64];
        assert!(*word & mask != 0, "undo of an insert that did not happen");
        *word &= !mask;
        self.len -= 1;
    }

    /// Whether this exact situation is archived — the read situational
    /// superko makes.
    ///
    /// # Panics
    ///
    /// Panics when the key is not of this board.
    #[must_use]
    pub fn contains_ssk(&self, key: ArchiveKey) -> bool {
        let bit = self.check(key);
        self.words[bit / 64] & (1u64 << (bit % 64)) != 0
    }

    /// Whether this position is archived under either color — the read
    /// positional superko makes.
    ///
    /// `Superko.PSK` quantifies over archived situations and compares boards,
    /// so reading the two bits of a code is a projection of that quantifier.
    /// The Lean lemma licensing it is not written; the divergence
    /// [`crate::divergence::Divergence::PskArchiveProjection`] carries that
    /// status.
    ///
    /// # Panics
    ///
    /// Panics when the code is not of this board.
    // DIVERGENCE: psk-archive-projection
    #[must_use]
    pub fn contains_psk(&self, code: PosCode) -> bool {
        self.contains_ssk(ArchiveKey::new(code, Color::Black))
            || self.contains_ssk(ArchiveKey::new(code, Color::White))
    }

    /// The read the named repetition rule makes.
    ///
    /// # Panics
    ///
    /// Panics when the key is not of this board.
    #[must_use]
    pub fn contains(&self, key: ArchiveKey, rep: Repetition) -> bool {
        match rep {
            Repetition::Ssk => self.contains_ssk(key),
            Repetition::Psk => self.contains_psk(key.code),
        }
    }

    /// Forget everything.
    pub fn clear(&mut self) {
        self.words.iter_mut().for_each(|w| *w = 0);
        self.len = 0;
    }
}
