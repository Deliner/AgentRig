---
name: branch-on-the-index
description: Select one value with match or if on the index instead of indexing an array literal whose elements call, so that only the chosen element is computed, or record why every element must run.
---

# Compute only the element the index selects

`[values.clone().min(), values.max()][end]` reads as a choice between two
values, but the array literal evaluates every element before the index picks
one. The calls of the elements not chosen cost time, allocate, can fail or panic
and change state for a value that is thrown away.

1. Read the index and establish the values it can take. An index past the last
   element panics in the indexed form.
2. Replace the indexed literal with a `match` on the index whose arms each
   compute one element, such as
   `match end { 0 => values.clone().min(), _ => values.max() }`. Use a wildcard
   arm only when the index cannot take a value past the elements; otherwise keep
   that outcome in its own arm, such as a returned error.
3. When the index is a `bool` or a condition converted to a number, branch on
   that condition with `if` instead.
4. When every element must run for its side effects, record the reasoning as an
   owned `DECISION:`/`INVARIANT:` reference on the line directly above the
   indexed literal.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust array literal that lists its elements, at
least one of which contains a call or method call, indexed by an expression
other than an integer literal or the full range `..`, in product and test code,
without a `DECISION:` or `INVARIANT:` reference on the line directly above it.
A repeat literal `[x; n]` and calls inside macro arguments are out of scope.
Types are not resolved, so a cheap call counts like an expensive one. Do not
weaken the rule or its selectors merely to pass. Apply complexity-discipline;
stop when only the chosen element is computed or an owned reference justifies
computing them all.
