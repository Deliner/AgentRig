---
name: firm-up-assertion
description: Promote a release-stripped Rust debug_assert to a real assert, remove it, or record why the debug-only check is enough.
---

# Firm up the contract instead of asserting it only in debug

`debug_assert!`, `debug_assert_eq!` and `debug_assert_ne!` compile only under
`cfg(debug_assertions)`, so the contract they state is checked in debug and
tests but stripped from an optimized release — exactly where a violation does
the damage. In shipped, non-test code the check silently does nothing.

1. Decide whether the property must hold in release. If it guards correctness
   or safety, promote it to `assert!`/`assert_eq!`/`assert_ne!`, which holds in
   every build.
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
`debug_assert_ne!` macro outside test code without a `DECISION:` or
`INVARIANT:` reference on the line directly above it. `assert!` and its siblings
are not flagged, since they hold in every build. Test code is out of scope.
Rerun the reported lint check and relevant behavior checks. Do not weaken the
rule or its selectors merely to pass. Apply complexity-discipline; stop when the
contract holds in release, is justified by an owned reference, or is removed.
