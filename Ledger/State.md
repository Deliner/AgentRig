# State

## Focus

P171 entry-use and P172 entry-bypass under the user's consent: VAC 18 moves the items of the resolution rust and
python entries and of the extraction references entry into named files.

## Workspace

Branch: feature/entry-boundaries
Revision: 964b07737e95fb54aec25297e4fc61aa20e33d02

## Progress

- resolution rust: resolver.rs holds Rust with Module; module_tree.rs holds Pending; use_aliases.rs holds Located
  and Query.
- resolution python: resolver.rs holds Python with module_parts; invalid_module_names_are_rejected checks names
  through Python::new.
- extraction references: reference.rs also holds References, PropagatedCall, RustItem and RustImport.
- Decision links moved: D136 tree to the rust resolver.rs, D125 model to reference.rs.

## Verification

- clippy, rustfmt and the resolution tests pass: 27 tests. The first gate run failed on function-lines 43 in the
  Python root test; the name check is now its own test.
- Candidate measurement with architecture, file-cycle and entry-use: entry-use 80 to 45, file-cycle 0, architecture 0.

## Blockers

The checks.yaml impact-map proposal for cli_dispatch commands.rs and dispatch.rs waits for the user's approval.

## Next action

Commit VAC 18, then split the extraction rust entry helpers into path syntax and recording files.
