# State

## Focus

Maintenance under the user's overnight speed-up instruction, no product feature: check each distinct skill path and
glob list once while validating the lint configuration, keeping every verdict and its first error unchanged.

## Workspace

Branch: feature/memoized-lint-validation
Revision: f31ad448a13388ac44edf55728a1e50e23e02f4d

## Progress

structural_lint/rule_set/lint_config/validation.rs routes the configuration skill, exclude, each rule's include,
exclude, warning and error skills and each override include through one Checked value, which remembers only
successful checks. The separate equal-skill branch is gone because a repeated skill is already a remembered success.

## Verification

structural_lint:: (358) and project_config:: (10) tests pass. A temporary uncommitted probe measured in release in an
init project: a lint configuration load fell from about 4.9 to 3.6 ms and `guard-reference prepared` from 8.7 to
7.7 ms. A suite-wide count found 3274 lint loads taking about 52 s of the suite's CPU time.

## Blockers

None.

## Next action

Commit through the hook, then `just propose-pin HEAD` and `just feature-merge`.
