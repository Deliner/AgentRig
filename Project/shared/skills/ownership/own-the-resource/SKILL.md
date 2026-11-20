---
name: own-the-resource
description: Let a Rust value drop or hold its guard instead of defeating RAII with keep, into_path, mem::forget, ManuallyDrop or Box::leak, or record why the leak is intended.
---

# Own the resource instead of defeating its cleanup

`.keep()` and `.into_path()` dissolve a `tempfile` guard so its file or
directory is never deleted on drop; `mem::forget` skips a value's destructor;
`ManuallyDrop::new` suppresses it; `Box::leak` hands heap to a `'static`
reference that is never freed. Each one makes a resource outlive its owner with
no visible cleanup, so a later reader cannot tell who releases it.

1. Prefer letting the value drop: hold the guard in a binding that lives exactly
   as long as the resource must, and let the end of scope release it.
2. If the resource must outlive the current scope, give it an owner that drops
   it — return the guard, store it in a struct whose `Drop` frees it, or hand it
   to code that closes it — instead of forgetting or leaking it.
3. If the leak is genuinely intended — a deliberately persisted path, or a value
   whose lifetime is the whole process — record the reasoning as a decision
   with edit-decisions and put its exception `// DECISION: DNNN RULE`, RULE
   being the rule ID the finding names, alone on the line directly above the
   call. List it in the decision as `[exception RULE](FILE)`; the user approves
   a new exception through propose-protected-change.
4. Rerun the reported lint check and the affected behavior checks.

The method forms are recognized by name alone, without checking the receiver:
every `.keep()` or `.into_path()` call is flagged, also on a type that owns no
cleanup, such as a directory-walk entry's `into_path`. When the receiver
releases nothing on drop, nothing leaks; the exception on the line directly
above the call records that.

## Rule binding

The measurement flags every Rust `.keep()` or `.into_path()` method call,
matched by name without checking the receiver's type, and every `mem::forget`,
`ManuallyDrop::new` or `Box::leak` path call outside test code. Test code is
out of scope. An exception naming this rule, `disabled-raii` by default, on
the line directly above the call lifts the finding, attesting the leak is
intended; a bare `DECISION:` or `INVARIANT:` reference lifts nothing. Rerun the
reported lint check and relevant behavior checks. Do not weaken the rule or its
selectors merely to pass. Apply complexity-discipline; stop when the value
drops under an owner, or an approved exception lifts the finding.
