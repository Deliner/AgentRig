# State

## Focus

P167 (B174) VAC 5: lint lifts only the rule an exception names on the line below and reports one that lifts nothing.

## Workspace

Branch: feature/named-exceptions
Revision: 57fb11af2784a7e25b9a659abebea082b567af9f

## Progress

- `structural_lint/lifting.rs` lifts a finding only under `DECISION: DNNN RULE` naming its rule on the line above.
- The comment rule reports every exception that lifted nothing in a file that parsed (AC3).
- Detectors no longer read markers; language tests are detection-only; `tests/lifting.rs` covers AC1 and AC3.
- D222 records the choice and supersedes the marker lift of D065, D096, D103 and D147.

## Verification

- `just run test`: 593 passed. `just lint --json`: no findings. Clippy, rustfmt and the memory check pass.

## Blockers

None.

## Next action

VAC 6: rewrite the repair skills for the exception form (AC5), then archive P167 and propose the pin.
