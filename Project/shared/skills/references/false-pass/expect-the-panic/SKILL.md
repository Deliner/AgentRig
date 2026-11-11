---
name: expect-the-panic
description: Name the panic a Rust should_panic test expects with expected, or test the returned error instead, or record why any panic is the intended outcome.
---

# Name the panic the test expects

A test under a bare `#[should_panic]` passes on any panic in its body. When the
setup unwraps a value, indexes a slice or calls code that panics for an
unrelated reason, the test stays green although the behavior it was written for
is broken or never reached.

1. Read the test and find the panic it is meant to catch: the call, the
   condition and the message the product code gives it.
2. Name that message with `#[should_panic(expected = "...")]`, using a
   substring that only the intended panic contains. Keep it short enough to
   survive wording changes elsewhere in the message.
3. When the code under test returns a `Result` or `Option`, prefer testing the
   returned value: assert on the specific error variant or kind instead of
   turning it into a panic.
4. When any panic is the intended outcome and no project message exists, such
   as a panic raised by the standard library or a dependency, record the
   reasoning as an owned `DECISION:`/`INVARIANT:` reference on the line
   directly above the attribute.
5. Rerun the reported lint check and the test itself, and confirm it fails when
   the intended panic is removed.

A `DECISION:` or `INVARIANT:` reference on the line directly above the
attribute keeps it; without one it is a finding.

## Rule binding

The measurement flags every Rust attribute whose path ends in `should_panic`
with neither a `= "..."` value nor an `expected` argument, without a
`DECISION:` or `INVARIANT:` reference on the line directly above it. The
attribute belongs to a test, so test code is in scope. Rerun the reported lint
check and relevant behavior checks. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when the test names its
panic, tests the returned error, or is justified by an owned reference.
