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
   code reads, record the reasoning as a decision with edit-decisions and put
   its exception `// DECISION: DNNN RULE`, RULE being the rule ID the finding
   names, alone on the line directly above the struct, below its attributes.
   List it in the decision as `[exception RULE](FILE)`; the user approves a new
   exception through propose-protected-change.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust struct whose checking constructor stores a
checked parameter in a field with a visibility modifier, once per struct. A
checking constructor is a fn without `self` in the struct's inherent impl, an
`impl TryFrom` or an `impl FromStr`, returning a `Result` or `Option` of `Self`,
that refuses on a parameter. Test code is out of scope. An exception naming
this rule, `open-checked-field` by default, on the line directly above the
struct, below its attributes, lifts the finding; a bare `DECISION:` or
`INVARIANT:` reference lifts nothing. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when every checked field is
private or an approved exception lifts the finding.
