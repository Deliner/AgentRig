---
name: name-the-arguments
description: Give two parameters of one type distinct types or a named record so that a call cannot swap them, or record why their order cannot be confused.
---

# Make a swapped call fail to compile

Two parameters of one type, such as `fn copy(source: &Path, target: &Path)` or
`fn resize(width: u32, height: u32)`, accept each other's values. The names
exist only at the definition; a call shows two values in some order, and a call
that swaps them compiles and runs with the wrong meaning.

1. Read the fn and its callers and establish what tells the parameters apart: a
   role such as source and target, a unit such as width and height, or nothing
   because the operation is symmetric.
2. When the parameters are different kinds of value, give each a type of its
   own, such as a newtype per role, so that a swapped call fails to compile.
3. When the parameters describe one thing together, pass a named record such as
   `Size { width, height }`, so that every call names each value.
4. When one parameter is the value the operation acts on, make the fn a method
   of that value's type, so that one same-type input remains.
5. When the operation is commutative, such as `max(a, b)`, or a foreign side such
   as JNI or an ABI fixes the signature, record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the fn or its
   attributes.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust fn item or fn signature with two
non-receiver parameters of token-equal type, once per fn. A fn inside an
`impl Trait for Type` is skipped because the trait owns its signature. Test code
and a fn with a `DECISION:` or `INVARIANT:` reference on the line directly above
it or its attributes are out of scope. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when a swapped call no longer
compiles, each value is named at the call, or the signature is justified by an
owned reference.
