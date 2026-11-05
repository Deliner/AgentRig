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
7. Make each test deterministic. Pin behavior through injected or fixture values,
   not wall-clock time, random seeds, the network, the filesystem outside a temp
   area, or another test's side effects or order. A test that turns on anything
   but the code under test is a defect; fix the dependence rather than disabling
   the test.
8. Keep test seams out of production code. A hook, override, environment
   switch or `cfg!(test)` branch that exists only so a test can steer the code
   is a defect: pass the dependency (clock, filesystem root, command runner)
   through the public surface, so production and tests take the same path. A
   seam at a security, linker or test-runtime boundary is the exception and
   carries a `// DECISION:` reference naming that boundary.

Stop when each changed behavior is pinned through its public surface at its
owner. Apply complexity-discipline. A no-op or pure prose change needs no test.

## Enforcement

These are review norms. A principle earns a lint rule only once it can be
checked statically without false positives, and none of these can be yet.
Placement (1) cannot: an import edge shows that a test depends on code, not
whether that code is the behavior under test or an incidental shared type, so
flagging every out-of-area dependency would punish ordinary shared vocabulary
and test helpers. Public surface and back doors (2, 4) cannot: no symbol
visibility or reverse-reference analysis exists. Determinism (7) cannot either: a
call to a clock, RNG or the network is not by itself a nondeterminism bug, and
order dependence is not syntactic, so a blanket ban would be false-positive-prone.
Seams (8) cannot: a parameter or `cfg` branch does not say whether a test is
its only caller, and the security, linker and test-runtime exceptions share
that syntax.
The one decidable part — a test disabled without a justifying reference — is
enforced by the `disabled-test` rule; the rest apply in review, where a finding
would be repaired here.
