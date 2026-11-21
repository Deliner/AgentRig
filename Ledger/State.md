# State

## Focus

P171 B176 entry-use and P172 B178 entry-bypass, delivered autonomously under consent. VAC 19 moves the Rust
extraction entry helpers into path syntax and recording files so the entry keeps dispatch and item recording.

## Workspace

Branch: feature/entry-boundaries
Revision: 7cbf6c8c83ddd587bfacabe9d3c7dd89d653ac4a

## Progress

- VAC 19: extraction/rust gains path_syntax.rs for path text, use trees, literals and scope, and recording.rs for
  normalized reference recording and module-level imports; prelude.rs takes the generic and local binding lookup;
  attributes.rs takes attribute macros; names_import leaves macros.rs so prelude no longer depends on macros.
- D136 extraction link moves to attributes.rs and D146 inline context to recording.rs; architecture.yaml updated.

## Verification

- clippy, rustfmt, extraction tests 13 and architecture tests 107 pass.
- The first gate failed on function-lines 44 for use_leaves; the use-as clause moved into aliased_leaf.
- Candidate measurement with entry-use, file-cycle and architecture: entry-use 45 to 21, file-cycle 0, architecture 0.

## Blockers

- The checks.yaml impact-map proposal in .cache/runtime/proposal waits for the user approve.

## Next action

Commit VAC 19, then move the languages, neutral metrics, exemptions and std forms entry items into owner files.
