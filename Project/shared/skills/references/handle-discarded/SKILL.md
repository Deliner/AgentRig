---
name: handle-discarded
description: Handle or propagate a Result instead of dropping it with a bare .ok() statement, or record why the outcome is ignored.
---

# Handle the Result instead of discarding it

A bare `expr.ok();` statement converts a `Result` to an `Option` and immediately
drops it, so a failure passes with no handling and no trace — the same silent
swallow that `silent-error` catches for a Python `except: pass`.

1. Decide what the failure means here. Propagate it with `?`, return a `Result`,
   or match the variants and handle both.
2. Keep the outcome when it matters: `let kept = expr.ok();` or an assignment
   preserves the `Option` for a later decision.
3. For a deliberate best-effort call whose failure is genuinely ignorable (a
   cleanup on an already-failing path), record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the statement.
4. Rerun the reported lint check and the affected behavior checks.

A `DECISION:` or `INVARIANT:` reference on the line directly above the statement
keeps it; without one it is a finding. Test code is out of scope.

## Rule binding

The measurement flags every Rust statement that is a bare call whose method is
`.ok()` outside test code, without a `DECISION:`/`INVARIANT:` reference on the
line directly above it. A `let`-binding, an assignment and `expr.ok()?` keep or
propagate the outcome and are not flagged. Rerun the reported lint check and
relevant behavior checks. Do not weaken the rule or its selectors merely to pass.
Apply complexity-discipline; stop when the Result is handled or justified by an
owned reference.
