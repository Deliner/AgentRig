---
name: break-file-cycle
description: Break a cycle of uses among the files of one directory by moving a shared type or function into the file that owns it, so that both files depend on it, or by joining files that carry one responsibility, never by merging files or hiding a link behind a re-export only to pass.
---

# Let each file depend only on what it needs

Files of one directory that use each other in a cycle can only be read,
changed and tested together: a change in one of them can break every other
file of the cycle, and none of them can be understood alone. Each file should
read as a whole from the files it depends on, with the dependencies running
one way.

1. Read every link of the finding: each `file:line uses file` names a use in
   the first file of something the second file defines. List what each link
   uses: the type, function, constant or trait.
2. Find the extra link. It is usually the use that goes against the direction
   of the others: a basic file reaching back into the file that builds on it,
   or a file using a helper that belongs to neither file's responsibility.
3. When the used item is shared by files of the cycle, move it into the file
   that owns its responsibility, an existing one or a new file named for it,
   and let each file of the cycle use it from there. The moved item then
   depends on none of the files it served.
4. When two files carry one responsibility and were split by size or by
   position, join them into one file named for that responsibility. Join
   only when one sentence describes the joined file; otherwise apply step 3.
5. Update the architecture.yaml inventory of the directory for each moved,
   new or joined file, build, run the affected tests and rerun the reported
   lint check.

Never merge files only to remove the cycle, never route a use through a
re-export in the directory entry `mod.rs`, `lib.rs`, `main.rs` or
`__init__.py` to hide a link, and never move an item into the entry only
because the entry takes no part in the check. The entry holds the
directory's own surface and its wiring.

## Rule binding

The measurement reads the dependencies the project's directory-architecture
rule resolves, test code included, and reports each cycle among the files of
one selected directory once, at its closing use, with the whole chain and
`file:line uses file` for every link. Module declarations such as `mod
child;` are not uses, and the entry `mod.rs`, `lib.rs`, `main.rs` or
`__init__.py` takes no part in a cycle. The rule accepts no exception. Do not
exclude the directory or weaken the rule merely to pass. Apply
complexity-discipline; stop when the directory has no file cycle and the
affected tests pass.
