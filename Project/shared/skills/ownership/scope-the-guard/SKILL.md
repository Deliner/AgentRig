---
name: scope-the-guard
description: Replace a visible function that returns a lock guard or a RefCell borrow with one that runs a closure over the guarded value, or record why handing out the guard is the type's purpose.
---

# Keep the guard inside the function that takes it

A function that returns a `MutexGuard`, `RwLockReadGuard`, `RwLockWriteGuard`,
`Ref` or `RefMut` lets each caller keep the lock or the `RefCell` borrow for as
long as it holds the value. Two calls in one expression, such as
`a.state().len() + a.state().len()` on a `Mutex`, deadlock, and a second
`borrow_mut()` while a `Ref` is alive panics, while the build stays clean. An
`async` caller can also keep the guard across an `.await`.

1. Read the function and its callers, and establish what each caller does with
   the guard.
2. Replace the function with one that takes a closure and returns its result,
   such as `pub fn with_state<R>(&self, f: impl FnOnce(&mut State) -> R) -> R`,
   which locks or borrows, calls `f` and releases the guard before it returns.
3. Rewrite each caller, such as `x.state().field` to
   `x.with_state(|state| state.field)`. The closure cannot return a reference
   into the guarded value, so a caller that needs a value afterwards copies or
   clones it out inside the closure.
4. When handing out the guard is the type's purpose, such as a lock wrapper
   whose API is its `lock()` method, or the returned type only shares the name
   and holds no lock or cell borrow, record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the function
   or its attributes.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement reports every Rust `fn` with a visibility modifier, `pub`,
`pub(crate)`, `pub(super)` or `pub(in ..)`, whose return type contains, at any
depth, a generic type whose last name is `MutexGuard`, `RwLockReadGuard`,
`RwLockWriteGuard`, `Ref` or `RefMut`, from any path. The check reads syntax
only, so another library's type named `Ref` is reported as well. A function
without a visibility modifier, including a trait method and its
implementations, is out of scope, as are test code and a function with a
`DECISION:` or `INVARIANT:` reference on the line directly above it or its
attributes. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when no visible function hands out a guard, or
handing it out is justified by an owned reference.
