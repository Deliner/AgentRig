---
name: check-the-flag-first
description: Check a flag before the call whose result a filter ignoring its value would keep or drop, with if or flag.then, so that the call runs only when its result is kept, or record why the call must run regardless.
---

# Run the call only when the flag keeps its result

`self.storage.next_save().filter(|_| idle)` reads as a save taken when idle,
but the call runs first and the flag decides afterwards. When the flag is false
the save batch is cloned and thrown away. A filter whose closure ignores its
value is a branch on a flag written after the work it guards.

1. Establish whether the call must run even when the flag is false, for a side
   effect the rest of the code relies on. If so, record that reasoning as an
   owned `DECISION:`/`INVARIANT:` reference on the line directly above the
   filter and stop.
2. In a statement, branch on the flag first: return early when the flag is
   false, or put `if idle` around the `if let`.
3. In an expression yielding an `Option`, put the flag before the call:
   `idle.then(|| self.storage.next_save()).flatten()`, also as the scrutinee of
   an `if let` or `let .. else`. For a call returning a plain value,
   `idle.then(|| one(&table.uniform))`.
4. For an iterator, guard the loop with `if flag`, or write
   `flag.then(|| items.iter()).into_iter().flatten()` where an iterator is
   needed; a filter that ignores each item still walks them all.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust `.filter(..)` method call whose receiver
contains a call and whose sole argument is a closure with the single parameter
`_` and a body without a call, in product and test code, without a `DECISION:`
or `INVARIANT:` reference on the line directly above it. Types are not
resolved, so an `Option`, a `Result` turned into one and an iterator are
reported alike, and a cheap accessor such as `as_ref` or `get` counts like an
expensive call. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the call runs only when the flag keeps its
result or an owned reference justifies running it regardless.
