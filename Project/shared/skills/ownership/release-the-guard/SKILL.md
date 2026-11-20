---
name: release-the-guard
description: Release a std Mutex or RwLock guard before the next .await by scoping or dropping it, or switch to an async-aware lock, or record why holding it across the suspension is safe.
---

# Release the lock before the task suspends

A `std::sync::Mutex` or `RwLock` guard keeps the lock while it is alive. When
the task awaits with the guard still alive, every other task that needs the
lock blocks a worker thread until this task resumes; on a single-threaded
executor the task that would release the lock never runs again, so the program
deadlocks. The guard also makes the future `!Send`, so it cannot move between
threads of a multi-threaded runtime.

1. Find the first `.await` after the binding the finding names, including one
   inside a macro body such as `allow_err!(conn.send(x).await)`.
2. If the guard is needed only before that point, end its life there: move the
   locked work into an inner block `{ let guard = …; … }`, or call
   `drop(guard)` before the `.await`. Copy or clone out the values the later
   code needs instead of keeping the guard.
3. If the critical section must span the `.await`, use a lock built for it,
   such as `tokio::sync::Mutex`, whose `lock().await` yields instead of
   blocking a thread.
4. If holding the guard is genuinely safe — the awaited future never touches
   the lock and the runtime cannot schedule a contending task meanwhile —
   record the reasoning as a decision with edit-decisions and put its exception
   `// DECISION: DNNN RULE`, RULE being the rule ID the finding names, alone on
   the line directly above the binding. List it in the decision as
   `[exception RULE](FILE)`; the user approves a new exception through
   propose-protected-change.
5. Rerun the reported lint check and the affected behavior checks.

The check reads syntax only: it recognizes the lock by a zero-argument
`.lock()`, `.read()` or `.write()` call and cannot see the lock's type, so it
reports a warning rather than an error.

## Rule binding

The measurement flags every Rust `let` of a plain name whose initializer,
without an `.await`, ends in a zero-argument `.lock()`, `.read()` or `.write()`
call, optionally followed by `?`, `.unwrap()`, `.expect(…)` or
`.unwrap_or_else(…)`, when a later statement of the same block contains an
`.await`, also inside a macro body, before an expression statement
`drop(name)`. An `.await` inside a closure, an async block or a nested
function does not count. Test code is out of scope. An exception naming this
rule, `guard-across-await` by default, on the line directly above the binding
lifts the finding; a bare `DECISION:` or `INVARIANT:` reference lifts nothing.
Rerun the reported lint check and relevant behavior checks. Do not weaken the
rule or its selectors merely to pass. Apply complexity-discipline; stop when
the guard ends before the `.await`, the lock is async-aware, or an approved
exception lifts the finding.
