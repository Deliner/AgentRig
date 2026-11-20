# State

## Focus

P171 (B176 entry-use) and P172 (B178 entry-bypass), one branch from master, autonomous under the user's consent
(D226). VAC 6: `structural_lint/rule_identities` is used only through its entry, which reexports its API.

## Workspace

Branch: feature/entry-boundaries
Revision: 1dc70eef60610f5f3548bced3d4e16f99725f069

## Progress

- VACs 1 to 5 and the consented pin of dae94a4 are committed.
- The rule identity parts are private modules; the entry reexports the identities, descriptor, parameters,
  settings and pairs, and `FILENAME` is `CONTRACT_FILENAME` at the entry; 119 users import from the entry.
- The scratch measurement fell from 1224 to 277 entry-bypass findings, none of them into `rule_identities`;
  entry-use stays at 225.

## Verification

- `just test structural_lint::` passed 345; `just run rustfmt` passed.

## Blockers

None.

## Next action

Commit this VAC, then declare `rule_set` and `targeting` groups and give the features their entry APIs.
