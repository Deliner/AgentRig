# State

## Focus

P147 (B148): AGENTS.md and CLAUDE.md carry one text. VAC3 pins `70bdfad` and enables `same-text` for AC2: the
`init` catalog and the rig's lint.yaml select `AGENTS.md` and `CLAUDE.md`, so a commit where they differ fails.

## Workspace

Branch: feature/one-agent-text
Revision: 70bdfaddd4561500601e79f4690a6d0ee842d607

## Progress

- Done in this VAC: stable.txt pins `70bdfad`; `same-text` is cataloged for `init` and enabled in the rig; the
  delivery test commits a differing `CLAUDE.md` in both consumer layouts and expects a refusal naming the absolute
  condense-markdown path; D195 links the new owners; Plan/147 Delivery records VAC2 and VAC3.
- Not yet: the AC3 `AGENTS.md` template.

## Verification

- `just run test -- project_setup`: 58 passed, including both delivery layouts with the refused commit.
- `just run test -- structural_lint::tests`: 104 passed.
- `just lint`, `just lint-config-check` and `just run rustfmt` pass.

## Blockers

None.

## Next action

Commit this VAC through the gate, then write the AC3 template.
