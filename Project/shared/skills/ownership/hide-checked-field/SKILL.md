---
name: hide-checked-field
description: Make a field that a checking constructor fills private and expose it through a method, so that no code can build or change the value around the check, or record why the field must stay visible.
---

# Keep a checked value behind its constructor

A struct whose constructor, `TryFrom` or `FromStr` refuses bad input promises
that every value passed the check. A `pub` or `pub(crate)` field breaks that
promise: code elsewhere can assign the field or build the struct with a literal
and skip the check entirely.

1. Read the constructor and the struct, and establish which fields the check
   protects and who reads or writes them.
2. Remove the visibility modifier from each checked field; when any field stays
   visible, check that a struct literal outside the module is still impossible.
3. Give readers a method such as `fn port(&self) -> u16`, and give writers a
   method that repeats the check, or construct a new value through the
   constructor.
4. When the field must stay visible, such as a `#[repr(C)]` layout that foreign
   code reads, record the reasoning as an owned `DECISION:`/`INVARIANT:`
   reference on the line directly above the struct or its attributes.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust struct whose checking constructor stores a
checked parameter in a field with a visibility modifier, once per struct. A
checking constructor is a fn without `self` in the struct's inherent impl, an
`impl TryFrom` or an `impl FromStr`, returning a `Result` or `Option` of `Self`,
that refuses on a parameter. Test code and a struct with a `DECISION:` or
`INVARIANT:` reference on the line directly above it or its attributes are out
of scope. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when every checked field is private or the visible
field is justified by an owned reference.
