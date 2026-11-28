---
name: mark-must-use
description: Mark a public method that takes self and returns Self as must_use, so that a call cannot drop the value it returns unnoticed, or record why callers may drop it.
---

# Mark a method that returns `Self` as `must_use`

A method such as `fn with_limit(self, limit: Duration) -> Self` returns a
changed value instead of changing the receiver. A call written as a statement,
`settings.with_limit(limit);`, drops that value, and the change is lost without
a warning.

1. Read what the method returns: the value the caller asked for, or the
   previous value of a receiver it changed.
2. For the value the caller asked for, put `#[must_use]` on the method, or on
   the type when no value of it should ever be dropped. The compiler then
   warns at every call that drops the value.
3. When callers may drop the value, such as the previous value that a
   `replace` or `take` on `&mut self` returns, record the reasoning as a
   decision with edit-decisions and put
   `#[expect(clippy::return_self_not_must_use)]` on the method, with its
   exception `// DECISION: DNNN allow-justification` alone on the line
   directly above the attribute. List it in the decision as
   `[exception allow-justification](FILE)`; the user approves a new exception
   through propose-protected-change, as justify-allow describes.
4. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The rig's Clippy profile denies `clippy::return_self_not_must_use` in the
library, binaries and examples of the configured Cargo manifest; tests are
outside the profile. Clippy reports a method of the crate's public API that
takes `self` in any form and returns `Self`, when neither the method nor the
type is `#[must_use]`; a function without `self`, such as a constructor, is not
reported. The lint check reports each message as an error naming this skill.
Do not weaken the profile merely to pass. Apply complexity-discipline; stop
when the method or its type is `#[must_use]`, or an approved exception lifts
the finding.
