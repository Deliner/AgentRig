# State

## Focus

P171 (B176 entry-use) and P172 (B178 entry-bypass), one branch from master, autonomous under the user's consent
(D226). VAC 8: the architecture check is used only through the `architecture` entry.

## Workspace

Branch: feature/entry-boundaries
Revision: 5cb44659159cf6d29688d7f7e0a2241cbed7071f

## Progress

- VACs 1 to 7 and the consented pin of dae94a4 are committed.
- The records, effects, contracts and graph rules moved from `architecture` into `architecture/contract_rules`,
  so `contract_check` no longer uses a file of its parent and the entry reexports its API without a cycle.
- `architecture/mod.rs` only declares its parts and exports the check, `Neighbours` and `Rooted`; consumers call
  `architecture::run`, `validate`, `edges` and the dependency rules; the decision links follow the moved owners.
- `extraction` exports `PropagatedCall`, the type of a public `References` field.
- The commit guard read the moved D156 variant-equality exception in `policy.rs` as new, so `relative` now
  matches every `Component` variant and the exception and its D156 link are gone.
- Scratch measurement before the last export: entry-use 219 to 192, entry-bypass 171 to 160, no cycle.

## Verification

- `just run clippy` passed; `just test structural_lint::` passed 345; `just run rustfmt` passed.

## Blockers

None.

## Next action

Commit this VAC, then route the cross-feature uses through the entries of the features they use.
