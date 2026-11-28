# State

## Focus

Record D236 on P199 (B206 AC1–AC4): its index row, `Ledger/Decisions/236.md` in Decision, Reasons and Alternatives,
and `// DECISION: D236` in the five Rust files the row links.

## Workspace

Branch: feature/decision-sections
Revision: 606537924da5f1d64898e68ef4e1bd541d3138f5

## Progress

- `6065379`: the six memory index files are outside `file-size` in the rig and after `init`, recorded as D237.
- In the working tree: the D236 row between D235 and D237 and `Ledger/Decisions/236.md`; the marker stands above
  `PRIOR_DECISION_SECTIONS` in `Project/project_memory/format.rs`, above `decision_details` in
  `Project/project_memory/mod.rs`, at the head of `Project/project_memory/history.rs` and above the tests
  `a_new_decision_has_three_sections_while_a_committed_one_keeps_the_prior_five` in
  `Project/project_memory/tests/validation.rs` and
  `after_init_the_commit_gate_takes_a_new_decision_only_in_three_sections` in
  `Project/change_delivery/git/tests/guards.rs`.
- No committed decision states the five prior sections, so D236 needs no succession note.

## Verification

- `just candidate memory-check` and `just check --only memory` passed with D236 and D237 new in three sections,
  with only the D112, D175 and D231 warnings; `just lint` and `just run rustfmt` passed.
- The gate of `6065379` passed 638 Rust tests and every other check.

## Blockers

None.

## Next action

Commit this VAC; then `just propose-pin HEAD` under the consent of the user, one commit with the pin and the archive
of P199 and B206, and `just feature-merge`.
