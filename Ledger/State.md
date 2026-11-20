# State

## Focus

P169 (B177, D224): bring the rig's Rust test code out of code files before the rule is enabled.

## Workspace

Branch: feature/test-code-out
Revision: f5e51af03f3129d8ddd112386782458ed4801eb8

## Progress

- VAC 1 committed: rule kind, detector, skill `move-test-code-out` and D224, `init` leaves the kind out until phase 2.
- VAC 2: grouped findings are tested through `lint --warnings`; hook input, Git context and cfg predicate tests moved
  to `tests/` directories declared in their entries; the Git context child test now runs under its real name and
  must report one passed test; the Rust example keeps `tests::doubles` in `tests/mod.rs`.
- Remaining: mid pin, lint.yaml proposal, phase 2 with `init` delivering the kind, archive, final pin, merge.

## Verification

- `just run test`: 597 passed.
- `just lint`: clean; candidate lint with only `test-in-code-file` on the rig: no findings.
- Probe project: `src/mesh.rs:6 (tests)` reported with the move hint; the bodiless entry declaration accepted.

## Blockers

None.

## Next action

Commit VAC 2, run `just propose-pin HEAD` and ask the user to type `! just approve`.
