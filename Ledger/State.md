# State

## Focus

On 2026-10-28 the user directed that directory architecture.yaml contracts
exist only under Project and nowhere else. Delivered as
[D052](Decisions/052.md): the lint architecture rule now selects only Project,
and every architecture.yaml outside Project was removed. The [Plan](Plan.md) is
empty; this is explicit maintenance, not a new product outcome.

## Workspace

Branch: feature/architecture-scope

Revision: 17f06e7161711adce8c5b7e429f5d48f104fbeab

The machine is macOS (Darwin 25.5.0). stable.txt still pins 7099bb4; this branch
changes Project (skill bodies), so under [D051](Decisions/051.md) its
gate-accepted head is pinned and bootstrapped in one command before
feature-merge. `just read`/`just write` run only the programs in
.agentrig/commands.yaml; agent shells lack cargo on PATH, so commits run through
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:...; git commit ...'`.

## Progress

.agentrig/lint.yaml architecture rule include is now [Project, 'Project/**']
with the dead root cache excludes dropped. Removed 21 architecture.yaml outside
Project (tooling, Ledger, .agentrig, just, .agents, .claude, .codex, .github,
root). Decisions.md links for D038, D039, D041 and D042 migrated to surviving
owners; Plan.md, edit-plan and develop-tools no longer mention architecture
inventories outside Project. D052 recorded with precedence over D028.

## Verification

Pending: stage and pass the commit gate. Lint must report no missing-contract
findings now that only Project is analyzed, and memory must resolve the migrated
links without a WARNING.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Stage the change, pass the commit gate, pin and bootstrap the accepted head
(D051), then run `just feature-merge` and report to the user.
