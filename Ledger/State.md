# State

## Focus

P147 (B148): AGENTS.md and CLAUDE.md carry one text. VAC1 adds the `same-text` lint kind, D195 and the
condense-markdown step that repairs it; the kind is not yet in the `init` catalog or the rig's lint.yaml, because the
installed runtime rejects unknown kinds until the D051 pin.

## Workspace

Branch: feature/one-agent-text
Revision: 51f55f7c90886177844aedabd534269a556f6b09

## Progress

- Done in this VAC: `Structure::SameText` with its descriptor, measurement, capability entry and example selecting
  `AGENTS.md` and `CLAUDE.md`; condense-markdown step 6; D195 and its index row.
- Kept out of this VAC: AC1, the root `CLAUDE.md` copy written by `init` and kept by the rig. With it the rig root has
  16 entries against the directory-size limit 15; the user chooses how to resolve it.

## Verification

- `just run test -- structural_lint`: 312 passed, including the new same-text test.
- `just run test -- project_setup`: 55 passed with the AC1 change present in the working tree.
- The commit gate verifies the staged tree without AC1.

## Blockers

AC1 waits for the user's choice on the rig root entry count.

## Next action

Commit this VAC through the gate, then pause P147 in a plan-only VAC and continue with P148.
