# Give each directory a name of its own

`./structure` reads the Rust tree in `Project/` and prints a `total` line, the mean score and the number of units,
then one line per directory with production Rust code: 100 `rename` or 0 `ok`, its path and, for `rename`, the words
of its name with how many names outside its branch use each and up to three of them. Lower is better.

A reader who sees only the names of a tree tells two directories apart by the words only one of them uses. A
directory's branch is its ancestors and everything inside it. Its line is `ok` when its own name holds a word that no
directory or file name outside its branch uses; words are compared by their first six letters, and words shorter
than three letters or on a short stop list do not count.

## Loop

1. Run `./structure` and keep the output as the starting point.
2. Take up to ten untried `rename` lines in output order. A directory counts as tried once you have kept or undone an
   edit on it, whatever path it has now. Skip code the repository vendors from another project.
3. For each line, read the directory's description, the `purpose` line of its `architecture.yaml`, and its code. Give
   the directory itself a new name; its files and subdirectories keep theirs. Rerun `./structure`. Keep the edit when
   the directory's line is `ok` and the total did not rise. Otherwise undo it at once.
4. After the round, `cargo check --all-targets` must pass. Repair paths, never descriptions.
5. Repeat from step 2. Stop after a round that kept fewer than half of the edits it tried, or when no untried
   `rename` line remains.

## Names

You invent every name. Name a directory for its main job, the kind of part it is for the code that uses it, in one
to three plain words a reader would use to say it.

- The word only this directory uses must be the one that says its job. Never add a word only to pass the check.
- A technical word alone, such as the name of a format, a library, a data kind or a layer, does not say the job. Say
  what the directory does with it.
- Use plain words. A figure of speech or a pattern word, such as gateway, hub or bridge, hides the job.
- Never name anything for a word its code does not do.

## Constraints

- Only directories are renamed. No file is renamed, split, merged or moved on its own, and no new directory is made.
- Description text stays verbatim, and so does every `//!` documentation.
- You may change `mod` lines, `use` paths, `#[path]`, `include_str!` and other path references, `Cargo.toml` target
  paths and `architecture.yaml` keys.
- Rust module names are snake_case. The Cargo layout directories `src`, `tests`, `examples` and `benches` keep their
  names. No name is longer than the longest existing Rust directory name. No `utils`, `helpers`, `misc` or `common`.

## Map

Record every change in a map, one line each, with paths relative to the repository root:

```text
move OLD/ -> NEW/
```

The left side is always the original path, as before any edit. A directory rename is one line with trailing slashes,
`move a/b/ -> a/c/`, and carries everything inside. A directory inside a renamed one that is also renamed gets its own
line from its original path to its current path; the longest matching left side wins. Applying the map to the
original file list gives exactly the current list.
