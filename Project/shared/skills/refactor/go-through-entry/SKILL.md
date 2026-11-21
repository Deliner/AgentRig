---
name: go-through-entry
description: Let code use another directory only through that directory's entry, by moving work to its owner, giving a shared capability its own owner, adding one API function to the entry or reexporting what is the API, and move a test of a directory's inside into its tests/ directory, never by opening everything through the entry or widening the API only for a test.
---

# Use another directory through its entry

A directory's entry `mod.rs`, `lib.rs`, `main.rs` or `__init__.py` holds
everything the directory promises to other directories. When code of another
directory reaches past the entry into the files inside, the directory cannot
rebuild its inside without editing its consumers and their tests.

Repair a use in working code in this order:

1. When the consumer does work that belongs to the owner, move the work to
   the owner and call the result through its entry.
2. When several directories need the capability and it does not belong to
   this directory, move it to its own owner and use that owner's entry.
3. Otherwise give the entry one function for the consumer's task.
4. Reexport an item in the entry only when the item itself is the
   directory's API.

Repair a use in a test in this order:

1. When the test checks behavior the directory promises to other
   directories, rewrite it to call the entry.
2. When the test checks the directory's inside, move it into the `tests/`
   directory of that directory, where it uses the directory's files.

In Rust, declare each part the repair leaves internal with a private `mod`,
so that the compiler holds the boundary. Never open everything through the
entry, and never widen the API only for a test. Update each architecture.yaml
inventory you change, build, run the affected tests and rerun the reported
lint check.

## Groups

A directory whose architecture.yaml declares `group: true` is a group of
independent parts. It has no boundary of its own: its files are open to
every directory, and each of its subdirectories keeps its own boundary and
entry. Declare a group only when its parts change independently of each
other, never to silence a finding.

## Rule binding

The measurement reads the dependencies the project's directory-architecture
rule resolves, test code included, and reports each use of a Rust or Python
source inside a directory that does not contain the using file, unless the
source is the entry of the topmost such directory, with `file:line`, the
target and the entry to go through. A directory without an entry is closed:
every outside use is a finding. Module declarations, embedded resources and
the packages a Python import initializes on its way are not uses. The rule
accepts no exception. Do not exclude the directory, declare a group or weaken
the rule merely to pass. Apply complexity-discipline; stop when every outside
use goes through the entry and the affected tests pass.
