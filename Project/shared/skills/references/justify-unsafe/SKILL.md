---
name: justify-unsafe
description: Justify an unsafe block or function with a recorded reference, or remove the unsafe.
---

# Record why each unsafe is sound, or remove it

Every unsafe block and unsafe function must carry a DECISION: or INVARIANT:
reference on the line directly above it. The reference points to the recorded
reason the unsafe is sound; without one the hazard is unexplained.

1. Read the unsafe construct and establish the exact obligation it takes on: the
   FFI or pointer contract it must satisfy, the arguments it builds, the return
   value it checks, and the aliasing or lifetime it relies on.
2. Prefer removing the unsafe. A safe standard API, a checked wrapper, or a crate
   that already owns the invariant discharges the obligation instead of recording
   a reason to keep it.
3. When the unsafe is genuinely required, record the soundness reason as a
   decision or bind it to an existing invariant, then put its `// DECISION: DNNN`
   or `// INVARIANT: INNN` reference on the line directly above the unsafe.
4. Keep one reason per obligation: distinct unsafe constructs that rely on the
   same recorded reason each carry that reference on their own line above.
5. Rerun the reported lint check and the affected behavior checks.

A comment that is not an exact `DECISION: DNNN` or `INVARIANT: INNN` reference
does not justify the unsafe. Do not silence the finding with a plain comment or
by widening the rule.

## Rule binding

The measurement flags every Rust unsafe block or unsafe function without a
DECISION: or INVARIANT: reference comment on the line directly above it. Rerun
the reported lint check and relevant behavior checks. Do not weaken the rule or
its selectors merely to pass. Apply complexity-discipline; stop when the finding
is resolved and the unsafe is either removed or justified.
