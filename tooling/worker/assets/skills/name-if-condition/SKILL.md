---
name: name-if-condition
description: Give inline boolean branch conditions a meaningful name when named-if-condition reports a finding.
---

Read the reported location and condition. If parsing failed, repair syntax first.

Extract the complete boolean expression immediately before its evaluation and name the reason for branching: should_retry, has_capacity, can_publish. Preserve short-circuiting, evaluation order, side effects and bindings; do not hoist an elif expression above earlier branches.

The rule accepts a name or a named field path, with optional parentheses. Calls, comparisons, negation, indexing and boolean combinations need a name. A simple Rust if let retains its bindings; a let chain requires restructuring without losing them. Python conditional expressions and comprehension filters are checked too: use a small named helper or explicit loop only when needed to preserve lazy evaluation.

Do not replace expressions with vague names such as condition or flag merely to satisfy syntax. The parser cannot judge names or prove Python values are bool. Rerun just lint and the relevant behavior check.
