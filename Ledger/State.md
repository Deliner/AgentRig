# State

## Focus

On 2026-10-27 the user deleted unneeded Backlog cards and selected the rest
for execution: [Plan](Plan.md) orders B001 → B003 → B002 → B008 → B009 →
B014. B001 and B003 are complete as [P018](Archive/Plan/018.md) and
[P019](Archive/Plan/019.md). P021 (hook coordination, [B002](Backlog/002.md))
is active. References to the removed review runner are assessed by the
executor at acceptance ([D033](Decisions/033.md)).

## Workspace

Branch: feature/target-state

Revision: 1898898b91ad1857691fc57d15d4c5ebc0c05042

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
52471a0 (D046 recovery), installed by `just bootstrap` under
.cache/development. Change the pin and run bootstrap in one shell command:
a pin without its install makes every hook fail. The installed runtime
rejects route `events` and `block` until the pin includes D047, so the
repository's .agentrig/hooks.yaml keeps the old route form until then. A
stray branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `PATH=/Users/admin/.cargo/bin:...` inside `just write --`.

## Progress

P021's first VAC implements [D047](Decisions/047.md): one ordered hook
response per event, routes with events (edit, shell, check) and optional
block, per-session delivery of each skill and area, adjacency validation of
route order, and an init route reminding edit-state at the VAC check entry.

## Verification

Before the commit: agent_hooks Rust tests (10) and Python tests for
agent_hooks, project_setup, plan and backlog guidance (127) passed with the
candidate.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the D047 VAC, pin it with bootstrap in one command, then add the
repository's blocking architecture and VAC check routes, group lint
diagnostics by cause (K8) and state the merge review criteria (K9, K10, K13).
