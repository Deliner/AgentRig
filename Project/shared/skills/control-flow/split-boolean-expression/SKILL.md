---
name: split-boolean-expression
description: Split a long boolean expression, or one that joins conjunctions with disjunctions, counted by boolean-expression into named parts while preserving its truth table.
---

# Give each group of conditions a name

Make every logical expression readable at a glance: a reader should see which
conditions belong together and what each group means without tracing operator
precedence. Resolve a finding by naming meaningful groups, not by hiding the same
operands behind a helper that still combines them all at once.

1. Read the expression and establish what each operand tests, how `&&` and `||`
   or `and` and `or` group them, and which operands short-circuit an effect,
   a panic or an expensive call.
2. Find the groups that state one reason: the conditions of one case, one
   validity check or one alternative. Bind each group to a `let` or a local
   variable named for that reason, such as `regular`, `elevated` or `malformed`.
3. Combine the named groups in a short final expression. A mixed `&&` and `||`
   usually splits at its `||`: each alternative becomes one name.
4. When several operands compare one value with constants, replace them with
   one `matches!`, a `match` or a membership test.
5. When an operand must stay lazy because it has an effect, may panic or is
   costly, keep it in the final expression or behind a named function instead of
   evaluating it eagerly in a binding.
6. Verify the changed code through its callers and affected behavior checks.

Do not drop or merge conditions, change their grouping or reorder effects merely
to lower the count. A negated group often reads better as a positive name
combined with `!`.

## Rule binding

The measurement counts the operands of one maximal Rust `&&` and `||` or Python
`and` and `or` expression anywhere in code. Parentheses group operands without
ending the expression; a negation, comparison, call, closure or macro is one
operand whose inner expressions are measured on their own, and a `let` binding of
a Rust let chain is not an operand. With the rule's `junctions: single` setting,
every such expression that joins `&&` with `||` or `and` with `or`, directly or
inside parentheses, is also an error finding whatever its operand count: bind
each conjunction or disjunction to a named value and combine the names with one
junction. Rerun the reported lint check and relevant behavior checks. Do not
raise the threshold or relax the setting merely to pass. Apply
complexity-discipline; stop when the finding is resolved and behavior is preserved.
