---
name: name-the-states
description: Replace an Option<bool> or Option<Option<..>> with an enum that names each of its states, or record why an external format fixes the nesting.
---

# Name each state of a nested option

`Option<bool>` holds three states, `None`, `Some(false)` and `Some(true)`, and
`Option<Option<T>>` holds absent, empty and present; neither names them. A reader
has to guess what `None` means, and `unwrap_or(false)` or `flatten()` silently
merges two of the states.

1. Read where the value is produced and consumed, and establish what each state
   means, such as not asked yet, refused and granted, or keep, clear and set.
2. Declare an enum with one variant per state, such as
   `enum Consent { Unasked, Refused, Granted }` or
   `enum Patch<T> { Keep, Clear, Set(T) }`, and replace the type and every
   construction and match.
3. Where a consumer merged two states, decide the merge explicitly in a `match`
   arm or a named method, so that it is visible.
4. When a data type mirrors an external format, such as a JSON field that may be
   missing or null, deserialize into the enum where the format allows it;
   otherwise record the reasoning as an owned `DECISION:`/`INVARIANT:`
   reference on the line directly above the type.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust `Option<bool>` or `Option<Option<..>>` type.
Test code and a type with a `DECISION:` or `INVARIANT:` reference on the line
directly above it are out of scope. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when every state has a name or
the nesting is justified by an owned reference.
