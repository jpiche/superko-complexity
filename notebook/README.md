# Lab notebook

Dated, append-only, one file per session or per thread:
`2026-09-09-encoding-question.md`.

Every entry opens with two lines naming who is accountable and which models did
the work:

```
**Author:** Joseph J. Piché
**Models:** Claude Opus 5 (`claude-opus-5`)
```

`**Models:** none` when no model was involved. See the Attribution section of
[`../CLAUDE.md`](../CLAUDE.md) for why the specific id matters and the blanket
disclosure in the README does not.

**Outside `docs/` on purpose.** The core tier describes the work as it stands
and forbids history. Research needs history somewhere, and this is where it
goes: hunches, half-arguments, approaches tried and abandoned, and the reasons
they were abandoned.

## What belongs here

- An approach that did not work, and how far it got before failing.
- A calculation that came out wrong, with the wrong answer left visible.
- A reading of a source that changed an assumption.
- The reasoning behind a decision that `docs/` records only as an outcome.

## Why

A refuted approach recorded is one nobody has to try again — including a future
agent with none of this session's context, which is most of them.

The rule: when a dead end explains a limit that still holds, lift the limit
into `docs/` in the present tense and leave the story here. Nothing here is
deleted.
