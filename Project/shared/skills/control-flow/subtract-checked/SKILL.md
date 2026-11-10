---
name: subtract-checked
description: Replace a subtraction from .len() or .count() with checked_sub or saturating_sub and handle the short case, rearrange it away, or record the check that guarantees the length.
---

# Subtract from a length only when it cannot underflow

`len()` and `count()` return `usize`. `items.len() - n` panics in a debug build
when the collection holds fewer than `n` items and wraps to a huge number in a
release build, so an index or a loop bound built from it reads past the data or
runs nearly forever. The empty collection is the usual trigger:
`items.len() - 1` on an empty vector.

1. Decide what a collection shorter than the subtrahend means here: an error,
   an empty result or zero.
2. Express that decision in the arithmetic: `items.len().checked_sub(n)` with
   the `None` case handled by `?`, `let … else` or a `match`, or
   `items.len().saturating_sub(n)` when zero is the right answer. For the last
   element use `items.last()`, `items.split_last()` or `items.iter().rev()`
   instead of `items[items.len() - 1]`.
3. Move the term to the other side of a comparison to drop the subtraction:
   `index + n < items.len()` instead of `index < items.len() - n`.
4. When a check just above guarantees the length, such as an `is_empty()`
   return or a minimum-size assertion, record that guarantee as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the
   subtraction.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust binary `-` whose left operand is a call of a
method named `len` or `count` without arguments, outside test code, without a
`DECISION:` or `INVARIANT:` reference on the line directly above the
expression. The receiver type is not resolved, so a `len` of any type counts;
`-=` and a length on the right side are not measured. Rerun the reported lint
check and relevant behavior checks. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when the subtraction is
checked, saturating or rearranged away, or the length is justified by an owned
reference.
