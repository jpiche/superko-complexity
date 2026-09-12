# The decision problem

This document fixes exactly what this project is about. Every claim in
[`claim-ledger.md`](claim-ledger.md) is a claim about the object defined here,
and every definitional choice below is a place where a different choice would
give a different problem — in a few cases, a problem in a different complexity
class.

The formal counterpart is [`../lean/SuperkoComplexity/Defs.lean`](../lean/SuperkoComplexity/Defs.lean).
The two must agree. Where they disagree the Lean is authoritative, and the
disagreement is a bug in this file.

> **Status.** Draft. `Defs.lean` compiles and is exercised by kernel-checked
> checks. Of the choices marked **OPEN** below, OPEN-1 and OPEN-2 are resolved
> by the rules texts, now held (C-18, C-19), and OPEN-3 by a decision that
> C-27 licenses; OPEN-4 is not settled, and `Defs.lean` commits to one reading
> of it. Every later claim inherits it.

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
   Tromp–Taylor and New Zealand, which permit it. The divergence is
   observable in a game count from four points on a line: from the empty 1×4
   board under positional superko this rule gives 719 178 893 games and
   Tromp–Farnebäck's gives 2 098 407 841 (C-36, `computed`). It is observable
   in a value too: with suicide removed, the 1×2 position `X.` with Black to
   play is worth −2 under positional superko and 0 under situational superko
   (C-54, `computed`), while under this rule no position on a board of at most
   five points has values differing between the two superko rules (C-53,
   `computed`).

A **situation** is a position together with the player to move. The **history**
is the set of situations that have occurred, including the current one.

**Situational superko** (AGA Rule 6): a play is illegal if the situation it
would create is already in the history. **Positional superko**: a play is
illegal if the *position* it would create has occurred before, whatever the
player to move. The project studies SSK and treats PSK as the comparison
variant; whether they differ in complexity is itself an open claim (C-12).

### OPEN-1: are passes subject to superko? — resolved

`Defs.lean` exempts passes: a pass is always legal. Both texts, now held, say
so (C-18, `cited`). AGA Rule 6 reads "It is illegal to play in such a way as
to recreate a previous board position from the game, with the same player to
play", and Rule 2 says "a pass is always legal (Rule 7)". Tromp–Taylor Rule 6
reads "A turn is either a pass; or a move that doesn't repeat an earlier grid
coloring", so the repetition clause scopes to moves by construction.

The consequence: a pass changes the player to move without changing the
position, so under SSK an exempt pass can enter a situation a play could not.
That is **not** what separates SSK from PSK. The two rules differ only at a
play recreating a board that has stood with the mover to play and never with
the opponent to play, and the walk back to that board has odd length, holds at
least three plays, and holds no pass made at that board: a pass archives both
of a board's situations and closes it to both rules (C-50 and C-51, `proved`;
C-52, `proved` by hand). Separating plays occur in games with no pass at all —
from the empty 1×3 board, the fifth move of one such game separates the rules
(C-52; `computed` in `crates/superko-graph/tests/containment.rs`) — so the
difference between them
does not depend on this OPEN, and it survives the reading in which passes are
subject to the repetition rule. A pass made at some other board can still sit
inside such a walk. The three-play walk with no pass, whose middle play removes
two stones and whose outer plays remove one between them, is the shape this
project has called "sending two, returning one"; no source for the name is
held.

One reading the texts do not make for us: AGA Rule 6 names the *situation*
(position with the same player to play) and so is situational superko;
Tromp–Taylor Rule 6 names the grid coloring alone and so is positional. The
project's object is the AGA rule, and `PSK` is defined alongside for the
comparison, as §3 says.

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
play on dispute (Rules 9 and 10, held): if the players still disagree after
resumed play and both pass twice, "any stones remaining on the board are
deemed alive" and the board is counted as it stands. The complexity literature
does not model this, and neither does this project: scoring here is a total
function of the final position.

This is a real narrowing and it should be stated rather than absorbed. The
justification is that the two-pass ending under superko makes the mechanical
score coincide with the agreed score under optimal play — a player who
disagrees about a group's status can simply decline to pass and demonstrate.
That justification is an argument, not a theorem, and it is claim C-16.

### OPEN-2: pass stones — resolved

AGA Rule 7 requires a passing player to hand the opponent a prisoner, and
Rule 11 requires White to make the last move, by an extra pass if necessary,
so that both players have taken the same number of turns. Under *territory*
counting this is what makes the two counting methods agree. Under *area*
counting the text itself disposes of the question: Rule 12 says "When
counting by area, the players add up their total area. Prisoners are
ignored." A pass stone is a prisoner, so it cannot enter the area result,
and White's extra pass changes no point of the board (C-19, `cited`).
`Defs.lean` omits pass stones on that basis; the odd cases — unequal pass
counts, an odd total number of turns — are odd only for territory counting.

### OPEN-3: komi and ties — resolved

Komi is a rational. With half-integer komi no tie occurs and the winner is
total. `Defs.lean` awards a tie to White.

The tie half is settled by theorem. `winner` sees komi only through its floor,
and every komi is equivalent, for `BlackWins`, to the half-integer
`⌊komi⌋ + ½` (C-27, `proved`): the tie convention is unobservable for
`BlackWins` at a fixed color to move. It *is* observable where a color swap
is attempted — with the board's colors swapped and komi negated, both winners
are White at margin zero — so no prose may call it unobservable without that
qualification.

The quantification half is settled by decision. The decision problem
quantifies over komi, which is part of the input and is encoded exactly by
C-31's `enc`. Fixing komi would have lost nothing for hardness, which chooses
its komi, but it would have narrowed the problem for no gain once C-27 shows
that every komi is an ordinary input. What remains open is the clamp: that
`BlackWins` depends on komi only through its floor clamped to the interval
from −(m·n)−1 to m·n is proved at the level of `winner` and not yet lifted
through `WinsFor` (C-35). Until it is, "input size polynomial in m·n" holds
for the instances a reduction produces, and the honest size for an arbitrary
input carries the komi's bit length.

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

**(C) Position as root, history the root situation alone.** The input is a
position, taken as the start of play; the history holds that one situation and
nothing else is forbidden yet. Polynomially bounded input, and the reading the
literature takes: Lichtenstein and Sipser ask about "an arbitrary GO position
on an n × n board", Saffidine, Teytaud and Yen call a position with no history
of forbidden states "the classical considered setting", and Hearn rejects
history-as-position because complexity results would then be taken relative
to an exponentially larger input. Stockmeyer and Chandra speak of "the size of
the starting position", on the strength of their abstract only.

`Defs.lean` commits to **(C)**: `start` seeds `seen` with the root situation.
That is the reading both rules texts support. AGA Rule 6 forbids recreating
"a previous board position from the game", and the position a game starts
from is a position from the game; Tromp–Taylor Rule 5 starts play "with an
empty grid" and Rule 6 forbids repeating "an earlier grid coloring", of which
the initial grid is one. The extension to a game that starts from an
arbitrary position — where the rules texts always start from the empty or
handicap board — is this project's decision, and it is the one consistent
with both wordings. The earlier phrase "history empty" was inexact, and
whether any position's value differs between the two readings is not
settled; nothing rests on it now that the reading is fixed.

The bit-level encoding of an instance is
`Superko.Enc.enc` in [`../lean/SuperkoComplexity/Encoding.lean`](../lean/SuperkoComplexity/Encoding.lean):
the dimensions in self-delimiting binary, one bit for the color to move, two
bits per point in row-major order, then the komi as a sign, a numerator and a
denominator in binary. Its length is exactly `2·m·n` plus terms logarithmic
in the dimensions and the komi (C-31). No source fixes a bit-level encoding,
so the identification of `enc` with the literature's is asserted by
inspection; that, together with OPEN-4, is what keeps C-1 open.

### OPEN-4: is (C) the right problem, and does reachability matter?

Two objections to (C) deserve answers before anything is built on it.

*Reachability.* A position given as input need not be reachable from the empty
board. Under (C) that is harmless — play starts there and superko constrains
only what is subsequently visited — but it means the problem is about arbitrary
positions rather than about Go games. `Position m n` ranges over all
`3^(m·n)` colorings, including ones no play produces and ones with libertyless
chains. Whether restricting to reachable positions changes the classification
is open. Lichtenstein and Sipser do not argue that their constructed positions
are reachable, and say their result has no a priori bearing on play from the
empty board; so at least one hardness construction is content with arbitrary
positions, and the restriction is a separate problem rather than a premise of
the literature.

*Faithfulness.* (C) discards the history that makes superko interesting. A
hardness proof under (C) is a strong result; a *membership* proof under (C) is
weaker than it looks, because it says nothing about positions with a
non-trivial history. The archive argument giving EXPSPACE membership is
insensitive to this, but a hypothetical EXPTIME algorithm might not be.

*Resolve by:* checking which encoding Lichtenstein–Sipser and Robson actually
use, and stating whether the folklore EXPSPACE and EXPTIME arguments survive
each of (A), (B), (C). This is claim C-1 and it blocks the ledger.

## 6. Winning

The game under superko is finite (C-13), and it is determined: from every
state exactly one color has a winning strategy (C-28, `proved`). Existence
recurses on the well-founded relation of C-13 and needs a rule that excludes
repeats; exclusivity needs no hypothesis at all.

`Defs.lean` defines "color `c` has a winning strategy from state `s`" as an
inductive predicate with three constructors: the game has ended and `c` won;
it is `c`'s turn and some legal move leads to a state `c` wins; it is the
opponent's turn and every legal move leads to a state `c` wins. The definition
is given for a named color rather than for "the player to move" because the
latter requires a negative occurrence, which Lean's strict positivity check
rejects.

Determinacy — that exactly one color wins from each state — is a theorem, not
part of the definition: claim C-28, proved by well-founded induction on the
relation C-13 supplies.

## 7. What this project decides

> **SUPERKO-GO.** Given `m`, `n`, a position on the `m × n` board, a komi, and
> a color to move, encoded per (C): does Black have a winning strategy under
> AGA rules?

In Lean the predicate is `Superko.Enc.BlackWinsFrom m n b komi c`, which is
`WinsFor m n SSK komi .black (start b c)` — a derived notion, since `WinsFor`
already takes the starting color — and the language over bit strings is
`Superko.Enc.goLang`, the set of `enc m n c b komi` for which it holds.
`Defs.lean`'s `BlackWins` is the Black-to-move slice, definitionally. The
color to move is part of the input so that a hardness reduction whose
instances have White to move lands in the language directly, with no
complement and no appeal to determinacy.

The classification of this problem is the object of the project. Where a claim
concerns PSK rather than SSK, or a different encoding, the ledger says so.
