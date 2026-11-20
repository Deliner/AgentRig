---
name: name-the-positions
description: Name the fields of a tuple struct or tuple variant that holds two values of one type, or record why the order of its positions is its meaning.
---

# Name the positions a swap would confuse

A tuple struct or tuple variant with two fields of one type, such as
`struct Span(usize, usize)` or `Move(i32, i32)`, tells its values apart only by
`.0` and `.1` or by their order in a pattern. A constructor or a pattern that
swaps them compiles, and a reader has to look up which position means what.

1. Read the type and its constructors and patterns, and establish what each
   position means.
2. Turn the positions into named fields, such as `struct Span { start: usize,
   end: usize }` or `Move { dx: i32, dy: i32 }`, and update every constructor,
   pattern and `.0`/`.1` access to the names.
3. When the positions are different kinds of value, a type per position, such as
   a newtype, also makes a swap fail to compile; use it when the values travel
   on their own beyond this type.
4. When the order is the meaning itself, such as the components of a vector in a
   fixed external format, record the reasoning as a decision with
   edit-decisions and put its exception `// DECISION: DNNN RULE`, RULE being
   the rule ID the finding names, alone on the line directly above the type or
   variant, below its attributes. List it in the decision as
   `[exception RULE](FILE)`; the user approves a new exception through
   propose-protected-change.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust tuple struct or tuple enum variant with two
fields of token-equal type. A tuple in a signature stays with `tuple-as-record`
and its `name-the-record` repair. Test code is out of scope. An exception
naming this rule, `same-type-positions` by default, on the line directly above
the type or variant, below its attributes, lifts the finding; a bare
`DECISION:` or `INVARIANT:` reference lifts nothing. Do not weaken the rule or
its selectors merely to pass. Apply complexity-discipline; stop when every
position has a name or a type of its own, or an approved exception lifts the
finding.
