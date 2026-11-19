# State

## Focus

P155 (B156) VAC 1: `write` refuses Git's `--no-verify` and `-n`, and a catalog command cannot change the base (D204).

## Workspace

Branch: feature/base-protection
Revision: 5a2ae0fd076bc72335b76dbd381c7dc54a69d825

## Progress

- Init's and the rig's `write` deny `git` `--no-verify` and `-n`.
- The runner sets `WORKER_CATALOG_ROOT` to the canonical root for every catalog process; the reference-transaction
  guard refuses an update of `refs/heads/<base>` when it names the repository; `feature-merge` removes it from its Git.
- The P154 test that commits through `write` now commits on a feature branch.
- D204 and its index row, the guide clause and the P155 delivery section are written.

## Verification

- `just run test -- change_delivery agent_hooks command_runner project_setup shared`: 191 passed.
- `write_commands_cannot_change_the_base` refuses `branch -f`, `checkout -B`, `switch -C`, `branch -D`, rebase,
  cherry-pick and a commit with an empty `MERGE_HEAD` with the guard's message, and the base stays; without the marker
  the first case moves the base and the test fails.
- The hook test refuses `--no-verify`, `--no-ver` and `-qnm`; without the new denies it fails on the reason.
- `just lint`, `just run clippy`, `just run rustfmt` and `just config-check` pass.

## Blockers

None.

## Next action

Commit VAC 1, pin it with `just bootstrap HEAD`, probe `just write -- git branch -f master HEAD` live, archive P155 and
B156 in the pin commit, then `just feature-merge`.
