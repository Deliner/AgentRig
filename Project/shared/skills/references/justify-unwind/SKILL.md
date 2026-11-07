---
name: justify-unwind
description: Let a Rust panic propagate instead of catching it with catch_unwind or vouching for state with AssertUnwindSafe, or record why the boundary must not unwind.
---

# Let the panic propagate unless the boundary must not unwind

`catch_unwind` turns a panic into a value, and `AssertUnwindSafe` vouches that
the state it wraps is still consistent after a panic interrupted its update. A
panic signals a bug, so catching it lets the program continue on state the
panic may have left half-written, and the original failure surfaces later as a
confusing second one. Recoverable failures belong in `Result`.

1. Read what the catch protects. If the wrapped code can fail in an expected
   way, return a `Result` from it and handle the error instead of catching the
   panic.
2. If the catch only hides a bug, remove it and let the panic propagate, then
   fix the cause it was hiding.
3. Keep the catch only at a boundary a panic must not cross: an `extern "C"`
   callback, a thread pool or task runner that isolates one job, or a plugin
   host. Discard or rebuild every value the panic could have left inconsistent,
   and wrap with `AssertUnwindSafe` only values that stay consistent or are
   dropped. Record that reasoning as an owned `DECISION:`/`INVARIANT:` reference
   on the line directly above the call.
4. Rerun the reported lint check and the affected behavior checks.

A `DECISION:` or `INVARIANT:` reference on the line directly above the call
keeps it, attesting the boundary is deliberate; without one it is a finding.

## Rule binding

The measurement flags every Rust call whose function path or bare name ends in
`catch_unwind` or `AssertUnwindSafe`, and every `.catch_unwind()` method call,
outside test code without a `DECISION:` or `INVARIANT:` reference on the line
directly above it. `UnwindSafe` bounds and types are not flagged. Test code is
out of scope. Rerun the reported lint check and relevant behavior checks. Do not
weaken the rule or its selectors merely to pass. Apply complexity-discipline;
stop when the panic propagates or the catch is justified by an owned reference.
