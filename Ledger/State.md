# State

## Focus

P159 (B161), VAC 2: enable `indexed-array-literal` with its skill branch-on-the-index, with the mid-branch pin.

## Workspace

- Branch: feature/array-index-literal
- Revision: 6badc8e376d2e93f84e4a13c2fe183bc7da951cf

## Progress

- VAC 1 `6badc8e` added the inert kind, detector, tests and D205; its gate passed with 490 tests.
- `just bootstrap HEAD` installed `6badc8e` and wrote it to stable.txt.
- This VAC adds the kind to the Rust handler and the catalog, which init ships, the skill with its link and init
  registry entry, and the rig's lint.yaml rule.

## Verification

- `just lint` passes with the rule enabled, so the rig has no finding.
- `just lint-config-check` on the installed runtime accepts the rule.
- `just run test -- project_setup::`: 57 passed; `just run test -- structural_lint::`: 314 passed.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, archive P159 and B161 with the pin, and run `just feature-merge`.
