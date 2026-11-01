---
name: remove-debug
description: Remove a leftover Rust dbg! macro and keep any value it wrapped.
---

# Remove a leftover dbg! debugging artifact

A `dbg!(...)` macro prints its argument and source location to stderr and
returns the value unchanged. It is a debugging aid left in the source: it writes
unconditioned diagnostic noise and marks unfinished work. A committed `dbg!` has
no legitimate use, so no reference justifies keeping it.

1. Read the `dbg!` and the expression it wraps. `dbg!(expr)` evaluates to
   `expr`, so the surrounding code may depend on that value.
2. Remove the macro and keep the value: rewrite `dbg!(expr)` as `expr`, and
   delete a standalone `dbg!(expr);` statement whose value is unused.
3. When the diagnostic was genuinely needed, replace it with the project's real
   logging or a test that asserts the value, not another debugging print.
4. Rerun the reported lint check and the affected behavior checks.

Unlike the reference-justified rules, a `dbg!` cannot be kept with a DECISION:
or INVARIANT: comment: remove it.

## Rule binding

The measurement flags every Rust `dbg!` macro invocation, unconditionally. Rerun
the reported lint check and relevant behavior checks. Do not weaken the rule or
its selectors merely to pass. Apply complexity-discipline; stop when the macro is
removed and the value it wrapped is preserved.
