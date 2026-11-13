# Group the parts of wide directories

`./structure` reads the Rust tree in `Project/` and prints a `total` line, the mean score and the number of units,
then one line per directory with production Rust code: its score, `group` or `ok`, its path and its parts. Lower is
better.

A reader who sees only the names of a tree takes in a directory through the names of its parts. Past about seven
names the reader stops telling them apart and lumps them together. A directory's parts are its Rust files other than
`mod.rs`, `lib.rs` and `main.rs` and its subdirectories with Rust code, besides `tests`, `examples` and `skills`. A
directory with more than seven parts scores 100 × (parts − 7) / 7, at most 100, and asks for `group`.

## Loop

1. Run `./structure` and keep the output as the starting point.
2. Take the untried `group` lines, highest score first, ties in output order. A directory counts as tried once you
   have kept, undone or left an edit on it, whatever path it has now. Skip code the repository vendors from another
   project.
3. For each line, read the directory's description, the `purpose` line of its `architecture.yaml`, the descriptions
   of its parts and the code that connects them. Find the groups: two or more parts that serve one step of the
   directory's job together, because they call each other, pass one value along or serve the same caller, and that a
   reader would name as one thing. Move each group into a new subdirectory of the directory, named for what the group
   does. Parts that belong to no group stay where they are. When the parts are independent peers of one kind, each
   one rule, check, command or format that works alone, they form no group: leave the directory as it is.
4. Rerun `./structure`. Keep the edit when the directory's score fell and every new subdirectory holds at least two
   parts. Otherwise undo it at once.
5. After the round, `cargo check --all-targets` must pass. Repair paths, never descriptions.
6. Repeat from step 2 until no untried `group` line remains.

## Names

You invent every group name. Name a group for the step its parts share, in one to three plain words a reader would use
to say it. A word for the kind of thing the parts are, such as types, items, parts or modules, does not say the step.
Use plain words; a figure of speech or a pattern word, such as gateway, hub or bridge, hides the step.

## Constraints

- Only grouping: a part moves with its name unchanged into a new subdirectory of its own directory. No file or
  directory is renamed, split or merged, and no part moves to another directory.
- Description text stays verbatim, and so does every `//!` documentation. A moved part takes its `architecture.yaml`
  key with the same text to its new place. A new directory gets a `mod.rs` that declares its parts and an
  `architecture.yaml` whose `purpose` you write in one sentence from its parts' descriptions.
- You may change `mod` lines, `use` paths, `#[path]`, `include_str!` and other path references, `Cargo.toml` target
  paths, `architecture.yaml` keys and the visibility of moved items, such as `pub(super)` to `pub(crate)`.
- Rust module names are snake_case. No name is longer than the longest existing Rust directory name. No `utils`,
  `helpers`, `misc` or `common`.

## Map

Record every change in a map, one line each, with paths relative to the repository root:

```text
move OLD -> NEW
```

The left side is always the original path, as before any edit. A moved directory is one line with trailing slashes,
`move a/b/ -> a/g/b/`, and carries everything inside; a moved file is `move a/c.rs -> a/g/c.rs`. Applying the map to
the original file list gives exactly the current list, plus an `architecture.yaml` and a `mod.rs` for each new
directory.
