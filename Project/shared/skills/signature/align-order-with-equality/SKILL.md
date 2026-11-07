---
name: align-order-with-equality
description: Make a Rust type's manual Ord or PartialOrd compare the same fields as its equality, through a shared key or derives, or record why a field derived from the compared ones is left out.
---

# Align order with equality

The standard library requires `a == b` exactly when `a.cmp(&b) == Equal`, and
the same for `partial_cmp` with `Some(Equal)`. When the order reads fewer
fields than equality, two values can compare `Equal` yet differ: a `BTreeSet`
keeps one of them, `sort` followed by `dedup` drops a distinct value and
binary search finds the wrong element. When the order reads more fields, equal
values sort apart and lookups by an equal key miss.

1. Read the finding's field lists: `equality-only` fields take part in `==`
   but not in the order, `order-only` fields take part in the order but not
   in `==`.
2. Decide which set is the type's identity. If every field is, derive both
   `PartialEq`/`Eq` and `PartialOrd`/`Ord`, or compare all fields in the
   manual order in a meaningful sequence.
3. If only some fields are the identity, implement `PartialEq` by hand on
   those same fields, or express both relations through one key method such
   as `fn key(&self) -> (&str, u32)` used by `eq`, `cmp` and `Hash`.
4. Let `partial_cmp` delegate with `Some(self.cmp(other))` so the two
   orderings cannot drift.
5. Keep the difference only when the extra field is derived from the compared
   ones, such as a cached hash, or when values with equal compared fields
   cannot coexist by construction. Record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the impl.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags a type's manual `impl Ord`, or its manual
`impl PartialOrd` when the same file has no manual `Ord` for it, when the
`self` fields read by `cmp` or `partial_cmp` differ from the fields equality
reads: those read by the `eq` of a manual `PartialEq`, or every named field of
a struct deriving `PartialEq` in the same file. Reads follow `self.method()`
calls into methods of the same type defined in that file. A comparison is not
judged when it uses `self` in any other way, calls a method defined elsewhere,
belongs to a tuple struct, or has no equality visible in the file; an impl of
a trait with type arguments, such as `PartialOrd<i32>`, is not the same-type
contract. Equality by an expression such as `self.a % 10` is not told apart
from a plain field read. Test code and an impl with a `DECISION:` or
`INVARIANT:` reference on the line directly above are out of scope. Rerun the
reported lint check and relevant behavior checks. Do not weaken the rule or
its selectors merely to pass. Apply complexity-discipline; stop when order
and equality read the same fields or the difference is justified by an owned
reference.
