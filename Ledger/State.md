# State

## Focus

P188 (B195): the decision marker check hands its resolved application links to the reference check.

## Workspace

Branch: feature/shared-applications
Revision: 31d425c003dc356f247adb0e1ac51a65b164390f

## Progress

VAC 2: the decision marker check resolves each Applies in link once, checks markers in the same order and returns
the resolved links by decision. The reference check takes them instead of reading Decisions.md and resolving its
links again.

## Verification

`just test` passes 617 tests, as on master `9e1b654`. `just lint` and the rustfmt check report nothing. Memory
check median 409 → 382.5 ms on AgentRig.

## Blockers

None.

## Next action

Commit VAC 2, then archive P188 and B195 with the timings and acceptance results and run `just feature-merge`.
