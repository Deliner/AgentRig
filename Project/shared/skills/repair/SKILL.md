---
name: repair
description: Repair reported configuration or verification failures.
---

# Correct the cause while preserving the governing requirement

A successful repair changes the cause of failure, not the definition of success.

1. Read the original finding, its location and the operation that produced it.
   Distinguish a failed check from unavailable or incomplete verification.
2. Identify the authoritative owner of the violated condition and applicable
   repair guidance. Preserve required behavior, user work and selection intent.
3. Make the smallest correction at that owner. Rerun the failed operation in the
   same relevant scope and compare its actual result with the original finding.
4. Stop when that failure is resolved and the affected contract is verified.

Do not weaken a threshold, selector or expected result to obtain success.
The same check identifier or another failure does not prove the same cause.
A repeated failure requires reassessing the attempted correction against the
new evidence, not accumulating workarounds.

## Project failure or guidance failure

When evidence shows that the instruction or diagnostic itself caused the error,
correct its canonical source within the authorized scope and verify the
previously failing situation. Record that evidence in the current VAC and commit;
do not create another registry or mandatory checkpoint. An unrelated failure
does not justify changing the guidance.

## Runtime binding

Use config-check for settings, lint-rules for implemented capabilities and doctor
for installation problems. Apply the named skill and execute the reported RERUN
command through the repository's configured shell adapter. For a gate check,
use just check --only CHECK_ID and preserve --staged when reported. just report
shows consecutive failures and delivered guidance, not proof of reading or a
shared cause.

When repairing lint configuration: a numeric warning is below error;
directory-entries rejects extensions; directory-architecture requires explicit
source extensions and architecture settings; unsupported selected languages are
configuration errors. When authorized work adds a rule, implemented measurements,
declared capabilities, behavioral verification and repair guidance are required.
