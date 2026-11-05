---
name: name-the-record
description: Replace a tuple in a function parameter or return, standing alone or nested in a generic or slice type, with a named struct, or record why the small tuple is intended.
---

# Name the record a signature tuple hides

A tuple in a parameter or return position is an unnamed record: its fields have
positions but no names, and several fields packed into one tuple parameter slip
past the `parameter-count` and `boolean-parameter` budgets. Callers and the body
read the fields by index, so the meaning lives only in the author's head. The
same record hides inside `Vec<(String, u64, bool)>`, `Option<(…)>` or `&[(…)]`:
three or more positions with a primitive, `String` or `&str` among them is a
row of a table, not a pair.

1. Give the tuple a named struct whose fields name each position, and take or
   return that struct. Reuse an existing record when one already models the data.
2. For a value the function builds to return, a dedicated return struct names the
   result; for several related inputs, one parameter struct names them together.
3. For a genuine coordinate pair or a deliberate small tuple, record the reasoning
   as an owned `DECISION:`/`INVARIANT:` reference on the line directly above the
   function or closure.
4. Rerun the reported lint check and the affected behavior checks.

A `DECISION:` or `INVARIANT:` reference on the line directly above the function
or closure keeps the tuple, attesting the unnamed record is intended; without
one it is a finding. Test code is out of scope.

## Rule binding

The measurement flags every Rust `function_item` or `function_signature_item`
outside test code whose parameters include a tuple-typed parameter or whose
return type is a tuple, and every function, signature or closure whose parameter
or return type nests, inside a generic or slice type at any depth, a tuple of
three or more elements with at least one primitive, `String` or `&str` element,
without a `DECISION:`/`INVARIANT:` reference on the line directly above it. A
unit `()` is not a tuple, so `-> ()` and `Result<(), E>` are not flagged; a `()`
element of a nested tuple counts toward its three elements but is not a
primitive element. A pair such as `Option<(A, B)>`, a nested tuple without a
primitive element, a bare `&(A, B, u8)`, a `type` alias and a struct field are
not flagged either. Rerun the reported lint check and relevant behavior checks.
Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the tuple is a named struct or justified by an
owned reference.
