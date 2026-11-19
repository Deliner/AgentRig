# State

## Focus

P150 (B151) allowed program languages, VAC1: the `program-language` lint rule kind, its language table, settings
validation, evaluation, catalog entry and the rewrite-in-allowed-language skill (D198). `just write` also runs `ln`.

## Workspace

Branch: feature/allowed-languages
Revision: 62dbf91cf5e8825bf8bf61b6362fc9fc63eb608f
Stash `P150 VAC2 init template` holds the init template change for VAC2.

## Progress

Rule kind, table of 16 languages, `allowed_languages` validation through capabilities, evaluation with the language
and allowed list in the message, skill in the refactor group behind a flat link, D198 recorded. The init template
and the rig rule wait for VAC2, because the installed runtime must know the kind first and I020 requires both at once.

## Verification

`just run test -- structural_lint project_setup`: 366 passed. Candidate lint, Clippy, config-check and the memory
check pass.

## Blockers

None.

## Next action

Commit VAC1, run `just bootstrap HEAD`, pop the stash, add the rig rule and the init rule tests in VAC2.
