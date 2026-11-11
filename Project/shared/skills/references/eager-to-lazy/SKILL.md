---
name: eager-to-lazy
description: Move a fallback argument that can fail or panic into the lazy *_else closure, or record why the eager evaluation is intended.
---

# Defer the fallback instead of evaluating it eagerly

The argument of `.unwrap_or`, `.ok_or`, `.map_or` and `.or` is evaluated before
the receiver is known to be empty. When that argument contains a `?`, the error
propagates even on the common path where the value is present: the function
returns early or errors although a value was available — a latent bug. A
division by a divisor that is not a literal panics on zero the same way, although
its result is never used.

1. Move the fallback into the lazy closure form — `unwrap_or_else`, `ok_or_else`,
   `map_or_else` or `or_else` — so the `?` or the division runs only when the
   value is actually absent. A `?` that must leave the enclosing function goes
   in the empty branch of a `match` or `let ... else` instead of a closure.
2. For `map_or`, the mapping closure already defers; only the eager default
   argument needs moving into `map_or_else`.
3. For a fallback whose `?` or division is genuinely intended (the value-present
   path is unreachable, or the divisor is checked), record the reasoning as an
   owned `DECISION:`/`INVARIANT:` reference on the line directly above the call.
4. Rerun the reported lint check and the affected behavior checks.

A `DECISION:` or `INVARIANT:` reference on the line directly above the call keeps
it; without one it is a finding. Test code is out of scope.

## Rule binding

The measurement flags every Rust `.unwrap_or`, `.ok_or`, `.map_or` or `.or` call
outside test code whose first argument subtree contains a `?` (`try_expression`)
or a division whose divisor is not a literal, stopping at a nested closure,
without a `DECISION:`/`INVARIANT:` reference on the line directly above it. The
lazy `*_else` closures and `map_or`'s mapping closure are not flagged. Rerun the
reported lint check and relevant behavior checks. Do not weaken the rule or its
selectors merely to pass. Apply complexity-discipline; stop when the fallback is
lazy or justified by an owned reference.
