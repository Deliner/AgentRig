# State

## Focus

P168 (B179): tests live only in a `tests/` directory. Current VAC: move the rig's nine `tests.rs` files to
`tests/mod.rs` with their own inventories, so no rig test file depends on the `tests.rs` pattern.

## Workspace

Branch: feature/tests-directory
Revision: ed75e7b95ae5ceb91f6415063a50ebd57a7e813d

## Progress

The nine files moved with Git history; each new `tests/` has an architecture.yaml and its parent lists it as a
directory. Module paths and test names are unchanged. Decision links to the moved files now point at `tests/mod.rs`.

## Verification

`just lint` reports no finding; the commit gate runs the affected test groups.

## Blockers

None.

## Next action

Record the decision replacing D145, then extend `test-placement` to `tests.rs` files and Python test files and drop
`**/tests.rs` from the shipped test-code patterns; then ask for the mid-branch pin.
