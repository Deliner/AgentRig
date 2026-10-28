# State

## Focus

On 2026-10-28 the user directed that directory architecture.yaml contracts
exist only under Project and nowhere else. Delivered as
[D052](Decisions/052.md): the lint architecture rule now selects only Project,
and every architecture.yaml outside Project was removed. The [Plan](Plan.md) is
empty; this is explicit maintenance, not a new product outcome.

## Workspace

Branch: feature/architecture-scope

Revision: 3cc7ba399112aaac6775609030cae02b708c9879

The machine is macOS (Darwin 25.5.0). stable.txt pins this head, installed by
bootstrap under .cache/development: this branch changes Project (skill bodies),
so [D051](Decisions/051.md) pinned and bootstrapped its gate-accepted head
before feature-merge. `just read`/`just write` run only the programs in
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

This head passed the full commit gate (build, lint, memory, 77 Rust and 218
Python tests). Lint reports no missing-contract findings now that only Project
is analyzed, and memory-check resolves the migrated links with no WARNING.
Bootstrap built and installed it.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the pin, run `just feature-merge` and report to the user.
