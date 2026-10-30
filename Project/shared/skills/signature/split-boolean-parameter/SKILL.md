---
name: split-boolean-parameter
description: Replace a boolean parameter with named modes or a split when boolean-parameter reports a finding.
---

# Name the modes a boolean parameter hides

A boolean parameter makes one function carry two behaviors and every call site
read as a bare `true`/`false` whose meaning lives elsewhere. Resolve the finding
by naming the two modes, not by hiding the flag behind an alias.

1. Read the parameter and the two behaviors it selects. Find every call site and
   which literal each one passes, and whether the flag selects a mode or merely
   records a fact that is used later.
2. When the two behaviors are largely separate, split into two functions named
   for each outcome and route common work through a shared helper. The caller
   then names the behavior directly instead of passing a flag.
3. When the mode threads through several layers, name it with a dedicated
   two-variant type (a Rust enum, a Python enum) and accept that instead of the
   bool; convert each call site to the named variant.
4. When the value is data that records a true/false fact rather than selecting
   behavior, carry it as a field or return value, out of the parameter list.
5. Verify the changed function through its call sites and affected behavior checks.

Do not keep the bool and alias it, accept the named type only to branch straight
back to a bool, or drop a mode to lower the count. The replacement must leave the
same behavior on every path.

## Rule binding

The rule flags a parameter typed as the Rust `bool` primitive, or annotated
`bool` or defaulting to `True` or `False` in Python; `Option<bool>`, `&bool` and
non-literal defaults are not flagged, so a named type clears the finding. Rerun
the reported lint check and relevant behavior checks. Do not remove the rule or
weaken its selectors merely to pass. Apply complexity-discipline; stop when the
finding is resolved and behavior is preserved.
