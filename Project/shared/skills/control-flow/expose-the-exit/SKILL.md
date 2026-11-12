---
name: expose-the-exit
description: Turn a macro_rules! body that returns, propagates with ? or leaves the caller's loop into code whose exit the caller writes, name the macro after its exit, or record why the hidden exit is intended.
---

# Show the exit at the call site

A `macro_rules!` body is pasted into its caller. A `return`, a `?`, or a
`break` or `continue` outside a loop of the macro body leaves the caller's
function or loop, while the invocation `check!(x)` reads like an expression
that yields a value. A reader or an agent editing the caller does not see the
exit: code after the call is skipped, a flag or a counter set before it stays
set, and moving the call into a closure changes which function it leaves.

1. Find what the exit carries: an error to propagate, an absent value, a skip
   of the current iteration or an early result.
2. When the macro only computes values, turn it into a function or a closure
   that returns `Result` or `Option`, and write `?`, `let … else` or the
   `return` at each call site.
3. When it must stay a macro, such as for a format string, a repetition or a
   type pattern, let it expand to an expression of type `Result` or `Option`
   and propagate at each call site.
4. When the exit is the whole meaning of the macro, name it after the exit: a
   part of its name split at underscores is `bail`, `ensure`, `require`,
   `return`, `try`, `break`, `continue`, `exit`, `throw`, `fail` or
   `propagate`, as in `bail_if_none!`.
5. Otherwise record why the hidden exit is intended as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the `return`,
   `?`, `break` or `continue`.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every `return`, `?`, and `break` or `continue` outside a
loop of the same body in a Rust `macro_rules!` transcriber that parses as a
function body once each `$` and the delimiters, separator and operator of each
`$(..)` repetition become spaces. Closures, nested `fn` items and `async`
blocks of that body do not count. A macro whose name has one of the exit words
of step 4 as an underscore-separated part is skipped. Test code and an exit
with a `DECISION:` or `INVARIANT:` reference on the line directly above it are
not measured. A `continue` inside a `$body:block` fragment is written by the
caller and is not seen. Rerun the reported lint check and relevant behavior
checks. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the exit is written at the call site, the
macro is named after its exit, or the exit is justified by an owned reference.
