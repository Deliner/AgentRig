---
name: act-and-match-the-error
description: Replace an exists, is_file or is_dir check before a file action with the action itself and a match on its error kind, or record why the path cannot change in between.
---

# Act on the path and read the answer from its error

Between `path.exists()` and `fs::remove_file(path)?` another process, a
thread or the user can create, delete or replace the path. `exists()` also
follows symbolic links and answers `false` for a dangling link or a path it
cannot inspect, so the node may not be of the kind the action expects. The
check then picks the wrong branch: the action fails with an error the code
meant to treat as normal, or it is skipped and the failure surfaces later. The
action reports the same facts in one step through its `io::Error`.

1. Name what absence means at this place: nothing to do, an empty result, a
   default or creating the path.
2. Drop the check and perform the action. Give the error kind the check stood
   for the absence behavior and let every other error propagate:

   ```rust
   match fs::remove_file(path) {
       Err(error) if error.kind() == io::ErrorKind::NotFound => {}
       result => result?,
   }
   ```

   For a read, return the empty result on `NotFound`. `fs::create_dir_all`
   already succeeds on an existing directory, and
   `OpenOptions::new().write(true).create_new(true)` fails with
   `AlreadyExists` instead of overwriting.
3. When the check chooses between two actions, perform the likely one and
   fall back on its error kind, so each action meets the path as it is.
4. When the path cannot change between the check and the action, such as a
   directory this process created and alone uses, record that guarantee as an
   owned `DECISION:`/`INVARIANT:` reference on the line directly above the
   check.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every Rust `exists()`, `is_file()` or `is_dir()` call
without arguments that an `if` tests, directly or through a `let` of a plain
name tested by the first later `if` of the same function. The branch where the
path is absent, chosen by the negations, must be missing or hold no `?`,
`Err(..)` call, `bail`, `ensure`, `panic`, `unreachable`, `todo` or
`unimplemented` macro and no file action. A later call in the same function,
in the present branch or after the `if` when the absent branch returns,
continues or breaks, must have an `fs` path segment before its name or end in
`File::open` or `File::create`, take an argument whose text equals the checked
receiver without `&` and `.as_path()`, and pass its error through `?`, also
after `.await`, `.context(..)`, `.with_context(..)` or `.map_err(..)`. Test
code and a check with a `DECISION:` or `INVARIANT:` reference on the line
directly above it are not measured. Paths are compared as text, so the same
path rebuilt or held by another variable is not seen. Rerun the reported lint
check and relevant behavior checks. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when the action's own error
decides the outcome or the stable path is justified by an owned reference.
