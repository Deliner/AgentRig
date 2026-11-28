---
name: bind-with-if
description: Bind a variable with an if expression instead of a declaration followed by assignments in the branches, so that the binding is immutable and set on every path.
---

# Bind a value with an `if` expression

`let mode; if verbose { mode = Mode::Full; } else { mode = Mode::Short; }`
and `let mut mode = Mode::Short; if verbose { mode = Mode::Full; }` spread one
binding over a declaration and assignments. A reader checks every branch to
learn the value, and the second form keeps the variable mutable for the rest
of its scope.

1. Bind the value with the condition:
   `let mode = if verbose { Mode::Full } else { Mode::Short };`. A default
   that one branch replaces becomes the `else` value.
2. Keep any statements a branch runs before its assignment inside that branch,
   and end the block with the value.
3. Rerun the reported lint check and the affected behavior checks.

No exception keeps the sequence: an `if` expression binds every value the
assignments can give, so do not suppress the lint.

## Rule binding

The rig's Clippy profile denies `clippy::useless_let_if_seq` in the library,
binaries and examples of the configured Cargo manifest; tests are outside the
profile. Clippy reports a `let` directly followed by an `if` that assigns the
variable at the end of its branches, and the lint check reports each message
as an error naming this skill. Do not weaken the profile merely to pass. Apply
complexity-discipline; stop when the variable is bound by the `if` expression.
