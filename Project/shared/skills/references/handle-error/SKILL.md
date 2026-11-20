---
name: handle-error
description: Handle a swallowed error, or record why it is safe to ignore with an exception.
---

# Handle the error, or record why it is safe to ignore

A Python except clause whose body is only `pass` swallows the error silently:
the failure it catches leaves no trace and the program continues as if nothing
went wrong. A Rust `read_dir` chain collapsed by `.flatten()`,
`.filter_map(Result::ok)`, `.filter_map(|e| e.ok())` or `.map_while(Result::ok)`
drops every entry that failed to read, so an unreadable directory silently
reads as empty. Any `.filter_map` or `.flat_map` that keeps only the `Ok`
values, such as `.filter_map(|text| text.parse().ok())`, drops every failed
item the same way: the result is just shorter. A `.ok().flatten()` turns a
`Result<Option<T>, E>` into `Option<T>`, so a failed parse reads as an absent
value. A thread's `.join()` followed by `.unwrap_or_default()`, `.unwrap_or(..)`,
`.unwrap_or_else(..)` or `.ok()` turns the thread's panic into a fallback value,
such as an empty part of the result. A Rust `Err(_)` arm that answers `Ok(false)` or `Ok(true)` in a
function returning `Result<bool, …>` turns a failed check into a verdict. A
returned `Ok(status.success())` in such a function, with the status from
`.status()?` or `.output()?.status` and no `.code()` read, turns a failed
command into the answer no: `git merge-base --is-ancestor` exits 1 for no and
128 for an error, and `.success()` is false for both.
Decide what the handler owes its caller instead of discarding the error.

1. Read the guarded operation and establish which error it catches and what
   state it leaves behind: a half-written file, an open resource, a retry the
   caller still expects, a listing the caller will go on to read, or a yes/no
   answer the caller will act on.
2. Prefer handling the error. Recover, retry, clean up, log it, or re-raise a
   clearer error so the failure stays visible. Narrow a broad `except` to the
   one exception that is genuinely expected. In Rust, iterate with
   `for entry in read_dir(..)? { let entry = entry?; … }` or collect the
   entries into `Result<Vec<_>, _>`. When a failed item is a normal event of
   the traversal, write it as a `for` loop with a `match` whose `Err(error)`
   arm states why skipping is safe, and propagate the error from a check with
   `?` or `Err(error) => Err(error.into())` instead of answering a verdict.
   Propagate the error of a `Result<Option<T>, E>` with `?` before taking the
   option. Let a joined thread's panic continue with
   `match handle.join() { Ok(value) => value, Err(panic) => resume_unwind(panic) }`
   or turn it into an error with `.map_err(..)?`.
   For a command whose exit code is the answer, compare `status.code()` with
   the codes the command documents as answers, such as `Some(0 | 1)`, refuse
   every other code with an error, then answer; or call an owner that already
   does, such as the repository's VCS owner.
3. When the error is genuinely safe to ignore, record why as a decision with
   edit-decisions and put its exception, `# DECISION: DNNN RULE` in Python or
   `// DECISION: DNNN RULE` in Rust, RULE being the rule ID the finding names,
   alone on the line directly above the `except` line in Python, or above the
   chain, the arm or the return in Rust. List it in the decision as
   `[exception RULE](FILE)`; the user approves a new exception through
   propose-protected-change.
4. Keep one reason per handler: distinct swallowing handlers that rely on the
   same recorded reason each carry that exception on their own line above.
5. Rerun the reported lint check and the affected behavior checks.

Only an exact exception `DECISION: DNNN RULE` naming this rule, `silent-error`
by default, justifies the swallow; a bare `DECISION:` or `INVARIANT:`
reference lifts nothing. Do not silence the finding with a bare comment, a
broadened `except`, or by widening the rule.

## Rule binding

The measurement flags every Python except clause whose body is only `pass`,
every Rust `read_dir` call chain collapsed by `.flatten()` or
`.map_while(ok)`, every Rust `.filter_map(ok)` or `.flat_map(ok)` on any chain,
where `ok` is `Result::ok` or a one-parameter closure whose body is only a
`.ok()` call, except `Result::ok` and `|x| x.ok()` on a `.lines()` call, which
Clippy's default `lines_filter_map_ok` reports, every Rust `.ok().flatten()`,
every Rust `.join()` without arguments directly followed by `.ok()`,
`.unwrap_or_default()`, `.unwrap_or(..)` or `.unwrap_or_else(..)`, every Rust `match`/`if let` arm `Err(_)`/`Err(..)` whose body is `Ok(true)`,
`Ok(false)` or a `return` of one inside a function declared to return
`Result<bool, …>`, and every `Ok(<status>.success())` such a function returns
as its tail or with `return` when the status comes from `.status()?`,
`.output()?.status` or a `let` binding of them and the body reads no `.code()`,
outside Rust test code. An exception naming this rule on the line directly
above the reported line lifts the finding. Rerun the reported lint check and
relevant behavior checks. Do not weaken the rule or its selectors merely to
pass. Apply complexity-discipline; stop when the finding is resolved and the
error is either handled or lifted by an approved exception.
