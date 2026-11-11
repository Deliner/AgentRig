---
name: require-the-method
description: Drop a trait method's default body that does not use self, so that each implementor writes its own answer, or record why the default must stay.
---

# Let each implementor decide its answer

A trait method whose default body ignores `self`, such as
`fn priority(&self) -> u8 { 0 }` or `fn name() -> &'static str { "unknown" }`,
hands every implementor a constant it never chose. A new implementor compiles
without noticing the method, and the shared constant is wrong for it.

1. Read the trait and its implementors, and establish whether one answer is
   right for every implementor or each must decide.
2. Remove the default body so the method is required, and write the method in
   each implementor; a value that is the same for many of them is written in
   each, where a reader of that type sees it.
3. When the answer is a constant per type, an associated const without a default
   such as `const PRIORITY: u8;` states that more directly than a method.
4. When the default must stay, such as a trait published to other crates whose
   implementors a new required method would break, record the reasoning as an
   owned `DECISION:`/`INVARIANT:` reference on the line directly above the
   method.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust trait fn with a nonempty default body that
does not mention `self`. An empty body is out of scope, as are test code and a
method with a `DECISION:` or `INVARIANT:` reference on the line directly above
it. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when each implementor writes the method or the
default is justified by an owned reference.
