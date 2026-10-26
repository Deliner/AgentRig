---
name: condense-markdown
description: Bring README and AGENTS.md within their size and width limits, and remove Markdown from places that forbid it, by keeping only what the reader needs.
---

# If it does not fit, something in it is superfluous

A size, width or placement finding is evidence of excess content, not a reason
to raise the limit, split the file, rename it or move the text elsewhere unread.

1. Name the reader and what they must do with this file. A README orients: what
   the unit is, how to use it, where to look next. AGENTS.md states rules an
   agent cannot infer from code, configuration, skills or command output.
2. Delete what that reader does not need: repetition, history, generic advice,
   and details recoverable from code, configuration, `--help` or `lint-rules`.
3. Keep durable knowledge with its owner instead of the file: a contextual
   choice belongs in a decision, an executable guarantee in an invariant with
   its test, a repeatable procedure in a skill, and the role of a file or
   directory in its architecture.yaml. Link the owner only when the reader
   needs it.
4. Wrap prose within the width; shorten wide tables and link text rather than
   breaking their syntax.
5. For a forbidden file, apply the same steps to its content, then delete it.

Stop when the file is within its limits and every remaining line serves its
reader. Rerun the reported lint check. Apply complexity-discipline.
