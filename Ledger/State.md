# State

## Focus

P200 (B109 AC1–AC2), code VAC: memory-check binds a cargo oracle to the exact module path of its linked file. The
target must equal a module path of that file joined by `::` with the file-local qualified name of exactly one
function carrying the invariant marker; module paths come from the crate roots of the manifest that the command
of the oracle check names.

## Workspace

Branch: feature/cargo-oracle-file
Revision: 80fd7d14b9ec8f9c030863d11a8316708b999dbf
The Cargo manifest reader (project_setup/rust/cargo.rs) and the module path walk of config-check (coverage.rs)
move together into Project/structural_lint/architecture/cargo_crates.rs, offered as CargoLayout, cargo_layout
and cargo_modules(root, files, argv, cwd), since project_memory cannot use project_setup (project_setup uses
project_memory). The manifest path is canonicalized instead of normalized lexically. The fs_with_context budget
rises from 24 to 25 for this reader; the D136 and D144 links move to the new file.

## Progress

- oracles.rs: `marked` compares a cargo target with module path plus qualified name and walks once per command.
- tests/oracles.rs covers AC1 (another scope, a partial segment, a repeated name) and AC2 (a marked test in a
  file the target does not reach, a file no crate root reaches).
- The `// DECISION: D157` marker stays while the code replaces the trailing-segment match of D157. The successor
  decision D238 follows in the next VAC.

## Verification

These results hold for the tree before the rebase onto master; the gated D238 commit checks the rebased tree.

- `just run test -- project_memory`: 47 passed; `just run test -- structural_lint`: 354 passed.
- `just run test -- project_setup`: 60 passed.
- `just check --only rustfmt --staged`: pass after applying the rustfmt line splits.
- `just lint`: clean.
- `just candidate memory-check`: passes with the three known marker warnings (D112, D175, D231).

## Blockers

None for this VAC. Completing P200 needs the D157 successor D238 and a pin.

## Next action

Continue the rebase, skip the obsolete pause commit, then record D238 in a gated VAC.
