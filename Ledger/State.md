# State

## Focus

Ship the `file-size` exclusion of the six memory index files on P199 (B206 AC5) in one commit: the rig
`.agentrig/lint.yaml` that the user approved, the `init` lint template and its test, and D237 with its markers.

## Workspace

Branch: feature/decision-sections
Revision: 733c37d98b0d7371cfe36314ebd6a29dda37ee6f

## Progress

- The user ran `! just approve`: `.agentrig/lint.yaml` excludes `Ledger/Decisions.md`, `Invariants.md`, `Plan.md`,
  `Backlog.md`, `Archive/Plan.md` and `Archive/Backlog.md` from `file-size` under a comment that names D237.
- `Project/project_setup/bundle/lint.rs` excludes `{memory}/NAME.md` for each name of `MEMORY_REGISTRIES` after the
  lint configuration and `Assets/**`; `Project/project_setup/tests/defaults.rs` checks that an `init` project with
  600 lines in each index and in the card `Ledger/Plan/001.md` gets a `file-size` finding for the card only.
- `Ledger/Decisions/237.md` records D237 in Decision, Reasons and Alternatives, beside its index row; the last notes
  paragraph says that D237 narrows D116; `lint.rs` and `defaults.rs` carry `// DECISION: D237`.

## Verification

- `just run test project_setup::tests::defaults::shipped_file_size_` passed both tests in 13.5 s; the flagged card
  shows that the rule measures Ledger Markdown, so only the exclusion spares the indexes.
- `just run test project_setup::initialization::tests::rig::` passed: the `init` template matches the approved rig
  `lint.yaml`; before the approval the gate refused the template for these six exclusions.
- `just candidate memory-check` passed with only the D112, D175 and D231 warnings; `just lint` and
  `just run rustfmt` passed with `Ledger/Decisions.md` at 501 nonblank lines.

## Blockers

None.

## Next action

Commit this VAC; then D236 with its markers, `just propose-pin HEAD`, the archive commit and `just feature-merge`.
