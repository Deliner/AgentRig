---
name: move-out-of-entry
description: Let a directory's own code use the file that defines an item instead of the directory entry, moving a type, function or constant the entry defines into a file named by its meaning, so the entry stays the directory's table of contents, never by moving the item into a types, common or utils dump.
---

# Keep the entry a table of contents

The entry `mod.rs`, `lib.rs`, `main.rs` or `__init__.py` tells a reader what
a directory holds and what it gives to other directories: its parts, its
wiring and its API. When files of the directory, or of a directory below it,
take items from the entry, the entry becomes one more file of the
implementation, and a change to it can break the code it describes.

1. Read the finding: `file:line` uses the entry it names. Find the item the
   source takes from it: the type, function, constant or trait. A reexport
   in the entry counts as a use of the entry.
2. When the entry only reexports the item, change the use to the file that
   defines it, such as `super::job::Job` instead of `super::Job`, or
   `from jobs.job import Job` instead of `from jobs import Job`.
3. When the entry defines the item, move it into a file of the directory
   named by its meaning, an existing file that owns the responsibility or a
   new one named for it, and let the directory's own code use that file.
   Keep a reexport in the entry only when the item is the directory's API
   for other directories. Choose the file so that the directory's files do
   not come to use each other in a cycle, which file-cycle reports.
4. When a subdirectory that the entry uses needs the item too, place the
   item in a separate sibling part that both use. Leaving it in the entry
   would make the entry and the subdirectory a directory cycle.
5. Update the architecture.yaml inventory for each new or moved file, build,
   run the affected tests and rerun the reported lint check.

Never move the item into a file named `types`, `common`, `utils`, `helpers`
or another name that says where items lie rather than what they do. The
entry may keep module declarations, wiring and functions that form the
directory's API for other directories; it need not be empty, but the
directory's own code does not call it.

## Rule binding

The measurement reads the dependencies the project's directory-architecture
rule resolves and reports every use, outside test code, of the entry of the
using file's directory or of a directory above it, with `file:line` and the
entry. Module declarations such as `mod child;` and the packages a Python
import initializes on its way, such as `jobs/__init__.py` for
`from jobs.job import Job`, are not uses. Test code may use the entry. The
rule accepts no exception. Do not exclude the directory or weaken the rule
merely to pass. Apply complexity-discipline; stop when the directory's own
code no longer uses its entry and the affected tests pass.
