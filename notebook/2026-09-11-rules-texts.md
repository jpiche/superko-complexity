# 2026-09-11 — the rules texts, read

**Author:** Joseph J. Piché
**Models:** Claude Fable 5.1 (`claude-fable-5-1`)

The AGA rules (the Rules Committee text dated 1991-09-01, as amended — its
Rule 3 carries the 7½ komi of 2004) and the Tromp–Taylor rules were held
and read today. Two ledger rows moved from unsettled to `cited`, one OPEN
item in `formal-model.md` closed by each, and C-1's root-seeding question
turned out to be answered by the wording of both texts. Nothing in
`Defs.lean` changed, and nothing needed to: every reading it had committed to
is the one the texts support.

## What each text says

- **Passes and repetition (OPEN-1, C-18).** AGA Rule 6: "It is illegal to
  play in such a way as to recreate a previous board position from the game,
  with the same player to play." Rule 2: "a pass is always legal (Rule 7)".
  Tromp–Taylor Rule 6: "A turn is either a pass; or a move that doesn't
  repeat an earlier grid coloring." Passes are exempt in both, by wording
  rather than by interpretation.
- **Pass stones (OPEN-2, C-19).** AGA Rule 12, area counting: "Prisoners are
  ignored." A pass stone is a prisoner (Rule 7), so it cannot enter an area
  result; Rule 11's extra White pass changes no point of the board. The
  received view was right, and it is now a citation rather than a view.
- **The root situation (C-1).** AGA forbids recreating a position "from the
  game"; Tromp–Taylor starts "with an empty grid" and forbids repeating "an
  earlier grid coloring". The starting position counts in both, which is
  what `start` does by seeding the history with the root situation. The
  texts start every game from the empty or handicap board; starting from an
  arbitrary position is this project's decision, recorded as such.
- **Which superko.** AGA Rule 6 names the situation — "with the same player
  to play" — and is situational; Tromp–Taylor's Comment 6 says its rule is
  positional and that the difference matters "only in exceedingly rare
  cases". The project's object is the AGA rule; PSK is the comparison.
- **Dead stones (C-16).** AGA Rules 9 and 10: disputes are settled by
  resumed play, and if both players pass twice while still disagreeing,
  "any stones remaining on the board are deemed alive" and the board is
  counted as it stands. That is the mechanism the C-16 argument uses; C-16
  stays `conjecture`, since the argument is still an argument.
- **Suicide.** AGA Rule 5: "self-capture is illegal". Tromp–Taylor Comment 7
  allows it. `Defs.lean` forbids it, as §3 said.
- **Area.** AGA Rule 12's area — live stones plus empty regions "entirely
  surrounded by stones of a single color", neutral points to neither — is
  Tromp–Taylor's reach-based definition in other words.

## How the texts were found

A rules reference maintained in another project of mine mapped the texts
and their divergences with citations by rule number, and pointed at the
primary copies. It was used as a map only: every sentence above is cited
to the primary text, and nothing from that document is carried into this
repository, per the provenance note in `references/README.md`.

## What this does not settle

C-1's remaining content: that the concrete encoding `enc` is the
literature's unstated one up to polynomial recoding, which is asserted by
inspection, and OPEN-4's reachability question. Neither is a rules
question. The Chinese rules text is still `sought`; it matters for the
PSK comparison and for reading Robson's "Chinese version of Go", not for
anything `Defs.lean` commits to.
