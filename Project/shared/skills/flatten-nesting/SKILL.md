---
name: flatten-nesting
description: Reduce the control-flow nesting depth reported by nesting-depth while preserving behavior.
---

# Flatten control flow without changing what it decides

Make the decisions readable at a shallow depth while preserving the same
conditions, order, effects and results. Depth, not length, is the finding:
resolve it by removing levels, not by moving code elsewhere unchanged.

1. Read the function and establish the conditions that lead to each branch, the
   order of effects, early exits and the returned value on every path.
2. Prefer an early return, `continue` or `?` for a guard that handles a case and
   leaves: invert the condition and exit so the main path stays at the outer
   level instead of inside an `else`.
3. Collapse a condition that only wraps more conditions into one combined test,
   and replace a deep `if`/`else` ladder with a flat `match` on the real case.
4. Extract a genuinely independent inner block into its own function only when it
   names a cohesive responsibility; apply refactor-long-function for that move.
5. Verify the changed function through its callers and affected behavior checks.

Do not hide depth behind a helper that still nests as deeply, duplicate a branch
to avoid an `else`, or drop a case to lower the count. A guard clause must leave
the same state as the branch it replaces.

## Rule binding

The measurement is the deepest chain of nested control-flow blocks in one
function; `if`, `match`, `loop`, `while` and `for` each add a level, Rust
else-if and Python elif continue the same level, and nested function and closure
bodies are measured on their own. Rerun the reported lint check and relevant
behavior checks. Do not raise the threshold merely to pass. Apply
complexity-discipline; stop when the finding is resolved and the behavior is
preserved.
