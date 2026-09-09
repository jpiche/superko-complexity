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
//! # Status
//!
//! Empty. The definitions lead: `Defs.lean` and `docs/formal-model.md` settle
//! what the rules are, and this crate mirrors the answer. Writing it first
//! would mean guessing at the choices that document still marks OPEN.
