---
name: extract-embedded-text
description: Move a long string literal counted by embedded-text into its own file read at compile time while preserving its exact value.
---

# Keep data in its own file

Make a code file read as code: a query, document, template, script or help text
of many lines belongs in a file of its own type, where it gets its own
highlighting, checks and history. Resolve a finding by moving the text, not by
splitting the same text across several shorter literals.

1. Read the literal and its uses. Establish its exact value, including a leading
   or trailing newline and indentation inside the delimiters, and whether it is
   a format string, a byte string or an argument of a macro.
2. Move the text into a file beside the code, named for its content and type,
   such as `report.sql`, `timings.html` or `help.txt`. Keep it in the module that
   owns it and register it where the project inventories files.
3. Replace the literal with `include_str!("report.sql")`, or `include_bytes!` for
   a byte string. The path is relative to the source file, and the value stays a
   compile-time constant with the same name and type.
4. A format string must stay a literal. Keep the short dynamic part in code and
   move the static body, or give the moved text named markers and substitute
   them explicitly at the one place that fills it.
5. Compare the old and new values, for example in a behavior check of the
   consumer, and verify the changed code through its affected checks.

Do not change the text, its line endings or its encoding while moving it. Text
held by tests as expected output stays in the test code.

## Rule binding

The measurement counts the lines one Rust string literal spans, plain, raw or
byte, from its opening to its closing delimiter, inside macro arguments too.
Literals in `#[cfg(test)]` or `#[test]` items and in the excluded test paths are
not measured. Rerun the reported lint check and relevant behavior checks. Do not
raise the threshold merely to pass. Apply complexity-discipline; stop when the
finding is resolved and the value is preserved.
