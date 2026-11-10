---
name: label-the-exit
description: Label the loop that an unlabeled break or continue inside a nested loop targets and name the label in the exit, extract the inner loop, or record why the exit needs no label.
---

# Name the loop an exit leaves

An unlabeled `break` or `continue` acts on the innermost enclosing loop. Inside
a loop that itself runs inside another loop of the same function, a reader, or
a later edit that wraps the body in one more loop, easily takes it for an exit
of the outer loop: the program then skips or repeats the wrong iterations with
no compile error.

1. Find the loop the exit is meant to leave or continue; read the surrounding
   code, not only the innermost loop.
2. Label that loop, such as `'rows: for row in rows`, and write the label in the
   exit: `break 'rows;` or `continue 'cells;`. Label the innermost loop too when
   it is the target, so every exit states its loop.
3. When the inner loop is a separate step, move it into a function or an
   iterator method such as `find`, `any` or `position`, so its exit becomes a
   `return` or a value and the outer loop keeps no nested exit.
4. When a label would add nothing, record the reasoning as an owned
   `DECISION:`/`INVARIANT:` reference on the line directly above the exit.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust `break` or `continue` without a label whose
nearest enclosing `loop`, `while` or `for` lies inside another such loop of the
same function. A closure, a nested function or an async block ends both
searches, so a loop around a closure does not count. Test code and an exit with
a `DECISION:` or `INVARIANT:` reference on the line directly above it are out
of scope. Rerun the reported lint check and relevant behavior checks. Do not
weaken the rule or its selectors merely to pass. Apply complexity-discipline;
stop when the exit names its loop, the inner loop is extracted, or the exit is
justified by an owned reference.
