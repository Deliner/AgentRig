# State

## Focus

P157 (B158), VAC1: AC1 and AC3. The protected set holds check settings, the lint configuration and its baseline;
without approval a staged baseline commits only when it adds no pair and raises no count (D214).

## Workspace

Branch: feature/gate-base-rules
Revision: c2052f200e6273f2394bc4e7d11cdf50af8ca33a

## Progress

- Protection covers `checks.yaml`, `tools.yaml`, `rust.yaml`, `paths.lint` and its baseline; the commit guard admits a
  lowered or removed baseline without approval through the new leaf `structural_lint/baseline_file`.
- Hook, delivery, approval and init offer tests follow the wider set; D214 row, detail and precedence note written.
- Remaining ACs: AC4 and AC5 history against the fork point, AC2 marker links, AC6 installed lint in the rig gate.

## Verification

- `just run test`: 582 passed before the baseline split; after it the baseline, approval and init layout tests pass.
- `just run clippy`, `just run rustfmt` and `just lint` pass.

## Blockers

None.

## Next action

Commit VAC1 through the hook, then start VAC2: compare decisions and invariants with the fork point from the base.
