---
name: keep-pairs-together
description: Keep values that belong together in one record instead of two collections paired by position, so that an index found in one collection never reads another, or record why the parallel collections must stay.
---

# Keep each pair in one record

`let role = [movement, jump, look].iter().position(|rect| rect.contains(point))`
followed by `[MOVEMENT, JUMP, LOOK][role]` pairs a rectangle with its role only
by order. The pairing is written twice, so reordering, inserting or removing an
item in one list silently gives the other list's values to the wrong items.

1. Find both collections and every place that relies on their common order.
2. Put each pair into one record: a list of structs holding both values, such as
   `Zone { rect, role }`, or a field of the item that already exists, and build
   it where the two lists were built.
3. Search the record itself, with `find` or `iter().find(..)`, and read the
   second value from the found record instead of indexing the other collection.
4. When the collections must stay separate, such as buffers handed to an
   external API that expects parallel arrays, record that requirement as a
   decision with edit-decisions and put its exception
   `// DECISION: DNNN RULE`, RULE being the rule ID the finding names, alone
   on the line directly above the reported read. List it in the decision as
   `[exception RULE](FILE)`; the user approves a new exception through
   propose-protected-change.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust index expression, and every `.get(..)` or
`.get_mut(..)` call with one argument, whose index is the result of a
one-argument `.position(..)` called on `.iter()`, `.iter_mut()` or
`.into_iter()` of a collection with other text. The result counts directly,
after `?` and method calls on it, through a name bound from it by `let`,
`let .. else` or `if let`, and through a parameter of a closure passed to a
method called on it. A rebinding to another name, a project's own `position`
method and an iterator adapted by `skip`, `zip` or similar are not followed.
Test code is out of scope. An exception naming this rule, `parallel-index` by
default, on the line directly above the reported read lifts the finding; a bare
`DECISION:` or `INVARIANT:` reference lifts nothing. Do not weaken the rule or
its selectors merely to pass. Apply complexity-discipline; stop when each pair
lives in one record or an approved exception lifts the finding.
