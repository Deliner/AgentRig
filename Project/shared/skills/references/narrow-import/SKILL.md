---
name: narrow-import
description: Replace a glob import with an explicit import list, or record why the wildcard is deliberate.
---

# Name what you import instead of pulling in the whole module

A glob import — Rust `use path::*`, Python `from module import *` — brings in
every name the target module currently exposes. When that module later adds,
removes or renames a name, the set of names in scope changes silently: a new
upstream name can shadow or collide with a local one, and a reader (or an
editing agent) cannot tell where a name comes from. The file's dependency on
another module's surface is then unbounded and invisible.

1. List the names the file actually uses from that module and import them
   explicitly (`use path::{A, B}`, `from module import a, b`). The import line
   then states the exact surface the file depends on.
2. If the module is a deliberate prelude or re-export facade meant to be
   glob-imported, record the reasoning as an owned `DECISION:`/`INVARIANT:`
   reference on the line directly above the import.
3. If the glob was a shortcut and the names are few, the explicit list is both
   shorter to read and stable under upstream change.
4. Rerun the reported lint check and the affected behavior checks.

A `DECISION:` or `INVARIANT:` reference on the line directly above the import
keeps it, attesting the glob is deliberate; without one it is a finding. Test
code is out of scope.

## Rule binding

The measurement flags every Rust `use path::*` glob outside test code, and every
Python `from module import *`, without a `DECISION:` or `INVARIANT:` reference on
the line directly above it. Narrowing to an explicit import list, or recording an
owned reference, clears it. Rerun the reported lint check and relevant behavior
checks. Do not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the import names its surface explicitly or the
glob is justified by an owned reference.
