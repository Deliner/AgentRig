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
   whose lifetime is the whole process — record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the call.
4. Rerun the reported lint check and the affected behavior checks.

A `DECISION:` or `INVARIANT:` reference on the line directly above the call keeps
it, attesting the leak is intended; without one it is a finding.

## Rule binding

The measurement flags every Rust `.keep()` or `.into_path()` method call and
every `mem::forget`, `ManuallyDrop::new` or `Box::leak` path call outside test
code without a `DECISION:` or `INVARIANT:` reference on the line directly above
it. Test code is out of scope. Rerun the reported lint check and relevant
behavior checks. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the value drops under an owner, or the leak is
justified by an owned reference.
