---
name: firm-up-assertion
description: Promote a release-stripped Rust debug_assert, or an assert compiled only with debug_assertions, to a real assert, remove it, or record why the debug-only check is enough.
---

# Firm up the contract instead of asserting it only in debug

`debug_assert!`, `debug_assert_eq!` and `debug_assert_ne!` compile only under
`cfg(debug_assertions)`, so the contract they state is checked in debug and
tests but stripped from an optimized release — exactly where a violation does
the damage. An `assert!` placed under `#[cfg(debug_assertions)]` or inside
`if cfg!(debug_assertions)` is the same debug-only check in disguise. In
shipped, non-test code the check silently does nothing.

1. Decide whether the property must hold in release. If it guards correctness
   or safety, promote it to an unconditional `assert!`/`assert_eq!`/
   `assert_ne!` outside the debug-only gate, which holds in every build.
2. If the check is an expensive internal sanity check not worth its cost in
   release and its failure is already impossible by construction, record the
   reasoning as an owned `DECISION:`/`INVARIANT:` reference on the line directly
   above it.
3. If the assertion is redundant with the surrounding types or logic, remove it.
4. Rerun the reported lint check and the affected behavior checks.

A `DECISION:` or `INVARIANT:` reference on the line directly above the call
keeps it, attesting the debug-only check is deliberate; without one it is a
finding.

## Rule binding

The measurement flags every Rust `debug_assert!`, `debug_assert_eq!` and
`debug_assert_ne!` macro, and every `assert!`, `assert_eq!` and `assert_ne!`
under a `cfg` attribute that requires `debug_assertions` (directly or inside
`all`/`any`) on the statement or an enclosing item, or in the then-branch of an
`if` whose whole condition is `cfg!(debug_assertions)`, outside test code
without a `DECISION:` or `INVARIANT:` reference on the line directly above it.
An unconditional `assert!` is not flagged, since it holds in every build. Test
code is out of scope. Rerun the reported lint check and relevant behavior
checks. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the contract holds in release, is justified by
an owned reference, or is removed.
