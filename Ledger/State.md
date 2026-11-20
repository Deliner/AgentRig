# State

## Focus

P170 (B175, D225): break the file cycles of the rig before `file-cycle` is enabled; this VAC fixes 7 of 8 directories.

## Workspace

Branch: feature/file-cycle
Revision: 05110a07929d2e3ec5b2e48b20dea7ca20121b2d

## Progress

- change_delivery/git: the proposal directory moved to `proposal.rs`.
- project_memory: reading memory at a revision moved to `revision_memory.rs`.
- project_setup: the hook registration check moved to `adapters.rs`.
- project_setup/initialization: the choice of interactive or declarative init moved to the entry.
- structural_lint/architecture: the contract model with `Kind` stays in `policy.rs`, its checks moved to `contracts.rs`.
- structural_lint/reporting: the inspection command table moved to `cli.rs`.
- structural_lint/reporting/structure: the identifier kinds moved to `clusters.rs`.

## Verification

- The scratch probe with `file-cycle` reports only the 6 cycles of structural_lint/rule_identities.
- `just lint`, `just run rustfmt` and `just check --only memory` pass.

## Blockers

None.

## Next action

Commit, then break the cycles through `descriptor.rs` and `kind.rs` in structural_lint/rule_identities.
