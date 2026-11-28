---
name: handle-discarded
description: Handle or propagate a Result, a combined value or a spawn handle instead of dropping it with a bare .ok(), .err(), Option-combinator, .status()?, .output()? or spawn statement or a let _ binding of a must_use value, or record why the outcome is ignored.
---

# Handle the Result instead of discarding it

A bare `expr.ok();` or `expr.err();` statement converts a `Result` to an
`Option` and immediately drops it, so a failure passes with no handling and no
trace — the same silent swallow that `silent-error` catches for a Python
`except: pass`. A bare statement ending in `.map(…)`, `.and_then(…)`,
`.map_or(…)` or `.map_or_else(…)` drops the value its closure produced, often a
`Result` the compiler cannot see through an `Option`; on a lazy iterator the
closure never runs at all. A bare `cmd.status()?;` or `cmd.output()?;` statement
propagates only the error of launching the command and drops its `ExitStatus`
or `Output`, so a command that ran and failed looks like a success. A bare
`thread::spawn(…);`, `tokio::spawn(…);` or `cmd.spawn()?;` statement, or a
`let _ =` binding of one, drops the handle: the thread or task runs detached,
its panic or error is never observed, and a child process is never waited for.

1. Decide what the failure means here. Propagate it with `?`, return a `Result`,
   or match the variants and handle both.
2. Keep the outcome when it matters: `let kept = expr.ok();` or an assignment
   preserves the `Option` for a later decision; `if let Some(value) = option`
   with the fallible call inside handles the combined value, and `for` or
   `.for_each` runs a side effect on every item; `let status = cmd.status()?;`
   or `let output = cmd.output()?;` followed by a check of the exit code and,
   for output, its streams turns a failed command into an error. Use
   `status.success()` alone only when every nonzero code means failure; when
   a nonzero code is an answer, such as git's 1 for no, compare `code()` with
   the documented answer codes and refuse the others, or call an owner that
   does. Keep a spawn handle in a named binding or with its owner, then `join`,
   `await` or `wait` it and handle the result it returns.
3. For a deliberate best-effort call whose failure is genuinely ignorable (a
   cleanup on an already-failing path, a command whose exit status carries no
   meaning here, a background task deliberately detached for the process
   lifetime), record the reasoning as a decision with edit-decisions and put
   its exception `// DECISION: DNNN RULE`, RULE being the rule ID the finding
   names, alone on the line directly above the statement. List it in the
   decision as `[exception RULE](FILE)`; the user approves a new exception
   through propose-protected-change.
4. Rerun the reported lint check and the affected behavior checks.

An exception naming this rule, `discarded-result` by default, on the line
directly above the statement keeps it; a bare `DECISION:` or `INVARIANT:`
reference lifts nothing. Test code is out of scope.

## Clippy profile

The rig's Clippy profile denies `clippy::let_underscore_must_use` in the
library, binaries and examples of the configured Cargo manifest; tests are
outside the profile. Clippy reports `let _ = expr;` when the type of the value
or the called function is `#[must_use]`, such as a `Result` or `is_ok()`, and
the lint check reports each message as an error naming this skill. Handle or
keep the value as in step 2. Renaming `_` to a name with a leading underscore
only to pass discards the outcome all the same and keeps the value alive to
the end of the scope; do not do it. When the outcome is genuinely ignorable,
as in step 3, record the reasoning as a decision with edit-decisions and put
`#[expect(clippy::let_underscore_must_use)]` on the statement, with its
exception `// DECISION: DNNN allow-justification` alone on the line directly
above the attribute. List it in the decision as
`[exception allow-justification](FILE)`; the user approves a new exception
through propose-protected-change, as justify-allow describes. Do not weaken
the profile merely to pass.

## Rule binding

The measurement flags every Rust statement that is a bare call whose last method
is `.ok()`, `.err()`, `.map(…)`, `.and_then(…)`, `.map_or(…)` or
`.map_or_else(…)`, every bare statement `….status()?` or `….output()?` whose
call has no arguments, and every bare statement or `let _ =` binding whose
value, through `?`, `.unwrap()` and `.expect(…)`, is a call whose path or method
ends in `spawn`, outside test code. A named `let`-binding such as `let handle =` or
`let _guard =`, an assignment, a returned tail expression, `expr.ok()?`, a
`….status()?.success()` or `spawn(…).join()` chain and an argument position
keep or propagate the outcome and are not flagged here; a `.success()` returned
from a `Result<bool>` function that never reads `.code()` is a `silent-error`
finding instead. The receiver type is not resolved, so a closure without effects
is flagged too, and any method named `spawn` counts as a spawn; binding or
removing the statement resolves it. Rerun the reported lint check and relevant
behavior checks. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the Result is handled or an approved exception
lifts the finding.
