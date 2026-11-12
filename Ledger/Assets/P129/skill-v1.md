# Tune a Rust tree by its structure scores

`agentrig lint-structure PATH` reads a Rust tree and prints a `total` line, the mean score and the number of units,
then one line per directory and file: a problem score from 0 to 100, an edit kind and, when shown, its evidence.
Lower is better. A unit's description is its `purpose` and its entry under `files` or `directories` in
`architecture.yaml`, or else the first paragraph of its `//!` documentation. The score grows when the names of the
unit and its children do not show what the description says, when a unit is crowded or mixes separate groups of code,
and when its content fits another place better.

## Loop

1. Run the command and keep the output as the starting point.
2. Take up to ten untried lines with the highest scores, ties in output order. Skip `ok` and `undescribed` lines and
   code the repository vendors from another project, such as a nested crate with its own license or upstream.
3. For each line, read the unit's description and code, make the edit its kind names below, then rerun the command.
   Keep the edit when the unit's score fell by at least 20 points and the total did not rise; for a split or a merge,
   compare the old line with the highest new one. Otherwise undo it at once.
4. After the round, `cargo check --all-targets` must pass. Repair paths, never descriptions.
5. Repeat from step 2. Stop after a round that kept fewer than half of the edits it tried, or when no untried line
   scores 50 or more.

You invent every name. Choose one to three plain words for the unit's main point, as a reader who sees only the tree
needs it: a domain word from the description rather than a metaphor or a vague role.

## Edits by kind

- `rename`: the names do not show the description. `missing:` lists its weightiest absent words; `words:` counts a
  name longer than three words.
- `split` with `clusters:`: the file, or a directory's entry file, holds independent groups of code. Move each group
  into its own file named for what it does.
- `split` with `entries:`: the directory has more than ten entries. Group related entries into subdirectories named
  for what each group does.
- `move` with `closer:`: the unit's content fits that directory better than its own. Move it there, or rename it when
  the move would mix concerns.
- `merge` with `close:`: a sibling says nearly the same. Combine the two, or rename them so the difference shows.

When a line shows no evidence, read the descriptions and code to find what its kind refers to.

## Constraints

- Description text stays verbatim. A moved or renamed unit takes its `architecture.yaml` key with the same text to its
  new place. A new directory gets a `mod.rs` and an `architecture.yaml` whose `purpose` you write; a unit made by a
  split or merge may get new text drawn from its sources. Never edit a description to change a score.
- You may change `mod` lines, `use` paths, `#[path]`, `include_str!` and other path references, `Cargo.toml` target
  paths and `architecture.yaml` keys.
- Rust module names are snake_case. Extensions stay. `lib.rs`, `main.rs`, `mod.rs`, `build.rs`, `Cargo.toml`,
  `Cargo.lock`, `architecture.yaml`, `README.md`, `SKILL.md`, `.gitignore` and `clippy.toml` keep their names. No name
  is longer than the longest existing one of its kind: Rust directory, Rust file stem or other name. No `utils`,
  `helpers`, `misc` or `common`.
- A directory has at most fifteen entries besides `architecture.yaml`. A changed directory never keeps only its
  contract and `mod.rs`.

## Map

Record every change in a map, one line each, with paths relative to the repository root:

```text
move OLD -> NEW
split OLD -> NEW1 NEW2 ...
merge OLD1 OLD2 ... -> NEW
```

The left side is always the original path, as before any edit. A directory move or rename is one line with trailing
slashes, `move a/b/ -> a/c/`, and carries everything inside. A file inside it that also changes gets its own line from
its original path; the longest matching left side wins. Applying the map to the original file list gives exactly the
current list, plus an `architecture.yaml` and a `mod.rs` for each new directory.
