# State

## Focus

P167 (B174), VAC 3: a new lint exception commits only after the user's approval (AC4, D220).

## Workspace

Branch: feature/named-exceptions
Revision: 3c53fdde383f18cc48efaf5998a6ad5ce3b029c0

## Progress

- `change_delivery/git/exceptions.rs` keys an exception by file, decision, rule and the line beneath it.
- The feature commit guard refuses staged exceptions beyond `HEAD` and the approved ones and lists each.
- `approve` records and prints the new exceptions of the working tree; the record holds files and exceptions.
- `propose-protected-change` describes the approval of exceptions.

## Verification

- `just run test -- change_delivery:: project_setup:: agent_hooks::`: 146 passed, with the new approval gate test.
- `just candidate check --only memory`: PASS. `just lint --json`: no findings.

## Blockers

None.

## Next action

Commit VAC 3, run `just propose-pin HEAD` and ask the user to type `! just approve`, then commit the pin.
