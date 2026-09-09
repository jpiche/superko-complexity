# The decision problem

This document fixes exactly what this project is about. Every claim in
[`claim-ledger.md`](claim-ledger.md) is a claim about the object defined here,
and every definitional choice below is a place where a different choice would
give a different problem — in a few cases, a problem in a different complexity
class.

The formal counterpart is [`../lean/SuperkoComplexity/Defs.lean`](../lean/SuperkoComplexity/Defs.lean).
The two must agree. Where they disagree the Lean is authoritative, and the
disagreement is a bug in this file.

> **Status.** Draft. The choices marked **OPEN** below are not yet settled, and
> `Defs.lean` currently commits to one reading of each. They are the first
> thing to resolve, because every later claim inherits them. No Lean file in
> this repository has been compiled.

## 1. The problem, informally

> Given an `m × n` Go position with komi, under AGA rules — area scoring, no
> suicide, situational superko — does Black have a winning strategy?

Everything difficult is in what "given a position" means. Section 5 is the
part of this document to read first.

## 2. Board and position

The board is the `m × n` grid, `Point = Fin m × Fin n`, with **orthogonal
adjacency**: two points are adjacent when they agree on one coordinate and
differ by one on the other.

A **position** assigns to each point either a color or nothing:
`Position = Point → Option Color`.

A **chain** is a maximal set of same-colored points connected under
adjacency. A chain has a **liberty** when some point adjacent to some member of
the chain is empty.

Generalized Go means the family of problems indexed by board size, with the
size part of the input. The literature is inconsistent about square versus
rectangular boards; nothing here depends on the difference, and rectangular is
kept because the small-board literature this project validates against
(Weninger–Hayward on 1×n, van der Werf on rectangles) uses it.

## 3. Moves and legality

A **move** is either a pass or a play at a point.

Playing color `c` at point `p` resolves in this order:

1. `p` must be empty;
2. place a `c` stone at `p`;
3. remove every `c.other` chain now without a liberty;
4. **no suicide** — if the chain containing `p` now has no liberty, the move
   was illegal. AGA rules forbid suicide; this is a divergence from
   Tromp–Taylor and New Zealand, which permit it.

A **situation** is a position together with the player to move. The **history**
is the set of situations that have occurred, including the current one.

**Situational superko** (AGA Rule 6): a play is illegal if the situation it
would create is already in the history. **Positional superko**: a play is
illegal if the *position* it would create has occurred before, whatever the
player to move. The project studies SSK and treats PSK as the comparison
variant; whether they differ in complexity is itself an open claim (C-12).

### OPEN-1: are passes subject to superko?

`Defs.lean` currently exempts passes: a pass is always legal.

The reading is standard: Tromp–Taylor scopes its repetition clause to moves,
treating a pass as a separate kind of turn, and AGA Rule 6 restricts playing so
as to recreate a position. Both are paraphrases here — neither source is yet
`held` (see [`../references/README.md`](../references/README.md)) — and it is a
reading either way, which matters. A pass changes the player to move without changing the
position, so under SSK an exempt pass can re-enter a situation that a play
could not. That is the parity resource that distinguishes SSK from PSK, and it
is the mechanism behind "sending two, returning one". If passes were subject
to superko the distinction would partly collapse.

*Resolve by:* reading AGA Rule 6 and Tromp–Taylor against each other, and
recording the citation in the ledger. This is a rules question, not a
mathematical one.

## 4. Ending and score

The game ends on **two consecutive passes**.

The score is **mechanical area scoring**, in the Tromp–Taylor formulation: a
player's score is the number of points of their color, plus the number of
empty points that reach their color and not the other, where an empty point
*reaches* a color when a path of adjacent empty points leads from it to a
stone of that color. Black's margin is Black's area minus White's area minus
komi.

### Why not dead stones

AGA rules determine the status of dead stones by agreement, with resumption of
play on dispute. The complexity literature does not model this, and neither
does this project: scoring here is a total function of the final position.

This is a real narrowing and it should be stated rather than absorbed. The
justification is that the two-pass ending under superko makes the mechanical
score coincide with the agreed score under optimal play — a player who
disagrees about a group's status can simply decline to pass and demonstrate.
That justification is an argument, not a theorem, and it is claim C-16.

### OPEN-2: pass stones

AGA Rule 7 requires a passing player to hand the opponent a prisoner. Under
*territory* scoring this is what makes the AGA result agree with the area
result; under area scoring, the received view is that pass stones do not
affect the outcome. `Defs.lean` omits them on that basis.

*Resolve by:* checking whether the omission is exactly neutral under area
scoring, including in the odd cases — unequal pass counts at the end, and games
ending after an odd total number of moves.

### OPEN-3: komi and ties

Komi is a rational. With half-integer komi no tie occurs and the winner is
total. `Defs.lean` awards a tie to White, which is arbitrary and invisible
whenever komi is half-integral.

*Resolve by:* deciding whether the decision problem quantifies over komi or
fixes it. Fixing komi at a half-integer is cleaner and loses nothing, since
hardness constructions choose the komi they need.

## 5. The input encoding — the choice that matters

Under Japanese rules, "given a position, does the player to move win?" is
well-posed, because the only history a legal move depends on is the immediately
preceding position, which is bounded and local. Under superko it is not
well-posed, because legality depends on the entire history, and a position does
not determine its history.

There are at least three different problems hiding in the phrase, and they are
not obviously equivalent:

**(A) Position plus explicit history.** The input is a position together with
the set of forbidden situations. This is the most faithful rendering of
"a superko game in progress", and it is the wrong problem: the input is
exponentially large in the board, so the classification is about a padded
instance and says little about Go. Any claim proved in this encoding must say
so.

**(B) Position plus the move sequence reaching it.** The input is a legal game
record from the empty board. Faithful and polynomially bounded only if the
record is short; a position may need exponentially many moves to reach, so the
input is again potentially exponential, and the problem is really about
*reachable-by-short-history* positions.

**(C) Position as root, history empty.** The input is a position, taken as the
start of play, with nothing forbidden yet. Polynomially bounded input, and the
natural reading of the classical statements.

`Defs.lean` commits to **(C)**.

### OPEN-4: is (C) the right problem, and does reachability matter?

Two objections to (C) deserve answers before anything is built on it.

*Reachability.* A position given as input need not be reachable from the empty
board. Under (C) that is harmless — play starts there and superko constrains
only what is subsequently visited — but it means the problem is about arbitrary
positions rather than about Go games. Whether restricting to reachable
positions changes the classification is open, and it is not obviously
irrelevant: the hardness constructions all build positions they must argue are
reachable.

*Faithfulness.* (C) discards the history that makes superko interesting. A
hardness proof under (C) is a strong result; a *membership* proof under (C) is
weaker than it looks, because it says nothing about positions with a
non-trivial history. The archive argument giving EXPSPACE membership is
insensitive to this, but a hypothetical EXPTIME algorithm might not be.

*Resolve by:* checking which encoding Lichtenstein–Sipser and Robson actually
use, and stating whether the folklore EXPSPACE and EXPTIME arguments survive
each of (A), (B), (C). This is claim C-1 and it blocks the ledger.

## 6. Winning

The game under superko is finite (C-13), so it is determined and a winning
strategy exists for exactly one player.

`Defs.lean` defines "color `c` has a winning strategy from state `s`" as an
inductive predicate with three constructors: the game has ended and `c` won;
it is `c`'s turn and some legal move leads to a state `c` wins; it is the
opponent's turn and every legal move leads to a state `c` wins. The definition
is given for a named color rather than for "the player to move" because the
latter requires a negative occurrence, which Lean's strict positivity check
rejects.

Determinacy — that exactly one color wins from each state — is a theorem, not
part of the definition, and it is claim C-13's corollary.

## 7. What this project decides

> **SUPERKO-GO.** Given `m`, `n`, a position on the `m × n` board, a komi, and
> a color to move, encoded per (C): does Black have a winning strategy under
> AGA rules?

The classification of this problem is the object of the project. Where a claim
concerns PSK rather than SSK, or a different encoding, the ledger says so.
