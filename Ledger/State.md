# State

## Focus

P196 (B203, D230, D231): after a merged branch the agent compacts its context. VAC3 ships the compaction mod
`self-compact` with the rig (AC8): source in `Project/shared/mods` as the marketplace `agentrig`; `init` writes it to
`.agentrig/mods`, enables `self-compact@agentrig` in `.claude/settings.json`, protects `mods/` and excludes it from
program-language; the rig holds the same files, approved by the user's `! just approve`.

## Workspace

Branch: feature/compact-after-merge
Revision: 18c7f5d186dcb03ab4dde44b4a8d81b5393e9819

## Progress

- VAC1 (7352c62) saved B203 and selected P196; VAC2 (18c7f5d) added the requirement, refusal and lift (AC1–AC6).
- VAC3: the mod without the commit suggestion; `adapters::settings` gives the hooks value that `claude::bundle` extends
  with the marketplace; the lint template excludes `mods/**`; protection adds `mods`; D231 with its index row.
- The user approved `.agentrig/mods/**`, `.claude/settings.json`, `.agentrig/lint.yaml` and `rig_differences.yaml`.

## Verification

- `just run test -- agents`: 6 passed, including `init_enables_the_compaction_mod_that_its_marketplace_ships`.
- `just run test -- protected`: 6 passed; edits of `.agentrig/mods/...` and `.cache/runtime/compaction` are refused.
- `just run test -- initialization::tests::rig`: passed after the approval; `just run lint`: clean.
- First commit gate: 17 failures, since `init` marked the mod's `hooks/` files executable and the tools check took
  them for tools. Fixed: `init` marks executable only the service `bin/` and `hooks/` (D231); `tool_registry` 3 and
  `change_delivery::git` 35 tests pass.
- `project_setup::tests` failed twice with `No space left on device` while the disk was 99% full (8.5 GiB free), then
  passed 28 of 28 on a rerun.

## Blockers

None; the disk has about 8.5 GiB free, so another `No space left on device` needs the user to free space.

## Next action

Commit VAC3. Then delete `~/.claude/skills/self-compact` (AC9), check the mod and Codex live with the user (AC8, AC10),
`just propose-pin HEAD`, archive P196 and B203, `just feature-merge` and check AC7 live.
