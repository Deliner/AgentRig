# State

## Focus

P193 (B200): the architecture rule parses each source once and resolves dependencies on several threads.

## Workspace

Branch: feature/faster-rules
Revision: ef4d6cb168b38e8ee1174774add857be2871d58f

## Progress

VAC 1 saved B200 and selected it as P193. VAC 2 lets `Resolvers::dependencies` in
structural_lint/architecture/contract_check/analysis.rs resolve the references of each file through the shared
`in_parallel` and put the parse issues of each file before its resolution gaps, as before. The temporary per-rule
timer in rule_evaluation stays unstaged.

## Verification

Release candidate on AgentRig, `just candidate lint`, warm runs of the architecture rule: before on `ca6d64a` 298 and
305 ms; with parallel dependency resolution 243, 236 and 248 ms. Candidate lint reports only the timer.

## Blockers

None.

## Next action

Commit VAC 2, then let the rule parse each Rust source once as VAC 3.
