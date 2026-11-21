# State

## Focus

P193 (B200): the architecture rule parses each source once and resolves dependencies on several threads.

## Workspace

Branch: feature/faster-rules
Revision: ca6d64a55889d457d2a5f6776ca9357fcf7c299a

## Progress

VAC 1 saves B200 and selects it as P193 by the user's standing instruction. The parallel dependency resolution in
structural_lint/architecture/contract_check and a temporary per-rule timer in rule_evaluation are in the working
tree, not staged.

## Verification

Release candidate on AgentRig, `just candidate lint`, warm runs of the architecture rule: before on `ca6d64a` 298 and
305 ms; with parallel dependency resolution 243, 236 and 248 ms.

## Blockers

None.

## Next action

Commit VAC 1, then commit the parallel dependency resolution as VAC 2 without the timer.
