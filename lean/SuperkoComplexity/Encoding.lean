/-
Copyright (c) 2026 Joseph J. Piché. All rights reserved.
Released under Apache 2.0 license as described in the file LICENSE.
Authors: Joseph J. Piché
-/
import SuperkoComplexity.Basic
import Mathlib.Data.Nat.Size
import Mathlib.Data.Nat.Prime.Basic
import Mathlib.Data.Rat.Lemmas
import Mathlib.Data.List.OfFn
import Mathlib.Tactic.Ring
import Mathlib.Tactic.Linarith
import Mathlib.Tactic.NormNum
import Mathlib.Tactic.Push

/-!
# The input encoding (C-31)

SUPERKO-GO is a predicate on `m`, `n`, a color to move, a `Position m n` and a
`komi : ℚ`. A complexity claim is a claim about a set of strings. This file is
the bridge: `enc` writes an instance as a bit string, `dec` reads it back, and
`dec_enc` says the round trip is the identity — so `enc` is injective and the
language `goLang` below has one instance per string in it.

## What the size bounds are for

Two directions, consumed by different claims, and they are not
interchangeable:

* `C31_enc_length_lower : 2 * (m * n) ≤ (enc …).length` is what **membership**
  consumes. The configuration bound is stated in terms of `3 ^ (m * n)`
  (`Superko.card_situation`, and `4 * 3 ^ (m * n)` in C-26); only the lower
  bound makes that *singly* exponential in the input length. The consumer-facing
  forms are `card_situation_le_two_pow_length`, `card_situation_mul_le` and
  `lengthBound_le_two_pow`. `sparse_encoding_refuted` is the same statement for
  a stone-list encoding, and under that encoding it fails at one instance, by
  `decide`: the empty 4×4 board encodes in twenty-four bits, and an archive of
  `2 · 3 ^ 16` situations does not fit in `2 ^ |w|` there. What that settles is
  the archive's `2 ^ |w|` budget at that single instance. Whether the language
  lies outside EXPSPACE under a stone-list encoding is not established here, and
  reading the one instance as a statement about the asymptotics is an inference
  this file does not make.

* `enc_length_upper` (and the sharper `length_enc`, an equation) is what
  **hardness** consumes: a reduction must show its output is polynomially
  bounded in its input.

## Choices this file makes

* **Row-major, explicitly.** Flat index `k` is row `k / n`, column `k % n`;
  `cellFlat_idx` is the statement that the point `(i, j)` sits at `i * n + j`.
  Neither `Fintype.equivFin` nor `Encodable` is used. `Fintype.equivFin` states
  no order at all. `finProdFinEquiv` is in fact row-major at Mathlib `f5e9087`
  (`finProdFinEquiv_apply_val : ↑(finProdFinEquiv x) = ↑x.2 + n * ↑x.1`, a
  `@[simps]`-generated lemma), but neither its name nor its docstring says so and
  nothing here would pin it, so a reader would have to read Mathlib's source and
  a refactor could flip it silently. `cellFlat_idx` pins the order inside this
  project instead. `Encodable` pairs: `Nat.max_sq_add_min_le_pair`
  (`Mathlib/Data/Nat/Pairing.lean:139` at Mathlib `f5e9087`) gives
  `max m n ^ 2 + min m n ≤ Nat.pair m n`, so pairing squares magnitudes and a
  code nested once per point has bit length exponential in `m * n` — which
  breaks the *upper* bound, and with it any reduction's output-size claim.

* **Komi is encoded exactly**, as a sign bit, `|num|` and `den`. A sign is
  required: a pair of naturals cannot represent negative komi, and `BlackWins`
  admits it. Encoding the *exact* rational rather than C-27's clamped `⌊komi⌋`
  is what keeps `dec_enc` an identity and keeps this file independent of C-27.
  C-27 then does what it is, which is to license a reduction's *choice* of a
  half-integer komi: `length_enc_halfInteger` is the size bound on that range.

* **The color to move is carried**, in one bit, as `docs/formal-model.md` §7
  says the input is: the question is whether *Black* has a winning strategy, and
  the color to move is part of the instance. `BlackWinsFrom … .black` is
  `BlackWins` by `Iff.rfl`, so this costs no change to `Defs.lean`. The
  alternative — drop the bit and normalize White-to-move instances by swapping
  colors and negating komi — needs C-27 first: `winner_swap_iff` says the swap
  identity holds exactly when the margin is non-zero, and
  `margin_ne_zero_halfInteger` says a half-integer komi is what makes it
  non-zero.

## What a reader must audit

Per `docs/trusted-base.md`, the proofs below are kernel-checked and need no
review; the definitions are what carry meaning. They are `enc` and its parts —
`sd` and `dbl`, which write a self-delimiting natural; `colorBit`, the color to
move; `boardBits` with `cellFlat`, `occBit` and `colBit`, which write the cell
array; and `komiBits`, which writes the komi — together with `BlackWinsFrom`,
which fixes *which question* a string asks, and `goLang`, the string set itself.
Three statements pin the layout those definitions claim: `cellFlat_idx`
(row-major: the point `(i, j)` is at flat index `i * n + j`), `boardBits_even`
(cell `k`'s occupancy bit is at position `2 * k`) and `boardBits_odd` (its color
bit is at `2 * k + 1`). A reader who accepts those definitions and checks those
three statements has checked this file. It adds nothing to `Defs.lean`.
-/

namespace Superko.Enc

open Superko

/-! ## Naturals, least significant bit first

`Nat.bits` is Mathlib's little-endian binary digit list with no trailing
`false`, and `Nat.size_eq_bits_len` says its length is `Nat.size`, Mathlib's
`⌈log₂ (k+1)⌉`. Only the inverse is missing, so only the inverse is defined
here. -/

/-- The natural a little-endian bit list denotes. -/
def natOf : List Bool → ℕ
  | [] => 0
  | d :: ds => (if d then 1 else 0) + 2 * natOf ds

example : Nat.bits 6 = [false, true, true] := by decide
example : natOf [false, true, true] = 6 := by decide

theorem natOf_bits (n : ℕ) : natOf n.bits = n := by
  induction n using Nat.strong_induction_on with
  | _ n ih =>
    match n with
    | 0 => simp [natOf]
    | (k + 1) =>
      rcases Nat.even_or_odd (k + 1) with he | ho
      · obtain ⟨j, hj⟩ := he
        have h2 : k + 1 = 2 * j := by omega
        have hj0 : j ≠ 0 := by omega
        rw [h2, Nat.bit0_bits j hj0, natOf, ih j (by omega)]
        simp
      · obtain ⟨j, hj⟩ := ho
        rw [hj, Nat.bit1_bits j, natOf, ih j (by omega)]
        simp; omega

/-! ## Self-delimiting naturals

Each binary digit is written twice and the number is closed by `false, true`,
the one two-bit block a doubled digit cannot be. -/

/-- Write each bit twice. -/
def dbl : List Bool → List Bool
  | [] => []
  | d :: ds => d :: d :: dbl ds

/-- A natural, self-delimiting: doubled binary digits, then the closer `01`. -/
def sd (k : ℕ) : List Bool := dbl (Nat.bits k) ++ [false, true]

/-- Read one self-delimiting natural off the front, with what is left. -/
def readSd : List Bool → Option (ℕ × List Bool)
  | false :: true :: rest => some (0, rest)
  | a :: b :: rest =>
      if a = b then (readSd rest).map (fun p => ((if a then 1 else 0) + 2 * p.1, p.2))
      else none
  | _ => none

example : sd 6 = [false, false, true, true, true, true, false, true] := by decide
example : readSd (sd 6 ++ [true]) = some (6, [true]) := by decide

theorem length_dbl (ds : List Bool) : (dbl ds).length = 2 * ds.length := by
  induction ds with
  | nil => rfl
  | cons d ds ih => simp [dbl, ih]; omega

theorem length_sd (k : ℕ) : (sd k).length = 2 * Nat.size k + 2 := by
  simp [sd, length_dbl, Nat.size_eq_bits_len]

theorem readSd_append (ds tail : List Bool) :
    readSd (dbl ds ++ (false :: true :: tail)) = some (natOf ds, tail) := by
  induction ds with
  | nil => simp [dbl, readSd, natOf]
  | cons d ds ih =>
      cases d
      · simpa [dbl, readSd, natOf] using ih
      · simpa [dbl, readSd, natOf] using ih

/-- The parser inverts the writer and leaves the tail untouched. -/
theorem readSd_sd (k : ℕ) (tail : List Bool) :
    readSd (sd k ++ tail) = some (k, tail) := by
  rw [sd, List.append_assoc]
  simpa [natOf_bits] using readSd_append (Nat.bits k) tail

/-- Reading a self-delimiting natural consumes at least its two closing bits.
Stated with an explicit bound `N` on the list's length so that the recursion is
structural; `readSd_len` is the form every caller wants. -/
theorem readSd_length_lt : ∀ (N : ℕ) (l : List Bool), l.length ≤ N →
    ∀ {k : ℕ} {rest : List Bool}, readSd l = some (k, rest) → rest.length + 2 ≤ l.length := by
  intro N
  induction N with
  | zero =>
      intro l hl k rest h
      match l with
      | [] => simp [readSd] at h
      | a :: t => simp at hl
  | succ N ih =>
      intro l hl k rest h
      match l with
      | [] => simp [readSd] at h
      | [a] => cases a <;> simp [readSd] at h
      | a :: b :: t =>
        have ht : t.length ≤ N := by simp at hl; omega
        cases a <;> cases b
        · cases hp : readSd t with
          | none => simp [readSd, hp] at h
          | some p =>
              have hih := ih t ht hp
              simp [readSd, hp] at h
              obtain ⟨-, hr⟩ := h
              subst hr
              simp
              omega
        · simp [readSd] at h
          obtain ⟨-, hr⟩ := h
          subst hr
          simp
        · simp [readSd] at h
        · cases hp : readSd t with
          | none => simp [readSd, hp] at h
          | some p =>
              have hih := ih t ht hp
              simp [readSd, hp] at h
              obtain ⟨-, hr⟩ := h
              subst hr
              simp
              omega

/-- A parsed self-delimiting natural leaves a strictly shorter tail — two bits
shorter at least, which is what bounds the dimensions a short string can
declare. -/
theorem readSd_len {l : List Bool} {k : ℕ} {rest : List Bool}
    (h : readSd l = some (k, rest)) : rest.length + 2 ≤ l.length :=
  readSd_length_lt l.length l le_rfl h

/-! ## Cells -/

/-- Bit 0 of a cell: is there a stone? -/
def occBit : Option Color → Bool
  | none => false
  | some _ => true

/-- Bit 1 of a cell: is the stone white? `false` at an empty point, where no
lemma below reads it. -/
def colBit : Option Color → Bool
  | some .white => true
  | _ => false

/-- The cell two bits record. -/
def cellOf (o w : Bool) : Option Color :=
  if o then some (if w then Color.white else Color.black) else none

theorem cellOf_bits (x : Option Color) : cellOf (occBit x) (colBit x) = x := by
  cases x with
  | none => rfl
  | some c => cases c <;> rfl

/-! ## The board, in row-major order -/

/-- The cell at flat **row-major** index `k`: row `k / n`, column `k % n`. An
out-of-range index gives `none`; no lemma below consults one. -/
def cellFlat {m n : ℕ} (b : Position m n) (k : ℕ) : Option Color :=
  if h : k / n < m ∧ k % n < n then b (⟨k / n, h.1⟩, ⟨k % n, h.2⟩) else none

/-- **Bridge.** **Row-major, explicitly.** The point in row `i`, column `j` is
at flat index `i * n + j`: the computable `cellFlat` agrees there with the
`Position` of `Defs.lean`. -/
theorem cellFlat_idx {m n : ℕ} (b : Position m n) (i : Fin m) (j : Fin n) :
    cellFlat b (i.val * n + j.val) = b (i, j) := by
  have hn : 0 < n := j.pos
  have hcomm : i.val * n + j.val = n * i.val + j.val := by ring
  have hdiv : (i.val * n + j.val) / n = i.val := by
    rw [hcomm, Nat.mul_add_div hn, Nat.div_eq_of_lt j.isLt, Nat.add_zero]
  have hmod : (i.val * n + j.val) % n = j.val := by
    rw [hcomm, Nat.mul_add_mod, Nat.mod_eq_of_lt j.isLt]
  rw [cellFlat, dite_eq_left ⟨by rw [hdiv]; exact i.isLt, by rw [hmod]; exact j.isLt⟩]
  refine congrArg b ?_
  simp only [Prod.mk.injEq]
  exact ⟨Fin.ext_iff.mpr hdiv, Fin.ext_iff.mpr hmod⟩

theorem idx_lt {m n : ℕ} (i : Fin m) (j : Fin n) : i.val * n + j.val < m * n := by
  have h1 : i.val + 1 ≤ m := i.isLt
  have h2 : j.val < n := j.isLt
  calc i.val * n + j.val < i.val * n + n := by omega
    _ = (i.val + 1) * n := by ring
    _ ≤ m * n := Nat.mul_le_mul_right n h1

/-- The board as `2 · m · n` bits: the cells in row-major order, two bits each,
occupied then white. -/
def boardBits {m n : ℕ} (b : Position m n) : List Bool :=
  List.ofFn fun t : Fin (2 * (m * n)) =>
    if t.val % 2 = 0 then occBit (cellFlat b (t.val / 2))
    else colBit (cellFlat b (t.val / 2))

theorem length_boardBits {m n : ℕ} (b : Position m n) :
    (boardBits b).length = 2 * (m * n) := List.length_ofFn

/-- The cell a bit list records at flat index `k`. -/
def cellAt (l : List Bool) (k : ℕ) : Option Color :=
  match l[2 * k]?, l[2 * k + 1]? with
  | some o, some w => cellOf o w
  | _, _ => none

/-- The position a bit list records, read in row-major order. -/
def boardOf (m n : ℕ) (l : List Bool) : Position m n :=
  fun p => cellAt l (p.1.val * n + p.2.val)

/-- **The occupancy bit of cell `k` sits at position `2 * k`.** -/
theorem boardBits_even {m n : ℕ} (b : Position m n) (k : ℕ) (hk : k < m * n) :
    (boardBits b)[2 * k]? = some (occBit (cellFlat b k)) := by
  have h : 2 * k < 2 * (m * n) := by omega
  rw [boardBits, List.getElem?_ofFn, dite_eq_left h]
  simp [Nat.mul_mod_right, Nat.mul_div_cancel_left k (by omega : 0 < 2)]

/-- **The color bit of cell `k` sits at position `2 * k + 1`.** -/
theorem boardBits_odd {m n : ℕ} (b : Position m n) (k : ℕ) (hk : k < m * n) :
    (boardBits b)[2 * k + 1]? = some (colBit (cellFlat b k)) := by
  have h : 2 * k + 1 < 2 * (m * n) := by omega
  rw [boardBits, List.getElem?_ofFn, dite_eq_left h]
  have h1 : (2 * k + 1) % 2 = 1 := by omega
  have h2 : (2 * k + 1) / 2 = k := by omega
  simp [h1, h2]

theorem cellAt_boardBits {m n : ℕ} (b : Position m n) (k : ℕ) (hk : k < m * n) :
    cellAt (boardBits b) k = cellFlat b k := by
  simp only [cellAt, boardBits_even b k hk, boardBits_odd b k hk, cellOf_bits]

/-- **Bridge.** **The board round trip.** Writing a `Position` of `Defs.lean` as
bits and reading it back gives that position, point for point. -/
theorem boardOf_boardBits {m n : ℕ} (b : Position m n) :
    boardOf m n (boardBits b) = b := by
  funext p
  obtain ⟨i, j⟩ := p
  rw [boardOf, cellAt_boardBits b _ (idx_lt i j), cellFlat_idx]

/-! ## Komi

Lean's `Rat` is a reduced fraction with a signed numerator, so three fields are
written: a sign bit, `|num|`, and `den`. A pair of naturals would not do —
`BlackWins` admits negative komi and hardness constructions use it. -/

/-- A rational, exactly: sign, then `|num|`, then `den`. -/
def komiBits (q : ℚ) : List Bool :=
  decide (q.num < 0) :: (sd q.num.natAbs ++ sd q.den)

/-- Read a komi off a bit list, requiring the list to be exactly consumed. -/
def readKomi (l : List Bool) : Option ℚ :=
  match l with
  | [] => none
  | s :: rest =>
    match readSd rest with
    | none => none
    | some (a, rest₁) =>
      match readSd rest₁ with
      | none => none
      | some (d, rest₂) =>
        match rest₂ with
        | [] => some ((if s then -(a : ℚ) else (a : ℚ)) / (d : ℚ))
        | _ => none

theorem length_komiBits (q : ℚ) :
    (komiBits q).length = 2 * Nat.size q.num.natAbs + 2 * Nat.size q.den + 5 := by
  simp [komiBits, length_sd]
  omega

/-- A komi occupies at least the sign bit: the empty string carries none. -/
theorem readKomi_ne_nil {l : List Bool} {q : ℚ} (h : readKomi l = some q) : l ≠ [] := by
  intro hnil; subst hnil; simp [readKomi] at h

/-- **The komi round trip.** -/
theorem readKomi_komiBits (q : ℚ) : readKomi (komiBits q) = some q := by
  have h1 : readSd (sd q.num.natAbs ++ sd q.den) = some (q.num.natAbs, sd q.den) :=
    readSd_sd _ _
  have h2 : readSd (sd q.den) = some (q.den, []) := by
    simpa using readSd_sd q.den []
  have e : ((q.num.natAbs : ℕ) : ℚ) = ((q.num.natAbs : ℤ) : ℚ) :=
    (Int.cast_natCast q.num.natAbs).symm
  have hnum : (if q.num < 0 then -((q.num.natAbs : ℕ) : ℚ) else ((q.num.natAbs : ℕ) : ℚ))
      = (q.num : ℚ) := by
    by_cases h : q.num < 0
    · have h' : (q.num.natAbs : ℤ) = -q.num := by omega
      rw [ite_eq_left h, e, h']; push_cast; ring
    · have h' : (q.num.natAbs : ℤ) = q.num := by omega
      rw [ite_eq_right h, e, h']
  simp only [komiBits, readKomi, h1, h2, decide_eq_true_eq]
  rw [hnum]
  exact congrArg some (by exact_mod_cast Rat.num_div_den q)

/-! ## The color to move -/

/-- Black is `false`, White `true`. -/
def colorBit : Color → Bool
  | .black => false
  | .white => true

/-- The color a bit records: `false` is Black, `true` is White. -/
def colorOf : Bool → Color
  | false => .black
  | true => .white

theorem colorOf_colorBit (c : Color) : colorOf (colorBit c) = c := by cases c <;> rfl

/-! ## The encoding -/

/-- **The input encoding.** Self-delimiting `m`, self-delimiting `n`, the color
to move in one bit, the board in `2 · m · n` bits in row-major order, then the
komi. -/
def enc (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) : List Bool :=
  sd m ++ (sd n ++ (colorBit c :: (boardBits b ++ komiBits komi)))

/-- **The decoding.** Dependently typed because `Position m n` depends on the
dimensions the string itself carries. -/
def dec (l : List Bool) : Option (Σ m n : ℕ, Color × Position m n × ℚ) :=
  match readSd l with
  | none => none
  | some (m, l₁) =>
    match readSd l₁ with
    | none => none
    | some (n, l₂) =>
      match l₂ with
      | [] => none
      | cb :: l₃ =>
        match readKomi (l₃.drop (2 * (m * n))) with
        | none => none
        | some komi =>
            some ⟨m, n, colorOf cb, boardOf m n (l₃.take (2 * (m * n))), komi⟩

/-- A 1×1 empty board, Black to move, komi 0 — eighteen bits, kernel-checked.
Reading left to right: `m = 1`, `n = 1`, Black, one empty cell, komi `0/1`. -/
example : enc 1 1 .black (fun _ => none) 0 =
    [true, true, false, true,
     true, true, false, true,
     false,
     false, false,
     false, false, true, true, true, false, true] := by decide

/-- Row-major, on a board a reader can check by hand: a single black stone at
row 1, column 2 of a 2×3 board occupies bits `2 * (1 * 3 + 2) = 10` and `11`. -/
example :
    boardBits (m := 2) (n := 3) (fun p => if p = (1, 2) then some Color.black else none)
      = [false, false, false, false, false, false, false, false, false, false, true, false] := by
  decide

/-- White to move on a 2×3 board with one black stone and komi `-3`:
thirty-six bits, which is what `length_enc` predicts
(`2·6 + 2·size 2 + 2·size 3 + 2·size 3 + 2·size 1 + 10`). (A komi needing
normalization, `-5/2`, the kernel will not reduce: `Nat.gcd` gets stuck, so
`length_enc` is the only witness available for that one.) -/
example :
    (enc 2 3 .white (fun p => if p = (0, 1) then some Color.black else none) (-3)).length
      = 36 := by decide

/-- The komi C-27 makes canonical is a half-integer, and the kernel does not
reduce `Rat` division — `Rat.den ((5 : ℚ)/(2 : ℚ))` gets stuck under `decide` —
so a kernel-checked example writes the komi with the reduced-fraction
constructor. Komi `1/2` on the empty 1×1 board: twenty-two bits.
`native_decide` is not an option here and is not used. -/
example :
    (enc 1 1 .black (fun _ => none) (Rat.mk' 1 2 (by decide) (by decide))).length = 22 := by
  decide

/-- **The round trip.** -/
theorem dec_enc (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) :
    dec (enc m n c b komi) = some ⟨m, n, c, b, komi⟩ := by
  have htake : (boardBits b ++ komiBits komi).take (2 * (m * n)) = boardBits b :=
    List.take_left' (length_boardBits b)
  have hdrop : (boardBits b ++ komiBits komi).drop (2 * (m * n)) = komiBits komi :=
    List.drop_left' (length_boardBits b)
  simp only [enc, dec, readSd_sd, htake, hdrop, readKomi_komiBits,
    boardOf_boardBits, colorOf_colorBit]

/-- `enc` recovers the dimensions. -/
theorem enc_inj_dims {m₁ n₁ m₂ n₂ : ℕ} {c₁ c₂ : Color} {b₁ : Position m₁ n₁}
    {b₂ : Position m₂ n₂} {k₁ k₂ : ℚ}
    (h : enc m₁ n₁ c₁ b₁ k₁ = enc m₂ n₂ c₂ b₂ k₂) : m₁ = m₂ ∧ n₁ = n₂ := by
  have h1 : dec (enc m₁ n₁ c₁ b₁ k₁) = dec (enc m₂ n₂ c₂ b₂ k₂) := by rw [h]
  rw [dec_enc, dec_enc] at h1
  simp only [Option.some.injEq, Sigma.mk.injEq] at h1
  obtain ⟨hm, h1⟩ := h1
  subst hm
  simp only [heq_eq_eq, Sigma.mk.injEq] at h1
  exact ⟨rfl, h1.1⟩

/-- `enc` is injective at fixed dimensions — the form a reduction uses. -/
theorem enc_inj {m n : ℕ} {c₁ c₂ : Color} {b₁ b₂ : Position m n} {k₁ k₂ : ℚ}
    (h : enc m n c₁ b₁ k₁ = enc m n c₂ b₂ k₂) : c₁ = c₂ ∧ b₁ = b₂ ∧ k₁ = k₂ := by
  have h1 : dec (enc m n c₁ b₁ k₁) = dec (enc m n c₂ b₂ k₂) := by rw [h]
  rw [dec_enc, dec_enc] at h1
  simp only [Option.some.injEq, Sigma.mk.injEq, heq_eq_eq, Prod.mk.injEq, true_and] at h1
  exact ⟨h1.1, h1.2.1, h1.2.2⟩

/-- **One string, one instance.** Two encodings agree only if the dimensions,
the color to move and the komi agree — so membership in `goLang` determines the
instance up to the board, which `enc_inj` then pins at fixed dimensions. -/
theorem enc_inj_full {m n m' n' : ℕ} {c c' : Color} {b : Position m n}
    {b' : Position m' n'} {komi komi' : ℚ}
    (h : enc m n c b komi = enc m' n' c' b' komi') :
    m = m' ∧ n = n' ∧ c = c' ∧ komi = komi' := by
  obtain ⟨hm, hn⟩ := enc_inj_dims h
  subst hm; subst hn
  obtain ⟨hc, -, hk⟩ := enc_inj h
  exact ⟨rfl, rfl, hc, hk⟩

/-! ## Komi normalization, arithmetically

C-27 says `winner` depends on komi only through `⌊komi⌋` clamped to
`[-(m·n)-1, m·n]`, so every instance is equivalent to one with a half-integer
komi. These two facts are what that range means in the encoding: the
half-integer just above `z` is the reduced fraction `(2z+1)/2`. -/

/-- The half-integer just above `z`, in lowest terms: numerator `2z + 1`. -/
theorem num_halfInteger (z : ℤ) : ((z : ℚ) + 1/2).num = 2 * z + 1 := by
  have hco : Nat.Coprime (2 * z + 1).natAbs 2 := by
    rw [Nat.coprime_two_right, Int.natAbs_odd]
    exact ⟨z, by ring⟩
  have hq : ((z : ℚ) + 1/2) = ((2 * z + 1 : ℤ) : ℚ) / ((2 : ℤ) : ℚ) := by
    push_cast; ring
  rw [hq, Rat.num_div_eq_of_coprime (by norm_num) (by exact hco)]

/-- Denominator `2`. -/
theorem den_halfInteger (z : ℤ) : ((z : ℚ) + 1/2).den = 2 := by
  have hco : Nat.Coprime (2 * z + 1).natAbs 2 := by
    rw [Nat.coprime_two_right, Int.natAbs_odd]
    exact ⟨z, by ring⟩
  have hq : ((z : ℚ) + 1/2) = ((2 * z + 1 : ℤ) : ℚ) / ((2 : ℤ) : ℚ) := by
    push_cast; ring
  have := Rat.den_div_eq_of_coprime (a := 2 * z + 1) (b := 2) (by norm_num) (by exact hco)
  rw [hq]
  omega

/-! ## Size -/

/-- **The length, exactly.** -/
theorem length_enc (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) :
    (enc m n c b komi).length =
      2 * (m * n) + 2 * Nat.size m + 2 * Nat.size n
        + 2 * Nat.size komi.num.natAbs + 2 * Nat.size komi.den + 10 := by
  simp only [enc, List.length_append, List.length_cons, length_sd, length_boardBits,
    length_komiBits]
  omega

/-- **The lower bound — what membership consumes (C-31).** Two bits per point,
so the board size is linear in the input length and `3 ^ (m * n)` is singly
exponential in it. -/
theorem C31_enc_length_lower (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) :
    2 * (m * n) ≤ (enc m n c b komi).length := by
  rw [length_enc]; omega

/-- **The upper bound — what hardness consumes.** Everything beyond the board
is logarithmic in the dimensions and in the komi. -/
theorem enc_length_upper (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) :
    (enc m n c b komi).length ≤
      2 * (m * n) + 2 * (Nat.size m + Nat.size n
        + Nat.size komi.num.natAbs + Nat.size komi.den) + 10 := by
  rw [length_enc]; omega

theorem size_le_self (k : ℕ) : Nat.size k ≤ k :=
  Nat.size_le.mpr (Nat.lt_two_pow_self)

/-- On the komi range C-27 leaves — a half-integer with `|⌊komi⌋|` at most
`m * n + 1` — the encoding is `2·m·n + O(log m + log n + log (m·n))`. -/
theorem length_enc_halfInteger (m n : ℕ) (c : Color) (b : Position m n) (z : ℤ)
    (hz : z.natAbs ≤ m * n + 1) :
    (enc m n c b ((z : ℚ) + 1/2)).length ≤
      2 * (m * n) + 2 * Nat.size m + 2 * Nat.size n
        + 2 * Nat.size (2 * (m * n) + 3) + 14 := by
  have hnum : ((z : ℚ) + 1/2).num = 2 * z + 1 := num_halfInteger z
  have hden : ((z : ℚ) + 1/2).den = 2 := den_halfInteger z
  have habs : (2 * z + 1).natAbs ≤ 2 * (m * n) + 3 := by omega
  have hsz : Nat.size (2 * z + 1).natAbs ≤ Nat.size (2 * (m * n) + 3) :=
    Nat.size_le_size habs
  rw [length_enc, hnum, hden]
  have : Nat.size 2 = 2 := by decide
  omega

/-- The same bound with every logarithm discharged, for a reader who wants a
polynomial and no `Nat.size`. -/
theorem length_enc_halfInteger_poly (m n : ℕ) (c : Color) (b : Position m n) (z : ℤ)
    (hz : z.natAbs ≤ m * n + 1) :
    (enc m n c b ((z : ℚ) + 1/2)).length ≤ 6 * (m * n) + 2 * m + 2 * n + 20 := by
  have h := length_enc_halfInteger m n c b z hz
  have h1 := size_le_self m
  have h2 := size_le_self n
  have h3 := size_le_self (2 * (m * n) + 3)
  omega

/-! ## What the EXPSPACE sentence consumes

The archive argument needs the set of situations, and the game-length bound of
C-26, to be singly exponential in the *input length*. That is exactly the lower
bound above, and these statements are the form the prose uses. -/

theorem four_pow_eq (k : ℕ) : 4 ^ k = 2 ^ (2 * k) := by
  rw [pow_mul 2 2 k]; norm_num

/-- `3 ^ (m * n) ≤ 4 ^ (m * n) = 2 ^ (2 * m * n)`, and the board contributes
`2 * m * n` bits. -/
theorem three_pow_le_two_pow_length (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) :
    3 ^ (m * n) ≤ 2 ^ (enc m n c b komi).length := by
  calc 3 ^ (m * n) ≤ 4 ^ (m * n) := Nat.pow_le_pow_left (by norm_num) _
    _ = 2 ^ (2 * (m * n)) := four_pow_eq (m * n)
    _ ≤ 2 ^ (enc m n c b komi).length :=
        Nat.pow_le_pow_right (by norm_num) (C31_enc_length_lower m n c b komi)

/-- **The situation count is at most `2 ^ |w|`.** `Superko.card_situation` is
`2 * 3 ^ (m * n)`; the slack in `enc` absorbs the factor of two. -/
theorem card_situation_le_two_pow_length (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) :
    Fintype.card (Situation m n) ≤ 2 ^ (enc m n c b komi).length := by
  have hlen : 2 * (m * n) + 1 ≤ (enc m n c b komi).length := by
    rw [length_enc]; omega
  calc Fintype.card (Situation m n) = 2 * 3 ^ (m * n) := card_situation m n
    _ ≤ 2 * 4 ^ (m * n) := by
        have := Nat.pow_le_pow_left (show 3 ≤ 4 by norm_num) (m * n); omega
    _ = 2 ^ (2 * (m * n) + 1) := by rw [pow_succ', four_pow_eq]
    _ ≤ 2 ^ (enc m n c b komi).length := Nat.pow_le_pow_right (by norm_num) hlen

/-- **The archive fits in `2 ^ (2 · |w|)` bits.** One archive entry is a
situation written as the board plus the color to move, `2 · m · n + 1` bits
under this encoding, and there are `Fintype.card (Situation m n)` entries. The
product is at most `2 ^ (2 · |w|)`, so an archive is exponential — singly — in
the input length, entry count and entry width together. C-32's prose consumes
this form; the bound is loose, since `2 · m · n + 1 ≤ |w| ≤ 2 ^ |w|` is all it
uses about the entry width. -/
theorem card_situation_mul_le (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) :
    Fintype.card (Situation m n) * (2 * (m * n) + 1)
      ≤ 2 ^ (2 * (enc m n c b komi).length) := by
  have h1 : Fintype.card (Situation m n) ≤ 2 ^ (enc m n c b komi).length :=
    card_situation_le_two_pow_length m n c b komi
  have h2 : 2 * (m * n) + 1 ≤ (enc m n c b komi).length := by
    rw [length_enc]; omega
  have h3 : 2 * (m * n) + 1 ≤ 2 ^ (enc m n c b komi).length :=
    le_trans h2 (Nat.le_of_lt Nat.lt_two_pow_self)
  have h4 : 2 ^ (2 * (enc m n c b komi).length)
      = 2 ^ (enc m n c b komi).length * 2 ^ (enc m n c b komi).length := by
    rw [two_mul, pow_add]
  rw [h4]
  exact Nat.mul_le_mul h1 h3

/-- **C-26's game length is at most `2 ^ |w|`.** -/
theorem lengthBound_le_two_pow (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) :
    4 * 3 ^ (m * n) ≤ 2 ^ (enc m n c b komi).length := by
  have hlen : 2 * (m * n) + 2 ≤ (enc m n c b komi).length := by
    rw [length_enc]; omega
  calc 4 * 3 ^ (m * n) ≤ 4 * 4 ^ (m * n) := by
        have := Nat.pow_le_pow_left (show 3 ≤ 4 by norm_num) (m * n); omega
    _ = 2 ^ (2 * (m * n) + 2) := by
        rw [pow_add, four_pow_eq]; norm_num; exact Nat.mul_comm _ _
    _ ≤ 2 ^ (enc m n c b komi).length := Nat.pow_le_pow_right (by norm_num) hlen

/-! ## The falsifier, carried out

The lower bound is not bookkeeping. Replace the `2 · m · n`-bit cell array with
a list of the occupied points — an encoding a reader might well propose, since
it is shorter on sparse boards — and at 4×4 the statement above fails, by
`decide`: the empty board encodes in twenty-four bits, and `Situation 4 4` has
`2 · 3 ^ 16` elements, more than `2 ^ 24`. What that settles is the archive's
`2 ^ |w|` budget at that one instance, and nothing more. Whether the language
lies outside EXPSPACE under a stone-list encoding is not established here;
reading the single instance as a statement about the asymptotics is an
inference this file does not make.

`encSparse` exists to be refuted and is used nowhere else. -/

/-- A stone-list encoding: dimensions, color, then one self-delimiting index
and color bit per occupied point. -/
def encSparse (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) : List Bool :=
  sd m ++ (sd n ++ (colorBit c ::
    ((List.range (m * n)).flatMap
        (fun k => if (cellFlat b k).isSome then sd k ++ [colBit (cellFlat b k)] else [])
      ++ komiBits komi)))

/-- The empty 4×4 board takes twenty-four bits under `encSparse`. -/
theorem length_encSparse_empty :
    (encSparse 4 4 .black (fun _ => none) 0).length = 24 := by decide

/-- **Under a stone-list encoding the archive's `2 ^ |w|` budget fails at one
instance.** On the empty 4×4 board `encSparse` writes twenty-four bits while
`Fintype.card (Situation 4 4) = 2 · 3 ^ 16 > 2 ^ 24`, by `decide`. This refutes
the budget at that instance. It does not establish that the language lies
outside EXPSPACE under that encoding, and the asymptotic reading is an
inference, not this statement. -/
theorem sparse_encoding_refuted :
    ¬ Fintype.card (Situation 4 4) ≤ 2 ^ (encSparse 4 4 .black (fun _ => none) 0).length := by
  rw [length_encSparse_empty, card_situation]
  decide

/-! ## The language

`goLang` is the set of strings `enc` sends to instances Black wins, which is the
question `docs/formal-model.md` §7 poses: the color to move is part of the
input, and the winner asked about is always Black. `BlackWinsFrom … .black` is
`BlackWins` definitionally, so the color bit extends the project's problem
without touching `Defs.lean`. -/

/-- **Does Black have a winning strategy** when play starts from this position
with `c` to move and nothing yet forbidden — encoding (C) of
`docs/formal-model.md` §5, and the question of §7. The color to move is part of
the instance; the player asked about is not. -/
def BlackWinsFrom (m n : ℕ) (b : Position m n) (komi : ℚ) (c : Color) : Prop :=
  WinsFor m n SSK komi .black (start b c)

/-- **Bridge.** Black to move is the project's `Superko.BlackWins` exactly, so
the color bit costs nothing in the trusted base. -/
theorem blackWinsFrom_black (m n : ℕ) (b : Position m n) (komi : ℚ) :
    BlackWinsFrom m n b komi .black ↔ BlackWins m n b komi := Iff.rfl

/-- **The decoder, made strict.** `dec` accepts some strings outside `enc`'s
image — a self-delimiting natural written with trailing zero digits, a komi
written unreduced or over denominator zero. Re-encoding and comparing rejects
exactly those, which is also how a machine would check well-formedness, and it
costs one linear scan. -/
def decStrict (l : List Bool) : Option (Σ m n : ℕ, Color × Position m n × ℚ) :=
  (dec l).bind fun y =>
    if enc y.1 y.2.1 y.2.2.1 y.2.2.2.1 y.2.2.2.2 = l then some y else none

theorem decStrict_enc (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ) :
    decStrict (enc m n c b komi) = some ⟨m, n, c, b, komi⟩ := by
  simp [decStrict, dec_enc]

theorem enc_of_decStrict {l : List Bool} {y : Σ m n : ℕ, Color × Position m n × ℚ}
    (h : decStrict l = some y) :
    enc y.1 y.2.1 y.2.2.1 y.2.2.2.1 y.2.2.2.2 = l := by
  rw [decStrict, Option.bind_eq_some_iff] at h
  obtain ⟨z, _, hif⟩ := h
  split_ifs at hif with hc
  rw [Option.some_inj] at hif
  subst hif
  exact hc

/-- **SUPERKO-GO as a language of bit strings.** -/
def goLang : Set (List Bool) :=
  {x | ∃ (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ),
    enc m n c b komi = x ∧ BlackWinsFrom m n b komi c}

/-- The Black-to-move slice, which is `Superko.BlackWins` exactly. -/
def goLangBlack : Set (List Bool) :=
  {x | ∃ (m n : ℕ) (b : Position m n) (komi : ℚ),
    enc m n .black b komi = x ∧ BlackWins m n b komi}

theorem goLangBlack_subset : goLangBlack ⊆ goLang := by
  rintro x ⟨m, n, b, komi, hx, hw⟩
  exact ⟨m, n, .black, b, komi, hx, hw⟩

/-- **The image of `enc` and the set `decStrict` accepts are the same set.** So
the language a machine decides by parsing its input and running the decider is
`goLang` itself, not a larger set containing padded duplicates. -/
theorem goLang_eq_decStrict :
    goLang = {x | ∃ (m n : ℕ) (c : Color) (b : Position m n) (komi : ℚ),
      decStrict x = some ⟨m, n, c, b, komi⟩ ∧ BlackWinsFrom m n b komi c} := by
  ext x
  constructor
  · rintro ⟨m, n, c, b, komi, hx, hw⟩
    exact ⟨m, n, c, b, komi, by rw [← hx, decStrict_enc], hw⟩
  · rintro ⟨m, n, c, b, komi, hd, hw⟩
    exact ⟨m, n, c, b, komi, enc_of_decStrict hd, hw⟩

/-- **The declared dimensions cannot lie.** A string `decStrict` accepts with
dimensions `m`, `n` is at least `2 · m · n` bits long, so no short input can
make a machine believe in a large board — which is what stops the archive from
being doubly exponential in the input length. -/
theorem two_mul_le_length_of_decStrict {l : List Bool} {m n : ℕ} {c : Color}
    {b : Position m n} {komi : ℚ} (h : decStrict l = some ⟨m, n, c, b, komi⟩) :
    2 * (m * n) ≤ l.length := by
  have he := enc_of_decStrict h
  rw [← he]
  exact C31_enc_length_lower m n c b komi

/-- **The dimension bound needs no well-formedness check.** The same conclusion
as `two_mul_le_length_of_decStrict`, with six bits to spare, for the permissive
`dec` rather than `decStrict` — so a machine that parses its input without
re-encoding it still cannot be made to believe in a large board. -/
theorem two_mul_le_length_of_dec {l : List Bool} {m n : ℕ} {c : Color}
    {b : Position m n} {komi : ℚ} (h : dec l = some ⟨m, n, c, b, komi⟩) :
    2 * (m * n) + 6 ≤ l.length := by
  cases hr1 : readSd l with
  | none => simp [dec, hr1] at h
  | some p1 =>
    obtain ⟨m₁, l₁⟩ := p1
    cases hr2 : readSd l₁ with
    | none => simp [dec, hr1, hr2] at h
    | some p2 =>
      obtain ⟨n₁, l₂⟩ := p2
      cases hl2 : l₂ with
      | nil => simp [dec, hr1, hr2, hl2] at h
      | cons cb l₃ =>
        cases hk : readKomi (l₃.drop (2 * (m₁ * n₁))) with
        | none => simp [dec, hr1, hr2, hl2, hk] at h
        | some komi₁ =>
          simp only [dec, hr1, hr2, hl2, hk, Option.some.injEq, Sigma.mk.injEq] at h
          obtain ⟨hm, h'⟩ := h
          subst hm
          simp only [heq_eq_eq, Sigma.mk.injEq] at h'
          obtain ⟨hn, -⟩ := h'
          subst hn
          have h1 := readSd_len hr1
          have h2 := readSd_len hr2
          have h3 : (l₃.drop (2 * (m₁ * n₁))).length ≠ 0 := by
            intro hz
            exact readKomi_ne_nil hk (List.length_eq_zero_iff.mp hz)
          have h4 : (l₃.drop (2 * (m₁ * n₁))).length = l₃.length - 2 * (m₁ * n₁) :=
            List.length_drop ..
          have h5 : l₂.length = l₃.length + 1 := by rw [hl2]; simp
          omega

/-! ## Komi and the color: the two choices, and what they cost

`num_halfInteger` and `den_halfInteger` are the arithmetic C-27 needs in order
to say "every komi is equivalent to a half-integer" in the encoding's own
terms. `margin_ne_zero_halfInteger` is why a half-integer is the right
normal form, and `winner_swap_iff` is why the color bit is carried rather than
normalized away. -/

/-- **No tie at a half-integer komi.** The area difference is an integer, so
the margin is never zero, so `winner`'s tie convention is unobservable. -/
theorem margin_ne_zero_halfInteger {m n : ℕ} (b : Position m n) (z : ℤ) :
    margin b ((z : ℚ) + 1/2) ≠ 0 := by
  intro h
  rw [margin] at h
  have h2 : ((2 * ((area b .black : ℤ) - (area b .white : ℤ) - z) : ℤ) : ℚ) = 1 := by
    push_cast
    linarith
  have : (2 * ((area b .black : ℤ) - (area b .white : ℤ) - z) : ℤ) = 1 := by
    exact_mod_cast h2
  omega

/-- **Why the color bit is carried.** Swapping the stones' colors and negating
komi turns a White-to-move instance into a Black-to-move one only if the scores
cooperate, and at margin zero they do not: `winner` gives a tie to White, so the
identity fails exactly there. The area hypotheses are what a board-swap lemma
would supply. -/
theorem winner_swap_iff {m n : ℕ} (b b' : Position m n) (komi : ℚ)
    (hb : area b' .black = area b .white) (hw : area b' .white = area b .black) :
    winner b' (-komi) = (winner b komi).other ↔ margin b komi ≠ 0 := by
  have hm : margin b' (-komi) = -margin b komi := by
    simp only [margin, hb, hw]; ring
  constructor
  · intro h hzero
    rw [winner, winner, hm, hzero] at h
    simp at h
    exact absurd h (by decide)
  · intro hne
    rcases lt_or_gt_of_ne hne with hlt | hgt
    · have h1 : ¬ (0 < margin b komi) := by linarith
      have h2 : 0 < margin b' (-komi) := by rw [hm]; linarith
      simp [winner, h1, h2, Color.other]
    · have h1 : 0 < margin b komi := hgt
      have h2 : ¬ (0 < margin b' (-komi)) := by rw [hm]; linarith
      simp [winner, h1, h2, Color.other]

end Superko.Enc
