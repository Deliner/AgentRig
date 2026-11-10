---
name: lock-the-data
description: Move the data a Mutex<()> or RwLock<()> protects inside the lock, or record which external resource the unit lock serializes.
---

# Put the protected data inside the lock

A `Mutex<()>` or `RwLock<()>` guards nothing the compiler can see. Which data
it protects is known only from comments or habit, so code that touches that
data without taking the lock compiles, and a later change easily forgets the
lock. A Rust lock owns its data so that the guard is the only way to reach it.

1. Find the data the lock serializes: the fields or statics that every holder
   of the guard reads or writes.
2. Move that data into the lock, such as `Mutex<State>` or `RwLock<Cache>`, and
   reach it only through the guard, so access without the lock does not
   compile. Group fields that change together in one struct.
3. When the lock serializes something outside the program's memory, such as a
   file, a device, a child process or a library call that is not thread-safe,
   put the handle of that resource inside the lock where one exists, such as
   `Mutex<File>`.
4. When no handle exists and the unit lock is the right owner, record which
   resource it serializes as an owned `DECISION:`/`INVARIANT:` reference on the
   line directly above the type or the constructor.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust `Mutex<()>` or `RwLock<()>` type from any
path, and every `Mutex::new(())` or `RwLock::new(())` call that initializes a
`let` binding without a type annotation, directly or as the argument of
another call such as `Arc::new`, so one site is reported once. Test code and a
site with a `DECISION:` or `INVARIANT:` reference on the line directly above it
are out of scope. Rerun the reported lint check and relevant behavior checks.
Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the lock owns the data it protects or the
unit lock is justified by an owned reference.
