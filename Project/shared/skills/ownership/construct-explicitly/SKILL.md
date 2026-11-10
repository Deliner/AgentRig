---
name: construct-explicitly
description: Replace From of a raw value for a one-field newtype with a named constructor or a checking TryFrom, so that .into() cannot build it silently, or record why the conversion must stay.
---

# Build a newtype where a reader sees it

A newtype such as `struct UserId(u64)` tells values apart only while nothing
turns a raw value into it implicitly. `impl From<u64> for UserId` lets `.into()`
and `?` build a `UserId` from any `u64` wherever one is expected, so a row count
passes as an id without a visible step.

1. Read the impl and its uses, and establish whether every raw value is valid
   and where conversions happen.
2. Remove the `From` impl.
3. Add a named constructor such as `UserId::new(raw)`, or an `impl TryFrom` that
   checks the value when it has a constraint, and replace each `.into()` with
   the explicit call.
4. When an error conversion relied on `From` for `?`, map the error at that
   call, or convert from a dedicated error type instead of a raw value.
5. When the conversion must stay, such as a type whose purpose is to accept any
   raw value at an API boundary, record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the impl.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust impl of `From` with a numeric primitive,
`bool`, `char`, `String` or `&str` argument for a one-field struct of the same
file. Test code and an impl with a `DECISION:` or `INVARIANT:` reference on the
line directly above it are out of scope. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when every construction is an
explicit call or the conversion is justified by an owned reference.
