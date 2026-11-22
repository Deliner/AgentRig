# State

## Focus

Maintenance under the user's overnight speed-up authorization, no business logic change: the generated Git hooks
take the project root from the working directory, where Git runs them, instead of starting
`git rev-parse --show-toplevel` and passing `--root` on every call. Agent hook adapters keep resolving the root.

## Workspace

Branch: feature/cwd-git-hooks
Revision: dded29a812c39bccabc24daa00fc4b584855e7fa

Uncommitted: `Project/shared/vcs/hooks.rs` (git hooks without the root line and `--root`; `generate` returns only
the files), `Project/project_setup/bundle/adapters.rs` (owns the root command of the agent hooks, its only user;
git hooks name the service launcher relatively), `bundle/{mod,claude}.rs`, `inspection/doctor.rs` and
`shared/vcs/mod.rs` (callers without the root parameter; registration checks return `bool`, as nothing fails),
`Project/change_delivery/git/tests/guards.rs` (commit, State reset and branch retention started in a
subdirectory), and the user-approved `.agentrig/hooks/{pre-commit,reference-transaction,post-commit}` with the pin
`tooling/distribution/stable.txt` of 3b1ed2e. dded29a on this branch registers job records once.

## Progress

Generator, adapter, callers and test are written. The user approved the matching repository hooks and the pin of
3b1ed2e; I023 compares the hooks with the init output, so all commit together.

## Verification

The full `just run test` passed 633 of 634 tests before approval, failing only the rig parity test on the three
hook bodies; with the approved hooks that test (I023) passes; Codex and Claude settings match. `just check --only` lint,
rustfmt, clippy and memory pass. The installed runtime f72dee2 accepts `guard-reference` without `--root` from the
root. Fixture timing: feature-merge 986 -> 899 ms, commit 419 -> 388 ms; whole suite unchanged.

## Blockers

None.

## Next action

Commit the VAC; `just propose-pin HEAD` and ask for the second approval; commit the pin; `just feature-merge`.
