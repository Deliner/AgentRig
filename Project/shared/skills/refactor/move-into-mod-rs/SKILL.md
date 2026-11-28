---
name: move-into-mod-rs
description: Move a module file that sits beside the directory of its child modules, such as parser.rs next to parser/, into that directory as mod.rs, so that every module keeps its entry inside its own directory.
---

# Keep the entry of a module inside its directory

A module with child modules can live in `parser.rs` next to a `parser/`
directory or in `parser/mod.rs`. Mixing both layouts splits such modules over
two places, and the rig reads the `mod.rs` of a directory as its entry, the
table of contents that declares its parts.

1. Move the module file into its directory as `mod.rs`: `src/parser.rs`
   becomes `src/parser/mod.rs` with its content unchanged. The `mod parser;`
   declaration in the parent stays as it is; update a `#[path]` attribute that
   names the old file.
2. Update the `architecture.yaml` contracts: the parent no longer lists
   `parser.rs`, and the contract of `parser/` registers `mod.rs`.
3. Rerun the reported lint check and the affected behavior checks.

No exception keeps the file: the move keeps the path and the contents of the
module, so do not suppress the lint.

## Rule binding

The rig's Clippy profile denies `clippy::self_named_module_files` in the
library, binaries and examples of the configured Cargo manifest; tests are
outside the profile. Clippy reports a module file with a directory of the same
name beside it, and the lint check reports each message as an error naming
this skill. Do not weaken the profile merely to pass. Apply
complexity-discipline; stop when every module with child modules has its
`mod.rs` inside its directory.
