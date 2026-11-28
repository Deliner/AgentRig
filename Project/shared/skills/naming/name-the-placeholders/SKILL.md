---
name: name-the-placeholders
description: Name every placeholder of a format string with two or more positional ones, binding expressions to names first, and write a plain variable into its placeholder, or record why the positions must stay.
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
   foreign side fixes, record the reasoning as a decision with edit-decisions
   and put its exception `// DECISION: DNNN RULE`, RULE being the rule ID the
   finding names, alone on the line directly above the macro call. List it in
   the decision as `[exception RULE](FILE)`; the user approves a new exception
   through propose-protected-change.
6. Rerun the reported lint check and the affected behavior checks.

## Clippy profile

The rig's Clippy profile denies `clippy::uninlined_format_args` in the
library, binaries and examples of the configured Cargo manifest; tests are
outside the profile. Clippy reports a format macro that passes a plain
variable as an argument, also to a single placeholder, a width or a precision,
such as `format!("{}", name)` or `format!("{:1$}", value, width)`, unless
another argument of the call is an expression; the lint check reports each
message as an error naming this skill. Write the variable into its
placeholder: `format!("{name}")` or `format!("{value:width$}")`. The written
form shows the same value, so no exception keeps the argument; do not suppress
the lint or weaken the profile merely to pass.

## Rule binding

The measurement reports every Rust `format`, `print`, `println`, `eprint`,
`eprintln`, `write`, `writeln`, `panic` or `format_args` macro whose first string
literal has two or more placeholders without a name; `{}` and `{0}` are without
a name, and one positional placeholder is allowed. Test code is out of scope.
An exception naming this rule, `positional-format` by default, on the line
directly above the macro call lifts the finding; a bare `DECISION:` or
`INVARIANT:` reference lifts nothing. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when every placeholder names
its value or an approved exception lifts the finding.
