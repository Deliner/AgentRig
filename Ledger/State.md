# State

## Focus

P167 (B174) VAC 6: the repair skills describe the exception `DECISION: DNNN RULE` and no bare marker lift (AC5).

## Workspace

Branch: feature/named-exceptions
Revision: afc174d3c02318611ee9f87d6cc0acd450991059

## Progress

- The 54 skills of rules that accept an exception place it directly above the reported line and list it in the decision.
- replace-comment repairs an exception that lifts nothing; handle-panic describes the covering attribute of D147.
- Skills of rules without exceptions say that no reference lifts their finding; descriptions say "exception".

## Verification

- `just run test`: 593 passed. `just lint --json`: no findings. No skill offers a bare marker as a lift.

## Blockers

None.

## Next action

Archive P167 and B174, propose the pin at the gate-accepted head and ask the user to approve it.
