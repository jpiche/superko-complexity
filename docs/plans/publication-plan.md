# Publication plan

Draft. Written early because two of its decisions constrain the work rather
than follow it.

## The license is not cosmetic

**Decided: Apache-2.0**, from the first commit. See [`../../LICENSE`](../../LICENSE).

This project's claim to attention is that a reader can check it instead of
trusting the author. That only works if they are permitted to: to run the Lean
build, to re-run the search, to fork the development and test whether a
definition they doubt changes the outcome. A restrictive license would remove
the one property this work has going for it.

Apache-2.0 over MIT for the patent grant and because it matches Mathlib's,
which keeps open the option of proposing results upstream. The prose is covered
by the same license rather than a separate CC-BY, on the grounds that one
license is easier to comply with than two and nothing here needs the
distinction.

**Decided: public from the first commit.** Nothing in the repository assumes a
reader who already knows the author, and a repository that is checkable but
unavailable is not checkable. Publishing before there is a result also dates
the work, which costs nothing and settles priority questions cheaply.

## Ordering

The main question may not fall. The plan assumes it does not, and is built so
that the work still produces publishable results:

1. **A minimal position separating PSK from SSK** (C-17), with both values
   computed and both strategies machine-checked. Novel, self-contained, needs no
   complexity infrastructure, and settles something practitioners have discussed
   without resolving.
2. **The 1×9 score** (C-11), settling a disagreement between two published
   sources. A note, not a paper — but it validates the kernel publicly and cheaply.
3. **A bound on H(n)** (C-15), if the fooling-set direction yields a
   construction rather than a measurement.
4. **The formalization itself** — a machine-checked definition of Go under both
   superko rules, validated against the published counts — is a contribution
   independent of any theorem proved with it. ITP or CPP.
5. The classification, if it comes.

Items 1–4 do not depend on item 5, and that is the point of the ordering.

## Potentially overlapping work

Chung's 2026 thesis names superko formula games as future work.

## Approach to the field

Unaffiliated, so the ordinary route is closed and the substitute is the artifact.
arXiv preprint plus the repository, then direct contact with the small number of
people who have published on this — the "People" section of
[`../../references/README.md`](../../references/README.md) records who they are,
and on what terms.

Lead with the checkable thing. "Here is a definition of Go under superko that
reproduces your 2×2 count, and here is a machine-checked theorem about it" is a
message that costs its reader a minute to evaluate. A claimed resolution of the
open problem is a message that costs them an afternoon, and they will not spend it.
