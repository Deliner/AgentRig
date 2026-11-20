# State

## Focus

P169 (B177, D224): pin the runtime that knows `test-in-code-file` so lint.yaml can name the rule in phase 2.

## Workspace

Branch: feature/test-code-out
Revision: 0749148133f6d0d13193e99dcafe41ae3d65177c

## Progress

- VAC 1 and VAC 2 committed: rule kind, detector, skill, D224, and the rig's Rust test code moved to `tests/`.
- Mid pin: the user approved 0749148; stable.txt names it.
- Remaining: lint.yaml proposal, phase 2 with `init` delivering the kind, archive, final pin, merge.

## Verification

- Gate at 0749148: 580 affected tests passed, lint, memory, rustfmt and clippy passed.
- `just approve` installed 0749148 and wrote it to stable.txt.

## Blockers

None.

## Next action

Commit the pin, propose lint.yaml with `test-in-code-file` and ask the user to type `! just approve`.
