---
name: handle-panic
description: Return or handle a Rust error instead of unwrap, expect, unwrap_err, expect_err, resume_unwind or a panic macro, or record why the panic is unreachable.
---

# Handle the error instead of panicking

`.unwrap()` and `.expect(...)` abort the program when an Option is None or a
Result is Err, `.unwrap_err()` and `.expect_err(...)` abort when a Result is
Ok, `resume_unwind` restarts a caught panic, and `panic!`, `todo!`,
`unimplemented!` and `unreachable!` abort outright. In shipped, non-test code
each one turns a condition the caller could recover from into a crash it
cannot.

1. Read what the call can fail on. For a Result or Option, propagate the
   failure with `?`, return a `Result`, or match the variants and handle both.
2. For an absent value that is a genuine invariant, restructure so the types
   carry the guarantee, or record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the call.
3. For `todo!`/`unimplemented!`, finish the work or return an error; for
   `unreachable!`, encode the impossible case in the types so the branch does
   not exist; for `resume_unwind`, return the failure as an error unless the
   panic must cross the boundary unchanged, and record why.
4. Rerun the reported lint check and the affected behavior checks.

A `DECISION:` or `INVARIANT:` reference on the line directly above the call
keeps it, attesting the panic is unreachable; without one it is a finding.
When the project's Clippy profile forbids the panic and an enclosing item
already carries a referenced `#[allow]` or `#[expect]` of that lint, such as
`clippy::expect_used` for `.expect(...)`, that one reference covers every
such panic inside the item; do not repeat it above each call.

## Rule binding

The measurement flags every Rust `.unwrap()`, `.expect(...)`, `.unwrap_err()`
and `.expect_err(...)` call, every `resume_unwind` call through a path or a bare
name, and every `panic!`, `todo!`, `unimplemented!` and `unreachable!` macro,
in source and in macro bodies, outside test code without a `DECISION:` or
`INVARIANT:` reference on the line directly above it or above an `#[allow]`
or `#[expect]` of its Clippy lint (`unwrap_used`, `expect_used`, `panic`,
`todo`, `unimplemented`, `unreachable`) on an enclosing item. Test code is out
of scope. Rerun the reported lint check and relevant behavior checks. Do not weaken
the rule or its selectors merely to pass. Apply complexity-discipline; stop when
the error is returned or handled, or the panic is justified by an owned
reference.
