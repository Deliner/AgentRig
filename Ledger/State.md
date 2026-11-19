# State

## Focus

P147 (B148): AGENTS.md and CLAUDE.md carry one text. VAC2 delivers AC1 and the user's root choice: `init` writes a
root `CLAUDE.md` copying `AGENTS.md`, the rig keeps the same pair, and the recipe groups move to `.agentrig/just/`
under D212, superseding D039's root `just/`.

## Workspace

Branch: feature/one-agent-text
Revision: c5bba5328baebb47c1c6deaf5a7db43f4e6e00ae

## Progress

- Done in this VAC: root `CLAUDE.md` and no `.claude/CLAUDE.md` in `init` and the rig; recipe groups generated and
  kept in `.agentrig/just/`; the rig comparison, layout test and changed-path map follow; D212 with its index row;
  recipe links in the decision index point into `.agentrig/just/`; `example` in describe.rs split after the rebase.
- Plan: P147 active again with the rebase and the user's answer in Delivery; the answer for P156 and P157 is noted.
- Not yet: the D051 pin, `same-text` in the `init` catalog and the rig's lint.yaml with a gate test (AC2), AC3.

## Verification

- `just run test -- project_setup`: 58 passed, including the pair test and the rig comparison.
- `just lint`, `just run clippy`, `just run rustfmt` and `just check --only memory` pass.

## Blockers

None.

## Next action

Commit this VAC through the gate, then pin it with `just bootstrap HEAD` and enable `same-text` for AC2.
