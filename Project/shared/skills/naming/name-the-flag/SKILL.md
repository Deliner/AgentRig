---
name: name-the-flag
description: Give a bool among the positions of a tuple struct or tuple variant a named field whose enum names both states, or record why its position is its meaning.
---

# Name the flag a position hides

A tuple struct or tuple variant with a `bool` among two or more positions, such
as `Save(PathBuf, bool)`, is built as `Save(path, true)`. The `true` does not
say what it switches, so a constructor that passes the inverted value or a
pattern that reads it the other way compiles unnoticed.

1. Read the type, its constructors and its patterns, and establish what `true`
   and `false` mean there, such as overwrite and keep existing.
2. Declare an enum naming both states, such as
   `enum SaveMode { Overwrite, KeepExisting }`, and turn the positions into
   named fields, such as `Save { path: PathBuf, mode: SaveMode }`.
3. Rewrite every constructor and pattern to the names and variants, and every
   `.0`/`.1` access to the field name. Where code tested the flag, match the
   enum so that each state is handled by its name.
4. When the position is fixed by an external format, such as a tuple variant
   whose order is a serialized message layout, record the reasoning as a
   decision with edit-decisions and put its exception
   `// DECISION: DNNN RULE`, RULE being the rule ID the finding names, alone
   on the line directly above the type or variant, below its attributes. List
   it in the decision as `[exception RULE](FILE)`; the user approves a new
   exception through propose-protected-change.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust tuple struct or tuple enum variant with two
or more fields, one of which has exactly the type `bool`. `Option<bool>` in a
position stays with `ambiguous-option` and `name-the-states`; `&bool` is not
counted. A struct with a single field, such as `struct Verbose(bool)`, names the
flag by its type and is out of scope. Two `bool` positions are also reported by
`same-type-positions`; one repair satisfies both, while an exception lifts
only the rule it names. Test code is out of scope. An exception naming this
rule, `positional-flag` by default, on the line directly above the type or
variant, below its attributes, lifts the finding; a bare `DECISION:` or
`INVARIANT:` reference lifts nothing. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when every flag is a named
field of an enum type, or an approved exception lifts the finding.
