---
name: name-the-placeholders
description: Name every placeholder of a format string with two or more positional ones, binding expressions to names first, or record why the positions must stay.
---

# Tie each placeholder to its value by name

`format!("{} -> {}", from, to)` pairs placeholders with arguments only by count
and order. Swapping two arguments, or inserting one, prints the wrong value
without a compile error, and a reader has to count to see which value goes
where.

1. Read the format string and its arguments and establish what each placeholder
   shows.
2. Write each plain variable into its placeholder: `format!("{from} -> {to}")`.
3. Bind each other expression to a named `let` first, or pass it as a named
   argument: `format!("{from} -> {to}", from = edge.start(), to = edge.end())`.
4. Keep format specifiers after the name, such as `{width:>8}` or `{value:?}`.
5. When the positions must stay, such as a template whose argument order a
   foreign side fixes, record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the macro call.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust `format`, `print`, `println`, `eprint`,
`eprintln`, `write`, `writeln`, `panic` or `format_args` macro whose first string
literal has two or more placeholders without a name; `{}` and `{0}` are without
a name, and one positional placeholder is allowed. Test code and a call with a
`DECISION:` or `INVARIANT:` reference on the line directly above it are out of
scope. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when every placeholder names its value or the
positions are justified by an owned reference.
