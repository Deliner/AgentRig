---
name: justify-skip
description: Re-enable or delete a disabled test, or record why it is skipped with a reference.
---

# Restore a disabled test, or record why it is skipped

A silently disabled test drops coverage without a trace: the suite stays green
while the behavior it pinned goes unchecked. Every disabled test must be
re-enabled, deleted, or carry a DECISION: or INVARIANT: reference on the line
directly above it recording why skipping it is the right call.

1. Read the test and establish what behavior it pins and why it was disabled: a
   genuine environment gap, a flaky dependency, a removed feature, or an
   oversight. A disabled test claims its behavior need not be checked now.
2. Prefer re-enabling it. Fix the flake, supply the missing fixture, or repair
   the code it exercises so the test runs and passes.
3. Delete it when the behavior is gone or another test already pins it. A test
   kept only to be skipped is dead weight that hides lost coverage.
4. When skipping is genuinely required, record the reason as a decision or bind
   it to an existing invariant, then put its `DECISION: DNNN` or `INVARIANT:
   INNN` reference on the line directly above the `#[ignore]` attribute or the
   `@pytest.mark.skip`/`skipif` decorator.
5. Rerun the reported lint check and the affected behavior checks.

The reference is the only accepted form: an exact `DECISION: DNNN` or
`INVARIANT: INNN` comment (`//` in Rust, `#` in Python) on the line directly
above the attribute or decorator. By project convention the rule does not read
the message of `#[ignore = ".."]` or a skip decorator's `reason`, even when it
says why the test is skipped: record that reason as the decision or invariant
and reference it. Any other comment does not justify the skip either. Do not
silence the finding with a plain comment or by widening the rule.

## Rule binding

The measurement flags every Rust `#[ignore]` attribute or Python
`@pytest.mark.skip`/`skipif` decorator without a DECISION: or INVARIANT:
reference comment on the line directly above it. No other comment, `#[ignore]`
message or `reason` argument is read. Rerun the reported lint check and relevant
behavior checks. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the finding is resolved and the test is
re-enabled, deleted or justified.
