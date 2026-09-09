# Test data

Fixture positions with expected values, read by the acceptance suite and the
regression tests.

Two kinds, and they are not interchangeable:

- **Literature fixtures.** Positions and values published by other people:
  the 1×n minimax scores, small-board game counts. These validate the
  definitions — disagreement means this project is wrong, not the source
  ([`../docs/trusted-base.md`](../docs/trusted-base.md)). Each fixture names
  its source.
- **Project fixtures.** Positions this project cares about: PSK/SSK divergence
  candidates, sending-two-returning-one shapes, double-ko seki, and whatever
  minimal separating position C-17 turns up. Each says why it is here.

A fixture with an expected value computed by this project's own code is a
regression test, not a validation, and must say so.

Empty.
