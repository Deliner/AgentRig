---
name: own-the-state
description: Give process-global mutable state an owner instead of a static lock or a public static cell, declared directly or through lazy_static!/thread_local!, or record why the global is intended.
---

# Own the state instead of sharing it through a global

A `static` holding a `Mutex`, `RwLock`, `DashMap` or `DashSet` (directly or
inside `LazyLock`, `OnceLock`, `Lazy` or `Arc`), a `pub static` holding any
cell (`RefCell`, `Cell`, `UnsafeCell`, `Atomic*` or a lock), and the same items
declared through `lazy_static!` or `thread_local!`, are mutable state that any
code in the process can change. The directory tree shows no owner, so every
writer has to be found by search and a behavior change needs all of them.

1. Prefer passing the state: construct it once at the entry point and hand it
   to the functions that use it as a parameter or a field of their context.
2. If the state must be reachable without a parameter, keep the cell private
   behind one owning function or type in the module that defines the behavior,
   so that module is the only writer and readers go through its public surface.
3. A private `Atomic*`, `Cell` or `RefCell` is the idiomatic once-flag or
   counter and is not a finding; a `OnceLock` or `LazyLock` holding an
   immutable value has no writers and is not a finding either.
4. If the global is genuinely intended — a process-wide registry with no
   possible owner, or an interned table — record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the item, or
   above the macro for an item on the macro's first line.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust `static` item outside test code, excluding
`static mut`, whose type — following `type` aliases declared at the top level of
the same file up to three levels — contains a lock, or a `pub` static whose type
contains any cell, including items declared inside `lazy_static!` or
`thread_local!`, without a `DECISION:` or `INVARIANT:` reference on the line
directly above it. Test code is out of scope. Rerun the reported lint check and
relevant behavior checks. Do not weaken the rule or its selectors merely to pass.
Apply complexity-discipline; stop when the state has an owner, or the global is
justified by an owned reference.
