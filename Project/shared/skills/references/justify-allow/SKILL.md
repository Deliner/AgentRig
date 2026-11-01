---
name: justify-allow
description: Remove a broad Rust allow attribute and fix the finding, or record why it is suppressed with a reference.
---

# Remove a broad allow, or record why the lint is suppressed

A broad `#[allow(...)]` or `#![allow(...)]` switches a compiler or Clippy lint
off for its scope: the gate stays green while the finding it would raise goes
unaddressed. An agent can quiet a rule instead of fixing the code. Every allow
must be removed with its finding fixed, or carry a DECISION: or INVARIANT:
reference on the line directly above it recording why the suppression is right.

1. Read the allow and the code it covers; establish which lint it silences and
   what that lint would report. An allow claims the finding need not be fixed.
2. Prefer removing the allow and fixing the finding: delete the dead code,
   repair the unused binding, handle the flagged case, or narrow the construct.
3. Narrow a still-needed allow to the smallest scope and the exact lint it must
   cover, rather than a broad crate or module `#![allow]`.
4. When the suppression is genuinely required — a deprecated upstream API with
   no replacement, a generated construct — record the reason as a decision or
   bind it to an existing invariant, then put its `DECISION: DNNN` or
   `INVARIANT: INNN` reference on the line directly above the attribute.
5. Rerun the reported lint check and the affected behavior checks.

A comment that is not an exact `DECISION: DNNN` or `INVARIANT: INNN` reference
does not justify the allow. Do not silence the finding with a plain comment or
by widening the rule.

## Rule binding

The measurement flags every Rust `#[allow(...)]` or `#![allow(...)]` attribute
whose path is `allow` without a DECISION: or INVARIANT: reference comment on the
line directly above it. Rerun the reported lint check and relevant behavior
checks. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the allow is removed or justified.
