---
name: destructure-the-fields
description: Make a manual PartialEq, Hash, PartialOrd, Ord or Clone destructure Self without .. so that a field added later must be decided, or record why the impl reads a subset.
---

# Make every field part of the decision

A manual `PartialEq`, `Hash`, `PartialOrd`, `Ord` or `Clone` that reads fields as
`self.id` names only the fields that existed when it was written. A field added
to the type later compiles and is silently left out of the comparison, the hash
or the copy, so equal values hash apart or a clone loses state.

1. Read the impl and the type, and establish which fields the operation must
   read and which it deliberately ignores.
2. When the impl matches what a derive produces, replace it with the derive.
3. Otherwise destructure both sides without `..`, such as
   `let Self { id, name, cache: _ } = self;`, and bind an ignored field to `_`
   by name, so that adding a field fails to compile until the impl decides it.
4. In a `Clone`, build the result with a struct literal of the type that names
   every field and has no `..base`.
5. Keep `Hash` and `Ord` reading the same fields as `PartialEq`; for the order,
   `align-order-with-equality` describes the repair.
6. When the impl cannot destructure the type, record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the impl.
7. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every manual Rust impl of `PartialEq`, `Hash`,
`PartialOrd`, `Ord` or `Clone` whose method reads a `self` field by access
without a struct or tuple-struct pattern of the type lacking `..`, and without a
struct literal of the type lacking a base. Test code and an impl with a
`DECISION:` or `INVARIANT:` reference on the line directly above it are out of
scope. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when every field is named in the impl, the impl is
derived, or it is justified by an owned reference.
