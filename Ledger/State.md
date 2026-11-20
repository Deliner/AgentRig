# State

## Focus

P171 (B176 entry-use) and P172 (B178 entry-bypass), one branch from master, autonomous under the user's consent
(D226). VAC 2: pin the runtime that knows both kinds and the contract field `group`, so the rig can configure them.

## Workspace

Branch: feature/entry-boundaries
Revision: dae94a426d799407cad0a8d18c189ee7df539aa9

## Progress

- VAC 1 committed: kinds `entry-use` and `entry-bypass`, their detectors, skills, tests and decisions D227 and D228.
- `just propose-pin HEAD` gated and installed dae94a4 under consent and wrote `tooling/distribution/stable.txt`.

## Verification

- The propose-pin gate passed build, lint, memory, rustfmt, clippy, 537 affected Rust tests and the Python checks.

## Blockers

None.

## Next action

Commit the pin, then measure both rules on the rig with a scratch configuration and repair the findings.
