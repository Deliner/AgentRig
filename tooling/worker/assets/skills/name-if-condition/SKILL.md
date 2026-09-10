---
name: name-if-condition
description: Give inline boolean branch conditions a meaningful name when named-if-condition reports a finding.
---

# Make the decision predicate explicit

The name must express the state established by the predicate and its relevance
to the selected behavior. Introducing an identifier alone is insufficient.

1. Read the condition and the behavior selected by each outcome. Establish
   exactly what the condition proves in this context.
2. Name that meaning at the same scope and strength as the evidence. Do not
   claim broader permission, validity or guarantees. Derive domain vocabulary
   from the actual owner and consumers; do not invent rules to justify a name.
3. Extract the complete predicate at its original evaluation point. Preserve
   short-circuiting, order, side effects, bindings and conditional evaluation.
4. Compare name, expression and selected behavior. A generic truth label or
   restatement of the comparison is insufficient when it conceals why this
   state selects this behavior.

Stop when the meaning is explicit, the finding is resolved and affected behavior
is preserved. Do not add an abstraction merely to name a value.
Apply complexity-discipline.

## Rule binding

Repair syntax first when analysis failed. The rule accepts a name or named field
path, optionally parenthesized; calls, comparisons, negation, indexing and
combinations require a name. Rust if-let keeps its bindings; let chains retain
theirs when restructured. Python conditional expressions and comprehension
filters retain lazy evaluation; use a named helper or explicit loop only when
needed. Do not hoist an elif predicate above earlier branches. The parser cannot
judge meaning or prove Python values are bool. Rerun the reported lint check
and relevant behavior check.
