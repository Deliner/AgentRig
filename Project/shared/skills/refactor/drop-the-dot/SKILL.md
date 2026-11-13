---
name: drop-the-dot
description: Write the extension passed to with_extension or set_extension without its leading dot, or record why the dotted literal is meant.
---

# Give the extension without its dot

`Path::with_extension` and `PathBuf::set_extension` insert the dot themselves.
`path.with_extension(".json")` therefore yields `name..json`, a file no other
code looks for, and nothing fails until the file is missing where it is
expected.

1. Remove the leading dot from the literal: `path.with_extension("json")`.
2. When the receiver is a type of the project or of a library whose method
   expects the dot, record that as an owned `DECISION:`/`INVARIANT:` reference
   on the line directly above the call.
3. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust `with_extension` or `set_extension` method
call whose sole argument is a string literal, plain or raw, starting with a dot,
outside test code, without a `DECISION:`/`INVARIANT:` reference on the line
directly above it. Types are not resolved, so a method of the same name on
another type counts. Do not weaken the rule or its selectors merely to pass.
Apply complexity-discipline; stop when the literal has no leading dot or an
owned reference justifies it.
