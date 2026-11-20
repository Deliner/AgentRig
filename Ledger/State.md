# State

## Focus

P169 (B177, D224): rule `test-in-code-file` and skill `move-test-code-out`, phase 1 of two.

## Workspace

Branch: feature/test-code-out
Revision: 4846866f6f410d5286492f2682b365892f7bbc3a

## Progress

- VAC 1: kind `test-in-code-file` with descriptor, detector on outermost test-marked items, Rust handler, capabilities
  and evaluation arms; `Code` carries the file path; entry names shared from Rust exemptions; `init` leaves the kind
  out until phase 2; skill `move-test-code-out` installed; `relocate-tests` and `testing-discipline` aligned; D224.
- Remaining: move the inline tests of four code files and the example `lib.rs`, mid pin, lint.yaml proposal, phase 2.

## Verification

- `just run test -- test_in_code_file`: 3 passed; full suite before the entry-names move: 596 passed, discovery fixed.
- `just lint`: clean; `just check --only memory`: pass.

## Blockers

None.

## Next action

Commit VAC 1, then move test code out of actionable_findings.rs, input.rs, git.rs, conditions.rs and example lib.rs.
