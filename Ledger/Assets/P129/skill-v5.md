# Make a Rust tree show what each directory does

`agentrig lint-structure PATH` reads a Rust tree and prints a `total` line, the mean score of the name lines and their
number, then one name line per directory and file and a `group` line for each wide directory: a problem score from 0
to 100, an edit kind, the path and its evidence. Lower is better. The work runs in two loops, the names first and the
groups second.

## Name lines

A directory's description is the `purpose` line of its `architecture.yaml` together with the first paragraph of its
entry file's `//!` documentation. A reader who sees only the names of a tree understands a directory when the names of
its parts show its description. The parts are its subdirectories with Rust code and its Rust files other than the
entry files `mod.rs`, `lib.rs` and `main.rs`; test code does not count. The directory's own name does not count for
its own line; it counts for its parent's line. A directory line scores 0 when its part names show at least 40 % of its
description words and 100 when they show none. `parts:` gives the number of parts and `missing:` the description words
no part name shows, the most frequent first. With fewer than three parts the line asks for `split`, otherwise for
`rename`.

## Group lines

A reader who sees only the names of a tree takes in a directory through the names of its parts. Past about seven
names the reader stops telling them apart and lumps them together. A directory with more than seven parts gets a
`group` line that scores 100 × (parts − 7) / 7, at most 100, with `parts:`, `subdirectories:` and `files:`. Group
lines stand apart from the name lines and do not count in the total.

## Loop 1: names

1. Run the command and keep the output as the starting point.
2. Take up to ten untried directory name lines, the paths ending in `/` whose kind is not `group`, with the highest
   scores, ties in output order. A directory counts as tried once you have kept or undone an edit on it, whatever path
   it has now. Skip the root line, `ok` and `undescribed` lines and code the repository vendors from another project,
   such as a nested crate with its own license or upstream.
3. For each line, read the directory's description and code, make the edit its kind names below, then rerun the
   command. Keep the edit when the directory's score fell by at least 20 points and the total did not rise. Otherwise
   undo it at once.
4. After the round, `cargo check --all-targets` must pass. Repair paths, never descriptions.
5. Repeat from step 2. Stop this loop after a round that kept fewer than half of the edits it tried, or when no
   untried directory name line scores 50 or more. Then go to loop 2.

## Loop 2: groups

1. Run the command again; its `group` lines are the starting point of this loop.
2. Take the untried `group` lines, highest score first, ties in output order, the root line `.` included. A directory
   counts as tried in this loop once you have kept, undone or left an edit on it here, whatever path it has now. Skip
   code the repository vendors from another project.
3. For each line, read the directory's description, the descriptions of its parts and the code that connects them.
   Find the groups: two or more parts that serve one step of the directory's job together, because they call each
   other, pass one value along or serve the same caller, and that a reader would name as one thing. Move each group
   into a new subdirectory of the directory, named for what the group does. Parts that belong to no group stay where
   they are. When the parts are independent peers of one kind, each one rule, check, command or format that works
   alone, they form no group: leave the directory as it is.
4. Rerun the command. Keep the edit when the directory's `group` score fell and every new subdirectory holds at least
   two parts. Otherwise undo it at once. Name lines may change in this loop; they do not decide it.
5. After the round, `cargo check --all-targets` must pass. Repair paths, never descriptions.
6. Repeat from step 2 until no untried `group` line remains. Then stop: do not return to loop 1, and the name lines of
   new group directories need no edit.

## Names

You invent every name. A part's name says what its own code does, in one to three plain words, as a reader who sees
only the tree needs it. Prefer the words of the directory's description when they name that piece of its job; a
`missing:` word is a hint, never a name to paste. Never name a part for a word its code does not do.

Name a group for the step its parts share, in one to three plain words a reader would use to say it. A word for the
kind of thing the parts are, such as types, items, parts or modules, does not say the step. Use plain words; a figure
of speech or a pattern word, such as gateway, hub or bridge, hides the step.

## Edits by kind in loop 1

- `split` on a directory: it has fewer than three parts, so its names cannot show its job. Read its entry file and
  find the cohesive pieces of that job: a type with its impls, or the functions serving one step. Move each piece into
  its own file named for that piece, declare it with `mod`, and leave the wiring and the directory's main entry point
  in the entry file. When the entry file holds only one such piece, rename the existing parts or the subdirectories
  instead, or undo.
- `rename` on a directory: its parts do not show its job. Rename the parts whose names are vague or say something
  else, so each one names its piece of the job.
- A subdirectory's own name counts for its parent. When a parent's line lists `missing:` words that one of its
  subdirectories does, renaming that subdirectory to say so improves the parent.

File lines are secondary. Act on them only inside a directory you are already editing:

- `split` with `clusters:`: the file holds independent groups of code. Move each group into its own file named for
  what it does.
- `move` with `closer:`: the unit's content fits that directory better than its own. Move it there, or rename it when
  the move would mix concerns.
- `merge` with `close:`: a sibling says nearly the same. Combine the two, or rename them so the difference shows.
- `rename` with `words:`: the name is longer than three words. Shorten it.

## Constraints

- Every new file holds real code moved from its source: at least one function or type that other code uses. No empty
  files, no files that only re-export, no code written to fill a name.
- `lib.rs` and `main.rs` keep all their code and are never split.
- Loop 2 only groups: a part moves with its name unchanged into a new subdirectory of its own directory. In loop 2 no
  file or directory is renamed, split or merged, and no part moves to another directory.
- Description text stays verbatim, and so does each file's `//!` documentation in the file that keeps its entry
  point. A moved or renamed unit takes its `architecture.yaml` key with the same text to its new place. A new
  directory gets a `mod.rs` that declares its parts and an `architecture.yaml` whose `purpose` you write in one
  sentence from its parts' descriptions; a file made by a split may get a `files` entry whose text you draw from its
  source. Never edit a description to change a score.
- You may change `mod` lines, `use` paths, `#[path]`, `include_str!` and other path references, `Cargo.toml` target
  paths, `architecture.yaml` keys and the visibility of moved items, such as `pub(super)` to `pub(crate)`.
- Rust module names are snake_case. Extensions stay. `lib.rs`, `main.rs`, `mod.rs`, `build.rs`, `Cargo.toml`,
  `Cargo.lock`, `architecture.yaml`, `README.md`, `SKILL.md`, `.gitignore` and `clippy.toml` keep their names, and so
  do the Cargo layout directories `src`, `tests`, `examples` and `benches`. No name is longer than the longest existing
  one of its kind: Rust directory, Rust file stem or other name. No `utils`, `helpers`, `misc` or `common`.
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
its original path; the longest matching left side wins. A split lists every file that holds part of the original,
the kept entry file first under its new place: `split a/b/mod.rs -> a/c/mod.rs a/c/parse.rs a/c/render.rs`.

Loop 2 adds `move` lines: `move a/b/ -> a/g/b/` for a grouped directory and `move a/c.rs -> a/g/c.rs` for a grouped
file. When a grouped part, or anything inside it, already stands on the right side of a line, rewrite that right side
to the new place instead of adding a line for it, so every left side stays an original path. Applying the map to the
original file list gives exactly the current list, plus an `architecture.yaml` and a `mod.rs` for each new directory.
