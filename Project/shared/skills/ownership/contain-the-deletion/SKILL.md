---
name: contain-the-deletion
description: Delete a directory tree through an owner that checks the path lies inside its own area, or record why the recursive deletion is intended.
---

# Give a recursive deletion an owner with a boundary

`remove_dir_all` is the most destructive file-system call: it erases whatever
the argument names, and a path built from the wrong root erases a working tree.
The call reads like `remove_file`, so nothing in the code says who owns the
deleted tree or what keeps the argument inside it.

1. Find the owner of the area being deleted: the component that creates and
   names the directory (a task area, a cache, a job directory). Delete there,
   never from a caller that only holds a path.
2. Make the owner derive the path from its own root and keep it inside that
   root before deleting (a name taken from the owner's own listing, or a
   `starts_with` check against the canonical root), so a wrong argument fails
   instead of deleting.
3. For a deletion that already is the bounded owner's own cleanup, or a
   deliberate one-off, record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the call.
4. Rerun the reported lint check and the affected behavior checks.

A `DECISION:` or `INVARIANT:` reference on the line directly above the call
keeps it, attesting the deletion has an owner and a boundary; without one it
is a finding. Test code is out of scope.

## Rule binding

The measurement flags every Rust `call_expression` outside test code whose
callee name is `remove_dir_all` — the last segment of a path such as
`std::fs::remove_dir_all` or `tokio::fs::remove_dir_all`, a bare imported or
local `remove_dir_all`, or a method `.remove_dir_all(..)` — in source and inside
macro bodies, without a `DECISION:`/`INVARIANT:` reference on the line directly
above it. `remove_file` and `remove_dir` are not flagged. Rerun the reported
lint check and relevant behavior checks. Do not weaken the rule or its
selectors merely to pass. Apply complexity-discipline; stop when every
recursive deletion is the bounded owner's call or justified by an owned
reference.
