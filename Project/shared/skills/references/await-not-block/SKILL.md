---
name: await-not-block
description: Replace a blocking thread::sleep or nested block_on inside async Rust with an awaited sleep or future, move blocking work to spawn_blocking, or record why blocking that executor thread is intended.
---

# Await instead of blocking the executor thread

An async task runs on an executor thread shared with other tasks.
`std::thread::sleep` parks that whole thread, so every task scheduled on it
stops; on a single-threaded runtime the program freezes for the full sleep. A
nested `block_on` runs a second executor on the same thread: tokio panics when
it detects the nesting, and other executors block the thread until the inner
future finishes, which can deadlock when that future waits for the outer one.

1. Read what the call waits for. For a delay, await the runtime's timer, such
   as `tokio::time::sleep(duration).await` or the equivalent of the runtime in
   use, instead of `thread::sleep`.
2. For a nested `block_on(future)`, await the future directly:
   `future.await`. Return the result up the async call chain instead of
   bridging back to synchronous code.
3. If the work is genuinely blocking — a synchronous library, a long CPU loop,
   a deliberate wait — move it off the executor with the runtime's
   `spawn_blocking` or a dedicated thread and await its handle.
4. Keep the blocking call only when blocking that thread is the intent, such
   as a debug action that hangs an executor on purpose or test-support code on
   its own runtime. Record the reasoning as an owned `DECISION:`/`INVARIANT:`
   reference on the line directly above the call.
5. Rerun the reported lint check and the affected behavior checks.

The check reads syntax only: a type's own `block_on` method that does not
block is reported too, so the finding is a warning rather than an error.

## Rule binding

The measurement flags every Rust call whose path ends in `thread::sleep`, a
bare `sleep` imported in the same file from a `thread` module, or a function,
turbofish function or method named `block_on`, when its nearest enclosing
scope boundary is an `async fn` or an `async` block. A closure, async closures
included, or a plain `fn` between the call and that boundary ends the async
context, so `spawn_blocking(|| thread::sleep(..))` is not flagged. Test code
and a call with a `DECISION:` or `INVARIANT:` reference on the line directly
above are out of scope. Rerun the reported lint check and relevant behavior
checks. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the async code awaits instead of blocking,
the blocking work runs off the executor, or the block is justified by an owned
reference.
