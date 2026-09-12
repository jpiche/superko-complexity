/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import Mathlib.Logic.Relation
import Mathlib.Data.Set.Card
import Mathlib.Data.Rat.Defs
import Mathlib.Data.Fintype.Pi

/-!
# The trusted definitional core

**This file is the audit target.** It holds every definition that the main
theorem statement mentions, and nothing else. A reader deciding whether to
believe a result from this project reads this file and checks that it
describes Go; everything downstream is checked by Lean's kernel and needs no
human review. See `docs/trusted-base.md`.

Derived notions belong in `Basic.lean`, not here. Every addition to this file
enlarges what a reader must audit and invalidates prior auditing.

## Status

**Compiles** against Lean 4.34.0-rc2 and Mathlib. The definitions have been
exercised on concrete positions — see `Sanity.lean`, whose checks are proved by
`decide` and so are verified by the kernel rather than observed.

What that establishes is limited and worth stating precisely. The checks are
hand-computed, so they catch only errors the author thought to test. The
project's actual definitional validation is the independent-agreement argument
in `docs/trusted-base.md` — reproducing counts computed by other people, above
all the 2×2 game count under positional superko — and none of that has been
done. These definitions are plausible, not validated.

## Rules modeled

AGA rules: area scoring, suicide forbidden, situational superko, two passes
end the game. Positional superko is defined alongside as the comparison
variant. Every choice that a different reading of the rules would change is
recorded in `docs/formal-model.md`, and the ones still unsettled are marked
OPEN there. The load-bearing ones:

* the input is a position taken as the root of play with an empty history
  (`docs/formal-model.md` §5, encoding (C), claim C-1);
* passes are exempt from the repetition rule (OPEN-1, claim C-18);
* scoring is mechanical — there is no dead-stone determination (§4, C-16);
* pass stones are omitted as neutral under area scoring (OPEN-2, C-19).

## Classical definitions

The definitions here are classical and therefore noncomputable: chains and
reachability are transitive closures, and deciding them inline would bloat the
file with instance plumbing that an auditor would have to read. Brevity of this
file is worth more than computability of it.

Decidable counterparts, proved equal to these, live in `Basic.lean`. Those are
what the certificate checkers in `Certificates/` use. The bridging lemmas are
the reason a certificate check says something about *these* definitions.
-/

open scoped Classical

namespace Superko

/- The definitions below are classical, hence noncomputable: chains and
reachability are transitive closures, and deciding them inline would fill the
audit target with instance plumbing. Decidable counterparts, proved equal to
these, live in `Basic.lean` and are what the certificate checkers run on. -/
noncomputable section

/-! ## Color -/

/-- The two colors of stone. -/
inductive Color where
  | black : Color
  | white : Color
  deriving DecidableEq, Repr

/-- The opposing color. -/
def Color.other : Color → Color
  | .black => .white
  | .white => .black

/-! ## Board and position -/

/-- A point of the `m × n` board. -/
abbrev Point (m n : ℕ) : Type := Fin m × Fin n

/-- Two points are **adjacent** when they agree on one coordinate and differ by
one on the other: the four orthogonal neighbors of the Go board. Diagonals are
not adjacency. -/
def Adj {m n : ℕ} (p q : Point m n) : Prop :=
  (p.1 = q.1 ∧ (p.2.val + 1 = q.2.val ∨ q.2.val + 1 = p.2.val)) ∨
  (p.2 = q.2 ∧ (p.1.val + 1 = q.1.val ∨ q.1.val + 1 = p.1.val))

/-- A **position** gives each point of the board a stone or leaves it empty. -/
abbrev Position (m n : ℕ) : Type := Point m n → Option Color

/-- Adjacent points carrying stones of the same color — the relation whose
connected components are chains. -/
def Joined {m n : ℕ} (b : Position m n) (p q : Point m n) : Prop :=
  Adj p q ∧ b p = b q ∧ (b p).isSome

/-- The **chain** containing `p`: the stones connected to `p` through
same-colored neighbors. Empty at `p` gives the empty relation, hence the
singleton `{p}`, which no rule below consults. -/
def chain {m n : ℕ} (b : Position m n) (p : Point m n) : Set (Point m n) :=
  {q | Relation.ReflTransGen (Joined b) p q}

/-- A chain has a **liberty** when some point adjacent to some stone of the
chain is empty. A chain without a liberty is captured. -/
def HasLiberty {m n : ℕ} (b : Position m n) (p : Point m n) : Prop :=
  ∃ q ∈ chain b p, ∃ r, Adj q r ∧ b r = none

/-- Remove every `c` stone whose chain has no liberty. -/
def clear {m n : ℕ} (b : Position m n) (c : Color) : Position m n :=
  fun q => if b q = some c ∧ ¬ HasLiberty b q then none else b q

/-- The position after `c` places a stone at `p` and every opposing chain left
without a liberty is removed.

Capture is resolved for the opponent only. A move that leaves the played stone's
own chain without a liberty is suicide, and `Legal` forbids it rather than
resolving it — AGA rules prohibit suicide, unlike Tromp–Taylor and New Zealand.
-/
def resolve {m n : ℕ} (b : Position m n) (c : Color) (p : Point m n) :
    Position m n :=
  clear (Function.update b p (some c)) c.other

/-! ## Situations, states and moves -/

/-- A **situation**: a position together with the player to move.

The repetition rules are stated over situations because that is what
distinguishes situational from positional superko — SSK forbids recreating a
situation, PSK a position. -/
structure Situation (m n : ℕ) where
  /-- The stones on the board. -/
  board : Position m n
  /-- Whose turn it is. -/
  toMove : Color

/-- A **move** is a pass or a play at a point. -/
inductive Move (m n : ℕ) where
  | pass : Move m n
  | play : Point m n → Move m n

/-- The situation a move leads to. Legality is a separate question; this is the
board mechanics alone. -/
def Situation.after {m n : ℕ} (s : Situation m n) : Move m n → Situation m n
  | .pass => ⟨s.board, s.toMove.other⟩
  | .play p => ⟨resolve s.board s.toMove p, s.toMove.other⟩

/-- A **state of play**. -/
structure State (m n : ℕ) where
  /-- The situation now. -/
  now : Situation m n
  /-- Every situation that has occurred, `now` included. The repetition rules
  read this; it is the history that makes superko history-dependent. -/
  seen : Set (Situation m n)
  /-- How many passes immediately precede. Two ends the game. -/
  passes : ℕ

/-- The state in which play begins from a given position.

This is encoding (C) of `docs/formal-model.md` §5: the position is the root of
play and nothing is forbidden yet. The alternatives — supplying a history, or
supplying the record that reached the position — give different problems, and
claim C-1 is the question of which one the classical statements intend. -/
def start {m n : ℕ} (b : Position m n) (c : Color) : State m n :=
  { now := ⟨b, c⟩, seen := {⟨b, c⟩}, passes := 0 }

/-- The state after a move. Total: applying it to an illegal move gives a
state, which no rule below reaches. -/
def step {m n : ℕ} (st : State m n) (mv : Move m n) : State m n :=
  { now := st.now.after mv
    seen := insert (st.now.after mv) st.seen
    passes := match mv with | .pass => st.passes + 1 | .play _ => 0 }

/-- The game has **ended**: two consecutive passes. -/
def Ended {m n : ℕ} (st : State m n) : Prop := 2 ≤ st.passes

/-! ## Repetition rules

A repetition rule says which moves a state permits. Two are defined; the
decision problem uses situational superko, and positional superko is here
because the project compares them (claims C-8, C-12, C-17) and because the
published count this project validates against (C-9) is a PSK count. -/

/-- A repetition rule: which moves are legal in which states. -/
abbrev Repetition (m n : ℕ) : Type := State m n → Move m n → Prop

/-- The board conditions every play must meet, whatever the repetition rule:
the point is empty, and the resulting chain has a liberty. The second conjunct
is the prohibition on suicide. -/
def PlayableAt {m n : ℕ} (st : State m n) (p : Point m n) : Prop :=
  st.now.board p = none ∧
    HasLiberty (resolve st.now.board st.now.toMove p) p

/-- **Situational superko** — AGA Rule 6. A play may not recreate a situation
that has occurred: a position with the same player to move.

A pass is always legal, even when the situation it leads to has occurred. That
reading is OPEN-1 in `docs/formal-model.md` and claim C-18.

The difference from `PSK` does not rest on that reading. Of the plays meeting
`PlayableAt`, SSK permits and PSK refuses exactly those whose position has
stood with the mover to move and never with the opponent to move, and such
plays occur in games with no pass (C-52, `proved` by hand in `proofs/C-52.md`,
and see `docs/formal-model.md` §3). -/
def SSK {m n : ℕ} : Repetition m n
  | _, .pass => True
  | st, .play p => PlayableAt st p ∧ st.now.after (.play p) ∉ st.seen

/-- **Positional superko**. A play may not recreate a position that has
occurred, whoever was to move. -/
def PSK {m n : ℕ} : Repetition m n
  | _, .pass => True
  | st, .play p =>
      PlayableAt st p ∧ ∀ s ∈ st.seen, s.board ≠ (st.now.after (.play p)).board

/-! ## Score -/

/-- An empty point **reaches** color `c` when a path of adjacent empty points
leads from it to a `c` stone. -/
def Reaches {m n : ℕ} (b : Position m n) (p : Point m n) (c : Color) : Prop :=
  ∃ q, b q = some c ∧
    Relation.ReflTransGen (fun x y => Adj x y ∧ b x = none) p q

/-- **Area score**: stones of the color, plus empty points reaching that
color and not the other.

This is mechanical scoring in the Tromp–Taylor formulation. There is no
dead-stone determination — the AGA procedure of agreement and resumption is
not modeled, and claim C-16 is the argument that under superko and optimal
play the two agree. -/
def area {m n : ℕ} (b : Position m n) (c : Color) : ℕ :=
  {p | b p = some c ∨
        (b p = none ∧ Reaches b p c ∧ ¬ Reaches b p c.other)}.ncard

/-- Black's margin: Black's area less White's, less komi. -/
def margin {m n : ℕ} (b : Position m n) (komi : ℚ) : ℚ :=
  (area b .black : ℚ) - (area b .white : ℚ) - komi

/-- The winner at a finished game. A tie goes to White; with half-integer komi
no tie arises, and the convention is invisible (OPEN-3). -/
def winner {m n : ℕ} (b : Position m n) (komi : ℚ) : Color :=
  if 0 < margin b komi then .black else .white

/-! ## Winning -/

/-- **`WinsFor L komi c st`: color `c` has a winning strategy from `st`** under
repetition rule `L`.

Three ways to win: the game has ended in `c`'s favor; it is `c`'s turn and some
legal move leads to a won state; it is the opponent's turn and every legal move
leads to a won state.

Stated for a named color rather than for "the player to move" because the
latter needs `¬ WinsFor` in a premise, which Lean's strict positivity check
rejects. That exactly one color wins from each state is determinacy — a
theorem, not part of this definition, and it rests on termination (C-13). -/
inductive WinsFor (m n : ℕ) (L : Repetition m n) (komi : ℚ) (c : Color) :
    State m n → Prop where
  /-- The game is over and `c` has won. -/
  | ended {st} : Ended st → winner st.now.board komi = c → WinsFor m n L komi c st
  /-- It is `c`'s turn, and some legal move leads to a state `c` wins. -/
  | mover {st mv} : ¬ Ended st → st.now.toMove = c → L st mv →
      WinsFor m n L komi c (step st mv) → WinsFor m n L komi c st
  /-- It is the opponent's turn, and every legal move leads to a state `c` wins. -/
  | waiter {st} : ¬ Ended st → st.now.toMove ≠ c →
      (∀ mv, L st mv → WinsFor m n L komi c (step st mv)) →
      WinsFor m n L komi c st

/-! ## The decision problem -/

/-- **SUPERKO-GO.** Given a position on the `m × n` board and a komi, does
Black — to move, from this position as the root of play — have a winning
strategy under AGA rules?

The classification of this predicate is the object of the project. -/
def BlackWins (m n : ℕ) (b : Position m n) (komi : ℚ) : Prop :=
  WinsFor m n SSK komi .black (start b .black)

/-- The same question under positional superko, for comparison (C-12). -/
def BlackWinsPSK (m n : ℕ) (b : Position m n) (komi : ℚ) : Prop :=
  WinsFor m n PSK komi .black (start b .black)

end

end Superko
