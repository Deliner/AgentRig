---
name: read-or-remove-the-collection
description: Read a collection that code fills but never queries where its contents are needed, or remove it with every operation that fills it, or record why it is kept unread.
---

# Read a collection or remove it

A collection that code fills with `push`, `insert` or `extend` and never reads
costs work for nothing. Usually a step that should read it is missing or reads
another value by mistake, such as sorting a copy and then iterating the
original, so a result the program needs is silently lost.

1. Find where the contents were meant to go: a return value, a report, a later
   loop or check.
2. When they are needed, read the collection there: return it, pass it on, or
   iterate it instead of the value read by mistake.
3. When nothing needs them, remove the collection with every operation that
   fills it. Keep a call inside such an operation only when the call has an
   effect of its own.
4. When the collection is kept for what holding its elements does, such as
   guards or handles that must live until the end of the scope, record the
   reasoning as a decision with edit-decisions and put
   `#[expect(clippy::collection_is_never_read)]` on the smallest item that
   holds the collection, with its exception
   `// DECISION: DNNN allow-justification` alone on the line directly above
   the attribute. List it in the decision as
   `[exception allow-justification](FILE)`; the user approves a new exception
   through propose-protected-change, as justify-allow describes.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The rig's Clippy profile denies `clippy::collection_is_never_read` in the
library, binaries and examples of the configured Cargo manifest; tests are
outside the profile. Clippy reports a local collection, such as a `Vec` or a
`HashMap`, that the code changes but never queries, and the lint check reports
each message as an error naming this skill. Do not weaken the profile merely to
pass. Apply complexity-discipline; stop when the collection is read where its
contents are needed or removed, or an approved exception lifts the finding.
