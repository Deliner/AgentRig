# State

## Focus

P152 (B153 AC1–AC4), VAC1: the hook allows a just command only as zsh and just will run it, and recipes keep the
project of their justfile (D201).

## Workspace

Branch: feature/hook-shell-parse
Revision: 702a28f2b4562e9b74ea736e7da6f42d5ee612fc

## Progress

- Shell guard refuses an unquoted `(` and, in double quotes, `$(`, a backtick and `${(`; only `just` by name passes.
- The hook resolves recipes through init's recipe table: no word after a recipe without parameters, no bare `just`;
  `run`, `job-start` and catalog recipes get the catalog checks and write routes of their command.
- Generated and rig recipes pass `--root .` before the agent's words.
- D201 recorded; the D197 marker moved to the recipe resolver.

## Verification

- `just run test -- agent_hooks project_setup command_runner`: 119 passed.
- The recipe root test fails on the old recipe form, as checked by removing `--root .` from the generator.
- `just lint`, `just run clippy`, `just check --only memory` and `just config-check` pass.

## Blockers

None.

## Next action

Commit VAC1, run `just bootstrap HEAD`, probe the refusals live in zsh, then archive P152 and B153 and merge.
