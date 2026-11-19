# State

## Focus

P147 (B148): AGENTS.md and CLAUDE.md carry one text. VAC4 brings the `init` AGENTS.md template to AC3: the one-text
line, a State section and a Commands section with the general rules taken from a production consumer.

## Workspace

Branch: feature/one-agent-text
Revision: ec1ea2bf142938e07a98f2058627fdcb1072d757

## Progress

- Done in this VAC: `assets::instructions` opens with the one-text line, moves the State lifecycle into a State
  section with the state-size limits, and adds Commands: `just --list`, a catalog command before a second run,
  `just check --staged` and running only the needed checks. The Rust 1.100 section already comes from D175.
  D195 links the template; Plan/147 Delivery records VAC3 and VAC4.
- Not yet: the D051 pin of this head, the archive VAC and `just feature-merge`.

## Verification

- `just run test -- project_setup`: 58 passed; the delivery test lints the initialized Python and Rust projects.
- `just lint` and `just run rustfmt` pass.

## Blockers

None.

## Next action

Commit this VAC through the gate, run `just bootstrap HEAD`, then archive P147 and B148 with the pin.
