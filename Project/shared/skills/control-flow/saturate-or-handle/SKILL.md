---
name: saturate-or-handle
description: Handle the missing value before adding to or multiplying a fallback of unwrap_or(MAX), or compute with saturating_add or saturating_mul, so that the bound neither overflows nor loses its meaning, or record why the fallback cannot be reached.
---

# Keep a MAX fallback a bound

`usize::try_from(queue.bytes()).unwrap_or(usize::MAX) + bulk.bytes()` reads as
a count that saturates when the queue is too large, but `MAX + n` overflows: it
panics in a debug build and wraps to a small number in a release build. Even
without an overflow, `MAX * 4 + 1` no longer means the bound the fallback
stood for.

1. Establish whether the fallback can be reached. When a check just above or
   the range of the value guarantees that it cannot, record that guarantee as
   a decision with edit-decisions, put its exception `// DECISION: DNNN RULE`,
   RULE being the rule ID the finding names, alone on the line directly above
   the arithmetic, list it in the decision as `[exception RULE](FILE)` and
   stop; the user approves a new exception through propose-protected-change.
2. Decide what a missing or too large value means here: an error, a skipped
   step or the bound itself.
3. For an error or a skipped step, handle the absence before the arithmetic,
   with `?`, a `let .. else` or a `match`, and compute with the real value:
   `let queued = usize::try_from(queue.bytes())?;`.
4. When the bound itself is the answer, saturate explicitly:
   `usize::try_from(queue.bytes()).unwrap_or(usize::MAX).saturating_add(bulk.bytes())`,
   or `saturating_mul` for a product.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust binary `+` or `*` whose left or right
operand is a `.unwrap_or(..)` method call whose sole argument is `MAX` or a
path ending in `::MAX`, outside test code. An exception naming this rule,
`max-fallback` by default, on the line directly above the reported line lifts
the finding; a bare `DECISION:` or `INVARIANT:` reference lifts nothing. Types
are not resolved, so a float `MAX` counts too; `+=`, `*=`, `unwrap_or_else` and
a parenthesized operand are not measured. Do not weaken the rule or its
selectors merely to pass. Apply complexity-discipline; stop when the arithmetic
handles the missing value or saturates, or an approved exception lifts the
finding.
