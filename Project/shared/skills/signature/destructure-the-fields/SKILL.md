---
name: destructure-the-fields
description: Make a manual PartialEq, Hash, PartialOrd, Ord, Clone or Debug destructure Self without .., and drop the .. of a pattern that binds every field, so that a field added later must be decided, or record why the impl reads a subset.
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
6. When the impl cannot destructure the type, record the reasoning as a
   decision with edit-decisions and put its exception
   `// DECISION: DNNN RULE`, RULE being the rule ID the finding names, alone
   on the line directly above the impl, below its attributes. List it in the
   decision as `[exception RULE](FILE)`; the user approves a new exception
   through propose-protected-change.
7. Rerun the reported lint check and the affected behavior checks.

## Clippy profile

The rig's Clippy profile denies `clippy::rest_pat_in_fully_bound_structs` and
`clippy::missing_fields_in_debug` in the library, binaries and examples of the
configured Cargo manifest; tests are outside the profile. The lint check
reports each Clippy message as an error naming this skill.

- `clippy::rest_pat_in_fully_bound_structs` reports a struct pattern that
  names every field and still ends in `..`. Remove the `..`, so that a field
  added later must be decided at the pattern.
- `clippy::missing_fields_in_debug` reports a manual `Debug` of a struct that
  builds `debug_struct` and leaves a field out without
  `finish_non_exhaustive`. Destructure `Self` without `..` as in step 3, show
  each kept field with `.field("name", name)`, and end with
  `.finish_non_exhaustive()` when a field is left out, such as a secret or a
  cache, so that the output marks the omission.

Either repair always applies, so no exception keeps these forms; do not
suppress the lints or weaken the profile merely to pass.

## Rule binding

The measurement reports every manual Rust impl of `PartialEq`, `Hash`,
`PartialOrd`, `Ord` or `Clone` whose method reads a `self` field by access
without a struct or tuple-struct pattern of the type lacking `..`, and without a
struct literal of the type lacking a base. Test code is out of scope. An
exception naming this rule, `partial-impl` by default, on the line directly
above the impl lifts the finding; a bare `DECISION:` or `INVARIANT:` reference
lifts nothing. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when every field is named in the impl, the impl is
derived, or an approved exception lifts the finding.
