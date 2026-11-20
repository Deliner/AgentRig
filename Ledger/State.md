# State

## Focus

P167 (B174): pin the revision that parses named exceptions and approves new ones in the gate (D218, D220).

## Workspace

Branch: feature/named-exceptions
Revision: 5f87bcbbcc3db9a9ae45557a27a5178b03b21e91

## Progress

- VAC 1 to 3 are committed: Plan and measurement, the exception form with decision lists (D219), the approval (D220).
- The user approved the pin 5f87bcb with `just approve`; it is installed and written to stable.txt.

## Verification

- `just propose-pin HEAD` listed 19 changed source files; the user's approval installed and pinned the revision.

## Blockers

None.

## Next action

Commit the pin, then VAC 4: name the rig's exceptions, list them in decisions, fix accidental lifts in code.
