//! Board, moves, repetition rules, and history-carrying state — the executable
//! mirror of `lean/SuperkoComplexity/Defs.lean`.
//!
//! # This crate is not trusted
//!
//! Nothing here establishes anything. It searches fast; Lean decides what is
//! true. A result computed by this crate enters the record as a `computed`
//! ledger row and a certificate under `certificates/`, and becomes `proved`
//! only when a Lean checker accepts that certificate.
//!
//! # The obligation this crate does carry
//!
//! It must agree with `Defs.lean`. The agreement is tested, not proved, and
//! the test is the acceptance suite: reproducing quantities other people
//! computed independently.
//!
//! | Quantity | Published value | Source |
//! |---|---|---|
//! | 2x2 games under positional superko | 386,356,909,593 | Tromp |
//! | Legal positions `L(m, n)` | table | Tromp-Farnebäck |
//! | 1xn minimax scores under PSK | table | Weninger-Hayward |
//!
//! Disagreement means this crate or the Lean core is wrong, not the
//! literature. See the README's "Definitional validation".
//!
//! # One implementation of the rules
//!
//! [`reference`] is a transliteration of `Defs.lean`, one item per Lean item,
//! in that file's order, written for a reader holding the Lean open. It is the
//! only place a rule is decided. [`table`] is not a second engine: its only
//! constructor calls the reference functions over every position, color and
//! point, so the fast path is a memo of the slow one and cannot drift from it.
//!
//! Two axes vary, and both are values rather than build configurations, so a
//! single binary can run all four combinations and a witness header can name
//! the one a count came from: [`config::Suicide`], whose `Forbid` is
//! `Defs.lean` and whose `RemoveOwn` has no Lean counterpart at all, and
//! [`config::Repetition`], which chooses between `Superko.SSK` and
//! `Superko.PSK`.
//!
//! Every other departure from a literal reading of `Defs.lean` is a variant of
//! [`divergence::Divergence`], carries a `DIVERGENCE:` marker at the line that
//! departs, and prints a consequence sentence in the witness header of any
//! result it was in force for. `tools/check-mirror.sh` holds the two sets
//! equal, and holds every `Defs.lean` item to a `**Mirrors**` doc line.
//!
//! # Status
//!
//! The mirror, the table, the archive and the divergence register exist and
//! are tested. This crate holds no enumerator and produces no count itself;
//! the enumerators are in `superko-graph`, which calls the items below.
//!
//! What the tests establish is `computed` and bounded: the `Sanity.lean`
//! checks hold by a second route on the same fixtures; every board function
//! agrees at every position with a second reading of `Defs.lean` that takes
//! `∃ r, Adj q r ∧ ..` as a scan over all `m * n` points instead of over the
//! four neighbors, for every board with `m * n <= 6`; the table equals its
//! generator over the same boards; the two-bit archive gives the same verdict
//! as [`reference::psk`] and [`reference::ssk`] on every state a short
//! enumeration from the empty 1x3 and 2x2 boards reaches;
//! [`reference::legality`] agrees with the rule predicates over the same walk,
//! under both repetition rules and both suicide conventions; and
//! [`reference::all_moves`] is pinned to a literal list, because every count
//! this workspace will produce depends on that order. Separately,
//! `tests/lean_oracle.rs` replays the fixtures under `test_data/lean-oracle/`,
//! which the Lean *compiler* produced from the computable twins of
//! `Defs.lean`: those rows are graded `observed`, the trust surface
//! `native_decide` has and not the kernel's, so they support no ledger entry
//! either.
//!
//! What they do not establish: that this crate agrees with `Defs.lean` on
//! anything larger than the boards those tests and fixtures cover. Of the
//! acceptance suite above, `superko-graph` has reproduced the legal-position
//! table `L(m, n)`, the 1xn game counts under the publication's own suicide
//! convention, and the 2x2 game count under both conventions (all
//! `computed`, recorded under `results/`, and cited by the ledger's C-23,
//! C-38 and C-39); on 1x4 the no-suicide rule gives a different count
//! (C-36). The 1xn minimax scores reproduce for n <= 6, by `superko-solve`
//! (C-24).

pub mod archive;
pub mod code;
pub mod config;
pub mod divergence;
pub mod reference;
pub mod table;

pub use archive::{Archive, ArchiveKey};
pub use code::{ParseError, PosCode};
pub use config::{Dims, Point, Repetition, RuleConfig, Suicide};
pub use divergence::Divergence;
pub use reference::{Color, Move, Position, Rat, Refusal, Situation, State};
pub use table::RuleTable;
