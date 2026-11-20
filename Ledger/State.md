# State

## Focus

P169 (B177, D224) phase 2: enable `test-in-code-file` in the rig and deliver it through `init` (AC4).

## Workspace

Branch: feature/test-code-out
Revision: f9b5528729c977c0ff88a32d64ea7c126e161ce5

## Progress

- VAC 1, VAC 2 and the mid pin committed; the user approved lint.yaml with the rule after `test-placement`.
- Phase 2: `init` catalogs `TestInCodeFile`, so Rust projects receive the same rule the rig runs.
- Remaining: archive P169 and B177, final pin, merge.

## Verification

- `just run test -- project_setup`: 58 passed, including I023 that compares the rig setup with `init` output.
- `just lint`: clean with the rule enabled.

## Blockers

None.

## Next action

Commit phase 2 with the approved lint.yaml, then archive P169 and B177.
