# Documentation layout

Four tiers, split by what a document is *for* rather than what it is about.
The fourth tier is the one a product repository does not need.

## `docs/` — core reference

**What is known, what is believed, and on what evidence.** Describes the state
of the work as it stands. A reader who has never seen a plan should be able to
work from these alone.

| | |
|---|---|
| [`style.md`](style.md) | how documents and comments are written here — the status rule, the terminology, the notation |
| [`formal-model.md`](formal-model.md) | **the decision problem**: the ruleset, the input encoding, the state, and every definitional choice with its justification |
| [`trusted-base.md`](trusted-base.md) | what a skeptical reader must believe, and what they need not |
| [`claim-ledger.md`](claim-ledger.md) | every claim: id, statement, status, formalization, dependencies, witness |
| [`state-of-the-art.md`](state-of-the-art.md) | what the literature has established, by whom, and where it stops |
| [`open-questions.md`](open-questions.md) | the live fronts, each pointing at the claims that would close it |

`formal-model.md` and [`../lean/SuperkoComplexity/Defs.lean`](../lean/SuperkoComplexity/Defs.lean)
are a matched pair: the prose and the formal text must say the same thing.
When they disagree, the Lean is authoritative and the prose is a bug.

**Scope rule:** these describe the work as it is. No phase numbers, no
work-item ids, no narrative of how a result was reached. That belongs in a
plan or in the notebook.

## `docs/plans/` — active and draft plans

Work in progress: research programmes, attack plans, and the review artifacts
they produce. These may reference phases, steps and open questions freely. A
plan that is finished and still worth reading moves to `finished/`.

## `docs/finished/` — archive

Completed plans retained because they explain why something is shaped the way
it is. Not maintained. Read as history.

## `../notebook/` — the lab notebook

**Outside `docs/` on purpose.** The core tier forbids history; research needs
history somewhere. The notebook is dated, append-only, and holds the material
the other tiers exclude: hunches, half-arguments, approaches tried and
abandoned, and the reasons.

A refuted approach recorded here is one nobody has to try again. When a dead
end explains a present-tense limit, lift the limit into `docs/` and leave the
story here.
