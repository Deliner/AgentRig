---
name: handle-error
description: Handle a swallowed exception, or record why the error is safe to ignore with a reference.
---

# Handle the error, or record why it is safe to ignore

A Python except clause whose body is only `pass` swallows the error silently:
the failure it catches leaves no trace and the program continues as if nothing
went wrong. Decide what the handler owes its caller instead of discarding it.

1. Read the `try` body and establish which exception the handler catches and
   what state it leaves behind: a half-written file, an open resource, a retry
   the caller still expects, or a value the caller will go on to read.
2. Prefer handling the error. Recover, retry, clean up, log it, or re-raise a
   clearer error so the failure stays visible. Narrow a broad `except` to the
   one exception that is genuinely expected.
3. When the error is genuinely safe to ignore, record why as a decision or bind
   it to an existing invariant, then put its `# DECISION: DNNN` or
   `# INVARIANT: INNN` reference on the line directly above the `pass`.
4. Keep one reason per handler: distinct swallowing handlers that rely on the
   same recorded reason each carry that reference on their own line above.
5. Rerun the reported lint check and the affected behavior checks.

A comment that is not an exact `DECISION: DNNN` or `INVARIANT: INNN` reference
does not justify the swallow. Do not silence the finding with a bare comment, a
broadened `except`, or by widening the rule.

## Rule binding

The measurement flags every Python except clause whose body is only `pass`
without a DECISION: or INVARIANT: reference comment on the line directly above
the pass. Rerun the reported lint check and relevant behavior checks. Do not
weaken the rule or its selectors merely to pass. Apply complexity-discipline;
stop when the finding is resolved and the error is either handled or justified.
