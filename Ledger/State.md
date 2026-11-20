# State

## Focus

P171 (B176 entry-use) and P172 (B178 entry-bypass), one branch from master, autonomous under the user's consent
(D226). VAC 5: every use of a `shared` part goes through its entry, and `shared` and `shared/testing` are groups.

## Workspace

Branch: feature/entry-boundaries
Revision: 1b434971411efea60a973ad8b2a8439075fab45c

## Progress

- VACs 1 to 4 and the consented pin of dae94a4 are committed.
- The entries of `paths`, `diagnostics` and `atomic_files` reexport their API and declare their parts private;
  `load_json`, `save_json` and `validate_skill` name the renamed functions at the entry.
- `Project/shared` and `Project/shared/testing` declare `group: true`; each child keeps its own entry.
- The scratch measurement fell from 1428 to 1224 entry-bypass findings, none of them into `shared`; entry-use
  stays at 225.

## Verification

- `just test shared::` passed 33; `just run rustfmt` passed.

## Blockers

None.

## Next action

Commit this VAC, then reexport the rule identities from `structural_lint/rule_identities/mod.rs` for its users.
