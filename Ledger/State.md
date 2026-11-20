# State

## Focus

P170 (B175, D225): break the last file cycle of the rig, through `descriptor.rs` and `kind.rs` in rule_identities.

## Workspace

Branch: feature/file-cycle
Revision: 95c52a70ade493aaf7546aa50448f027194aaee2

## Progress

- The dispatch `Kind::descriptor` moved from `descriptor.rs` to `kind.rs`; `descriptor.rs` no longer uses `kind.rs`.
- The earlier commit of this branch broke the cycles of the other 7 directories.

## Verification

- The scratch probe with `file-cycle` over the whole rig reports no finding.
- `just lint`, `just run rustfmt` and `just check --only memory` pass.

## Blockers

None.

## Next action

Commit, propose the mid pin, then propose lint.yaml with the `file-cycle` rule for the user's approval.
