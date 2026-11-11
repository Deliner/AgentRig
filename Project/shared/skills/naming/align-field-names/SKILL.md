---
name: align-field-names
description: Repair a struct literal field set from a variable named after another field of the same literal, by swapping it back or renaming the variables, or record why the exchange is intended.
---

# Let each field take a value named for it

In `Range { start: end, end: start }` or `Size { width: height, .. }` a field is
set from a variable named after a sibling field. Either the values were swapped
by mistake, or the exchange is intended and reads exactly like that mistake.

1. Read the literal and where its variables come from, and establish whether the
   exchange is intended.
2. When it is a mistake, set each field from the variable of its own name, using
   field shorthand such as `Range { start, end }`, and check the callers that
   relied on the swapped values.
3. When the exchange is intended, such as a transpose or a reversed range, name
   the variables for what they hold at this point, such as
   `let Size { width: old_width, height: old_height } = size;` followed by
   `Size { width: old_height, height: old_width }`, so that the literal states
   the exchange.
4. When renaming would obscure a well-known form, record the reasoning as an
   owned `DECISION:`/`INVARIANT:` reference on the line directly above the
   literal.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust struct literal field initialized by a plain
identifier that is not the field's own name but names another field of the same
literal, once per literal. Test code and a literal with a `DECISION:` or
`INVARIANT:` reference on the line directly above it are out of scope. Do not
weaken the rule or its selectors merely to pass. Apply complexity-discipline;
stop when each field takes its own name, the variables state the exchange, or
it is justified by an owned reference.
