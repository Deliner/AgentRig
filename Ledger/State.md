# State

## Focus

P158 (B159): agent texts agree with each other and with the rig's commands. This VAC delivers AC1 and AC2.

## Workspace

- Branch: feature/agent-texts
- Revision: 4de4a6bd6a5e712bfd286c0e421a24974a9a04eb

## Progress

- AC1 text fixes:
  - F4: repair and edit-plan name `just config-check`, `just lint-rules`, `just check --only memory` and the user's
    terminal `doctor`.
  - F5: RERUN runs through `just check` or the failed recipe; develop-tools reruns `just check --only tools`.
  - F6: edit-backlog uses the user's language, a draft form matching Context, Proposal and Expected benefit, and
    "index and card".
  - F7: the Decisions.md precedence paragraph becomes one rule; its pairs move into succession notes.
  - F8: testing-discipline Enforcement names its rules.
  - F9: clarify-structure commits each round as one VAC.
  - AGENTS.md: complete hook routes, one skill location and the runtime path.
  - Distribution README: the unmatched-path wording agrees with the gate README.
  - execute-plan-feature: `just write`.
  - Consumer example: Just recipes for Git, one gate, the State step and `just check --only memory`.
  - testing-discipline: `// DECISION: DNNN`.
  - Test purpose: filtered by test name for Rust.
  - Deferral by the user only, recorded in the agreed card.
- AC2: `agent_hooks::tests::named_commands` checks every `just` command in AGENTS.md, README and SKILL texts through
  the hook, with the rig catalog and after init, and is mapped in .agentrig/checks.yaml.

## Verification

- The focused tests pass. With a planted `just doctor` the rig test fails, which shows it is falsifiable.
- `just lint`, rustfmt and Clippy pass.

## Blockers

None.

## Next action

Run `just check --only memory`, commit this VAC, then `just bootstrap HEAD`, commit the pin, archive P158 and B159,
and run `just feature-merge`.
