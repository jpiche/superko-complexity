# Style

How documents and comments are written here. The rule applies to `docs/`,
to Lean docstrings, to Rust comments and to commit messages.

## The status rule

**No mathematical assertion without its status.**

Every claim about the mathematics carries, explicitly or by an unambiguous
reference to [`claim-ledger.md`](claim-ledger.md), one of the ledger statuses.
Prose that asserts flatly what the ledger records as `folklore`, `computed` or
`conjecture` is a defect.

| Wrong | Right |
|---|---|
| Superko Go is in EXPSPACE. | SUPERKO-GO is in EXPSPACE by an archive argument that no source proves (C-3). |
| The construction reduces to undirected vertex geography. | Demaine and Hearn observe that *Robson's* construction reduces to undirected vertex geography (C-6); the observation does not extend to arbitrary positions. |
| H(n) is exponential. | H(n) is exponential on the boards enumerated so far (C-15, `computed`); the asymptotics are open. |

The words *clearly*, *obviously*, *evidently*, *it follows that* and *of
course* are banned outright. Each is a place where a status was skipped.

## Hedging is not the same as honesty

The status rule is not a license to hedge everything. A `proved` claim is
stated flatly, in the indicative, with no softening — that is the point of
proving it. Uniform tentativeness destroys exactly the signal this project
exists to produce.

## No history in the core tier

`docs/` describes the state of the work as it stands. No phase numbers, no
work-item ids, no narrative of how a result was reached, no "we originally
tried". Where a dead end explains a limit that still holds, state the limit in
the present tense and put the story in [`../notebook/`](../notebook/).

Refuted approaches are valuable and are never deleted — they move to the
notebook, or to a `refuted` ledger row with the counterexample.

## Keep measurements with their witness

When a number appears, the command that produced it appears too, or the
`results/` file that holds it does. A measurement nobody can reproduce is an
opinion. This is what [`../tools/verify-results.sh`](../tools/verify-results.sh)
enforces.

## Citations

Name the source in the ledger row, and in prose name the author and year.
Distinguish what a source *proves* from what it *asserts* from what it
*reports someone believes* — this literature contains all three about the same
question, and collapsing them is how the folklore became folklore.

## Prose

*The Elements of Style*. Active voice, statements in positive form, no
needless words, language definite, specific and concrete.

Write for a reader who knows Go and knows complexity theory but has never seen
this project. Do not explain what a ko is. Do explain which of the several
things "superko" might mean is meant here.

## Notation

- **Mathematical notation** in prose: typeset Unicode — `≤ ≥ ≠ × · ± → ⇒ ∈ ∪
  ∩ ⊆ ∅ ⌈⌉ ⌊⌋ ℕ ℤ ℚ α β Σ`. ASCII inside backticks, where it quotes code or
  Lean exactly.
- **Complexity classes** in small caps in the paper, plain uppercase here:
  PSPACE, EXPTIME, EXPSPACE.
- **Rulesets**: spell out *positional superko* and *situational superko* on
  first use in a document; PSK and SSK thereafter. Never write bare "superko"
  where the distinction could matter, which is nearly everywhere.
- **Claim references** inline as `(C-14)`, no link needed; the ledger is the
  index.
- **Lean names** in backticks, fully qualified on first use in a document.

## Lean docstrings

Every definition in [`../lean/SuperkoComplexity/Defs.lean`](../lean/SuperkoComplexity/Defs.lean)
carries a docstring saying what it means *in Go terms*, not in type-theoretic
terms. The file's readers are auditing whether it describes Go; a docstring
that restates the type signature helps nobody.

Where a definition embodies a rules choice, the docstring names the choice and
points at the `formal-model.md` section that justifies it.
