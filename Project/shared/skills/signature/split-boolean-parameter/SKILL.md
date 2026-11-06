---
name: split-boolean-parameter
description: Replace a boolean parameter with named modes or a split when boolean-parameter reports a finding.
---

# Name the modes a boolean parameter hides

A boolean parameter either makes one function carry two behaviors or passes a
true/false fact, and in both cases a call site passes only a boolean, a literal
or a computed value, whose meaning lives in the declaration. Resolve the finding
by naming the modes or the fact with a type, not by hiding the flag behind an
alias.

1. Read the parameter and the two behaviors it selects. Find every call site and
   the value each one passes, and whether the flag selects a mode or merely
   records a fact that is used later.
2. When the two behaviors are largely separate, split into two functions named
   for each outcome and route common work through a shared helper. The caller
   then names the behavior directly instead of passing a flag.
3. When the mode threads through several layers, name it with a dedicated
   two-variant type (a Rust enum, a Python enum) and accept that instead of the
   bool; convert each call site to the named variant.
4. When the value is data that records a true/false fact rather than selecting
   behavior, the rule flags it all the same: name the fact with a dedicated
   type, such as a two-variant enum whose variants state it or a named struct
   that carries it with its related data, and accept that instead of the bool.
5. Verify the changed function through its call sites and affected behavior checks.

Do not keep the bool and alias it, drop its type annotation, accept the named
type only to branch straight back to a bool, or drop a mode to lower the count.
The replacement must leave the same behavior on every path.

## Rule binding

The rule reads parameter declarations, never call arguments, and flags a mode and
a fact alike. In Rust it flags each parameter of a function, a trait or extern
function signature or a closure whose declared type is exactly the primitive
`bool`; `Option<bool>`, `&bool`, a named type and an untyped closure parameter
are not flagged. In Python it flags each parameter of a `def` or `lambda` that is
annotated exactly `bool`, whatever its default, or that defaults to the literal
`True` or `False`, whatever its annotation; `bool | None`, `Optional[bool]`, a
quoted `"bool"` and an unannotated parameter with a computed default such as
`flag=read_flag()` are not flagged. A named type that replaces the `bool` and any
`True`/`False` default clears the finding. Rerun the reported lint check and
relevant behavior checks. Do not remove the rule or weaken its selectors merely
to pass. Apply complexity-discipline; stop when the finding is resolved and
behavior is preserved.
