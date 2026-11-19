---
name: rewrite-in-allowed-language
description: Rewrite a program whose language the project does not allow in one of its allowed languages and delete the original, when program-language reports a file.
---

# Keep every program in an allowed language

The project lists the languages its programs may use in `allowed_languages` of
its program-language lint rule. Its lint rules, tests and agent guidance cover
those languages only. A program in another language escapes them, and everyone
who changes it must know one more language.

1. Read the reported file and establish what it does: its inputs, outputs,
   effects and every caller, such as a recipe, a configured command, a hook, a
   script or a document that runs or names it.
2. Choose its owner in an allowed language: the module or command that already
   owns this responsibility, or a new module next to its callers. Prefer an
   existing operation or catalog command over a new program.
3. Rewrite the behavior there, keeping the contract its callers rely on, and
   test it through its public surface when the behavior is not trivial.
4. Point every caller at the new owner and delete the reported file.
5. Vendored third-party code or a fixture whose language is the subject of a
   test is not a program of the project: exclude it from the rule with a
   `DECISION:` reference above the exclude. Never exclude the project's own
   program merely to pass.
6. Rerun lint and the affected tests.

## Rule binding

A file's extension names its language. A file without an extension is a
program when its first line starts with `#!`; its language is the interpreter
named there, after `env` and its options. Data, configuration and documents,
such as YAML, JSON, TOML and Markdown, are not programs. Adding a language to
`allowed_languages` changes project policy and needs the user's agreement; do
not widen the list or narrow the rule's selectors merely to pass. Apply
complexity-discipline; stop when the behavior lives in an allowed language and
the rule reports no file.
