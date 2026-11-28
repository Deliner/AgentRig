# State

## Focus

P203 (B138), VAC 4 of 4: each Clippy profile lint leads its finding to the skill that repairs it (AC1, AC2); any
other compiler message and a run that cannot check the project keep the configuration skill (AC3).

## Workspace

- Branch: feature/clippy-lint-skills
- Revision: a8dbc2a
- Code: `structural_lint/clippy` profile table pairs each lint with its skill; messages carry the skill under the
  configured `paths.skills`; `rule_evaluation/findings.rs` falls back to `config_skill`; the lint command and the
  gate pass the skills directory.
- Tests: `structural_lint/tests/clippy.rs` (as_conversions, unwrap_used, unused_variables), init test in
  `project_setup/tests/clippy.rs`, profile table test in `structural_lint/clippy/tests`.
- Memory: D240 narrows D239's configuration skill clause; `Project/README.md` lint paragraph; architecture texts.

## Progress

- VAC 1 `29508a5`, VAC 2 `56c4d81`, VAC 3 `a8dbc2a`: every profile lint is named in the skill that repairs it.
- VAC 4 written: profile entries carry skills, findings name them, D240 recorded; commit pending.

## Verification

- `just run rustfmt` and `just check --only lint` (installed `f2d0320`) pass.
- `just run test -- clippy`: 19 passed, among them the profile table test and the three changed Clippy tests.
- Probe in `Project/shared/console/mod.rs`, removed after the run: `just run lint` (candidate) reported
  `clippy::as_conversions` with `.claude/skills/convert-without-as/SKILL.md`, `clippy::unwrap_used` and
  `panic-path` with `handle-panic`, `unused_variables` with `repair`.

## Blockers

None.

## Next action

Implement VAC 4, run `just candidate lint`, the Clippy tests of structural_lint and project_setup, and commit; then
`just propose-pin HEAD` under the consent of the user, archive P203 and B138 and run `just feature-merge`.
