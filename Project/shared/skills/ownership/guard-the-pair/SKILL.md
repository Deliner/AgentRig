---
name: guard-the-pair
description: Close a configured call pair such as begin and commit on every path, through a guard whose Drop closes or a scope that always closes, when an early exit lies between the two calls, or record why the exit may leave it open.
---

# Close the pair on every path

`tx.begin(); step()?; tx.commit();` closes only on the path that reaches the
last line. A `?`, `return`, `break` or `continue` between the two calls leaves
the transaction open, the lock held or the timer running, and nothing in the
code shows it.

1. Read the block and the pair, and establish what must happen when the early
   exit is taken: close, roll back or leave the state for the caller.
2. Make the opening call return a guard whose `Drop` closes or rolls back, and
   keep the explicit closing call as a method of the guard that consumes it.
3. When the pair belongs to one operation, move the steps into a function that
   opens, runs a closure and closes on every outcome, such as
   `with_transaction(|tx| ...)`.
4. When the steps between can be ordered differently, move the early exits
   before the opening call or after the closing call.
5. When leaving the pair open on the exit is the intended contract, such as a
   receiver whose own `Drop` already closes, record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the opening
   statement.
6. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The rule's `pairs` setting lists the open and close method names of the
project. The measurement reports every Rust block statement calling the open
method of a configured pair on a receiver, followed in the same block by the
close method on the token-equal receiver with a `?`, `return`, `break` or
`continue` between them outside closures. Test code and a statement with a
`DECISION:` or `INVARIANT:` reference on the line directly above it are out of
scope. Do not weaken the rule, its pairs or its selectors merely to pass. Apply
complexity-discipline; stop when the pair closes on every path or the exit is
justified by an owned reference.
