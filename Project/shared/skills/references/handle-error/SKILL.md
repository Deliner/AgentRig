---
name: handle-error
description: Handle a swallowed error, or record why it is safe to ignore with a reference.
---

# Handle the error, or record why it is safe to ignore

A Python except clause whose body is only `pass` swallows the error silently:
the failure it catches leaves no trace and the program continues as if nothing
went wrong. A Rust `read_dir` chain collapsed by `.flatten()`,
`.filter_map(Result::ok)`, `.filter_map(|e| e.ok())` or `.map_while(Result::ok)`
drops every entry that failed to read, so an unreadable directory silently
reads as empty. A Rust `Err(_)` arm that answers `Ok(false)` or `Ok(true)` in a
function returning `Result<bool, …>` turns a failed check into a verdict.
Decide what the handler owes its caller instead of discarding the error.

1. Read the guarded operation and establish which error it catches and what
   state it leaves behind: a half-written file, an open resource, a retry the
   caller still expects, a listing the caller will go on to read, or a yes/no
   answer the caller will act on.
2. Prefer handling the error. Recover, retry, clean up, log it, or re-raise a
   clearer error so the failure stays visible. Narrow a broad `except` to the
   one exception that is genuinely expected. In Rust, iterate with
   `for entry in read_dir(..)? { let entry = entry?; … }` or collect the
   entries into `Result<Vec<_>, _>`, and propagate the error from a check with
   `?` or `Err(error) => Err(error.into())` instead of answering a verdict.
3. When the error is genuinely safe to ignore, record why as a decision or bind
   it to an existing invariant, then put its `DECISION: DNNN` or
   `INVARIANT: INNN` reference comment on the line directly above the `pass`,
   the chain or the arm.
4. Keep one reason per handler: distinct swallowing handlers that rely on the
   same recorded reason each carry that reference on their own line above.
5. Rerun the reported lint check and the affected behavior checks.

A comment that is not an exact `DECISION: DNNN` or `INVARIANT: INNN` reference
does not justify the swallow. Do not silence the finding with a bare comment, a
broadened `except`, or by widening the rule.

## Rule binding

The measurement flags every Python except clause whose body is only `pass`,
every Rust `read_dir` call chain collapsed by one of the listed operations, and
every Rust `match`/`if let` arm `Err(_)`/`Err(..)` whose body is `Ok(true)`,
`Ok(false)` or a `return` of one inside a function declared to return
`Result<bool, …>`, outside Rust test code, without a DECISION: or INVARIANT:
reference comment on the line directly above. Rerun the reported lint check and
relevant behavior checks. Do not weaken the rule or its selectors merely to
pass. Apply complexity-discipline; stop when the finding is resolved and the
error is either handled or justified.
