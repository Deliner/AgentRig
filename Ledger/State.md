# State

## Focus

P171 (B176 entry-use) and P172 (B178 entry-bypass), one branch from master, autonomous under the user's consent
(D226). VAC 3: the dependency rules report one finding per source line and target, not one per imported item.

## Workspace

Branch: feature/entry-boundaries
Revision: 5bc583781096cb8ea9636b4915b2572258882a81

## Progress

- VAC 1 and the consented pin of dae94a4 are committed.
- `findings` in `contract_check/dependency_rules.rs` keeps distinct issues; `Issue` orders by path, line and message.
- A scratch configuration measured the rig: 225 entry-use and 1434 entry-bypass findings with `shared` as a group;
  940 of the latter use files of `structural_lint/rule_identities` past its entry.

## Verification

- `just test dependency_rules::` passed 10; the entry-use fixture now imports two items of the entry on one line.

## Blockers

None.

## Next action

Commit this VAC, then route the uses of `shared` parts through their entries and declare `shared` a group.
