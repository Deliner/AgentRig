# State

## Focus

P153 (B154 AC1–AC2), VAC1: a denied flag is refused in every spelling its program reads, and init's test command takes
only operands (D202).

## Workspace

Branch: feature/program-arguments
Revision: 654e76f16c196a7eda7a15b6a0ac5935d4128c44

## Progress

- `deny` entries are a typed `Flag` parsed at load: `--name` denies abbreviations and `=` values, `-c` denies clusters
  containing c, `-name` denies words starting with it; an entry in no form is a configuration error.
- The rig's `rg` denies `--pre` and `--hostname-bin`.
- `accepts_args` takes `operands`, which refuses agent words starting with `-`; init's `test` command uses it.
- The rig's own test command stays `accepts_args: true`, listed as a rig difference, until the pinned runtime reads
  `operands`.
- D202 recorded.

## Verification

- `just run test -- agent_hooks project_config project_setup command_runner project_memory`: 169 passed.
- With the old prefix matcher and with `operands` accepting options, the hook test and both delivery tests fail.
- `just lint`, `just run clippy` and `just check --only memory` pass.

## Blockers

None.

## Next action

Commit VAC1, run `just bootstrap HEAD`, switch the rig's test command to `operands`, probe live, archive P153 and B154,
then merge.
