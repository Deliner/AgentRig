---
name: wrap-the-primitive
description: Replace a type alias of a numeric primitive, char or String with a newtype that the compiler tells apart, or record why the alias must stay.
---

# Give a primitive alias a type of its own

`type UserId = u64;` names a meaning, but `UserId` is `u64`: a row count, a port
or another id passes wherever a `UserId` is expected, and the compiler cannot
tell them apart. The alias suggests a check it does not perform.

1. Read where the alias is constructed and consumed, and establish what values
   it stands for and which operations it needs.
2. Declare a newtype such as `struct UserId(u64);` with a constructor, derive or
   implement only the traits its callers use, and give it the operations it
   needs as methods instead of exposing arithmetic on the raw value.
3. When the value has a constraint, check it in the constructor and keep the
   field private, as `type-driven-discipline` describes.
4. When the alias adds no meaning beyond the primitive, remove it and use the
   primitive directly.
5. When the alias must stay, such as a C type mirrored for FFI, record the
   reasoning as an owned `DECISION:`/`INVARIANT:` reference on the line directly
   above the alias.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust type alias without generics whose type is a
numeric primitive, `char` or `String`. Test code and an alias with a
`DECISION:` or `INVARIANT:` reference on the line directly above it are out of
scope. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the alias is a type of its own, removed, or
justified by an owned reference.
