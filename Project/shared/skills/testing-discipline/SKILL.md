---
name: testing-discipline
description: Place a test by the behavior it exercises and assert observable behavior through the public surface.
---

# Test the behavior a user sees, through the public surface

A test states what the code does for its caller, not how it is built. Written
this way it survives refactoring and never forces internals to leak.

1. Place the test by the behavior it exercises. Put it beside the unit it
   checks. If it exercises behavior spanning several units, raise it to their
   common owner; if it checks one narrower unit, lower it there. Derive the
   level from the owning directory and its architecture contract, not from
   convenience.
2. Enter only through the public surface. Call the public entry points of the
   thing under test. An item made public solely so a test can reach it is a
   defect: make it private, crate-local or test-scoped, or exercise it through a
   public path. Do not keep the widened visibility.
3. Assert observable behavior, not internal state. Check the result the caller
   observes, not private fields, intermediate values or representation.
4. Use no back doors. Do not reach private internals through direct field
   access, reflection or test-only hooks; that re-creates the coupling points 2
   and 3 remove.
5. Name the behavior, not the mechanism. The test name states the capability it
   pins, so a reader learns the contract without reading the body.
6. Do not duplicate coverage downward. When a behavior is covered at its proper
   level, do not re-assert it through a lower level's internals; keep one
   behavioral test at the owner.

Stop when each changed behavior is pinned through its public surface at its
owner. Apply complexity-discipline. A no-op or pure prose change needs no test.

## Enforcement

These are review norms. A principle earns a lint rule only once it can be
checked statically without false positives, and none of these can be yet.
Placement (1) cannot: an import edge shows that a test depends on code, not
whether that code is the behavior under test or an incidental shared type, so
flagging every out-of-area dependency would punish ordinary shared vocabulary
and test helpers. Public surface and back doors (2, 4) cannot: no symbol
visibility or reverse-reference analysis exists. Until such analysis is built,
apply these in review; a finding would be repaired here.
