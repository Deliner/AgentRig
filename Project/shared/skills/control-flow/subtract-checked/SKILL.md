---
name: subtract-checked
description: Replace a subtraction from .len(), .count(), .count_ones() or .count_zeros(), or of a Duration from an Instant or a Duration, with checked_sub or saturating_sub and handle the short case, rearrange it away, or record the check that guarantees the count.
---

# Subtract from a count only when it cannot underflow

`len()` and `count()` return `usize`. `items.len() - n` panics in a debug build
when the collection holds fewer than `n` items and wraps to a huge number in a
release build, so an index or a loop bound built from it reads past the data or
runs nearly forever. The empty collection is the usual trigger:
`items.len() - 1` on an empty vector. Bit counts behave the same:
`widen(changed.count_ones()) - dense` panics when the input breaks the expected
subset of the masks, where a decoder should return its ordinary error.

1. Decide what a count smaller than the subtrahend means here: an error, an
   empty result or zero.
2. Express that decision in the arithmetic: `items.len().checked_sub(n)` with
   the `None` case handled by `?`, `let … else` or a `match`, or
   `items.len().saturating_sub(n)` when zero is the right answer. For the last
   element use `items.last()`, `items.split_last()` or `items.iter().rev()`
   instead of `items[items.len() - 1]`.
3. Move the term to the other side of a comparison to drop the subtraction:
   `index + n < items.len()` instead of `index < items.len() - n`.
4. When a check just above guarantees the count, such as an `is_empty()`
   return, a minimum-size assertion or a subset check of the bit masks, record
   that guarantee as a decision with edit-decisions and put its exception
   `// DECISION: DNNN RULE`, RULE being the rule ID the finding names, alone
   on the line directly above the reported line. List it in the decision as
   `[exception RULE](FILE)`; the user approves a new exception through
   propose-protected-change.
5. Rerun the reported lint check and the affected behavior checks.

## Clippy profile

The rig's Clippy profile denies `clippy::unchecked_time_subtraction` in the
library, binaries and examples of the configured Cargo manifest; tests are
outside the profile. Clippy reports a `-` between an `Instant` and a
`Duration` or between two `Duration` values, which panics when the result
cannot be represented, such as a negative duration; the lint check reports
each message as an error naming this skill. Decide what that case means as in
step 1, then write `deadline.checked_sub(margin)` or
`budget.checked_sub(spent)` and handle `None`, or
`budget.saturating_sub(spent)` when zero is the right answer. When a check
just above guarantees the order, record that guarantee as a decision with
edit-decisions and put `#[expect(clippy::unchecked_time_subtraction)]` on the
smallest item that holds the subtraction, with its exception
`// DECISION: DNNN allow-justification` alone on the line directly above the
attribute. List it in the decision as `[exception allow-justification](FILE)`;
the user approves a new exception through propose-protected-change, as
justify-allow describes. Do not weaken the profile merely to pass.

## Rule binding

The measurement flags every Rust binary `-` whose left operand is a call of a
method named `len`, `count`, `count_ones` or `count_zeros` without arguments,
or a call whose only argument is such a call, such as a widening function,
outside test code. An exception naming this rule, `length-subtraction` by
default, on the line directly above the reported line lifts the finding; a
bare `DECISION:` or `INVARIANT:` reference lifts nothing. The receiver type is
not resolved, so a `len` of any type counts; `-=`, a count on the right side
and a count wrapped twice are not measured. Rerun the reported lint check and
relevant behavior checks. Do not weaken the rule or its selectors merely to
pass. Apply complexity-discipline; stop when the subtraction is checked,
saturating or rearranged away, or an approved exception lifts the finding.
