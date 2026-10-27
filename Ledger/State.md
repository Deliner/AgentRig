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

Revision: 487dd9cb4bd9082ee8445f4d3adf2638f05e26fa

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
487dd9c (D047 hook coordination), installed by `just bootstrap` under
.cache/development. Change the pin and run bootstrap in one shell command:
a pin without its install makes every hook fail. A stray branch
feature/init-probe at a14f284 remains from an install probe; guard-reference
refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `PATH=/Users/admin/.cargo/bin:...` inside `just write --`.

## Progress

P021's first VAC implements [D047](Decisions/047.md): one ordered hook
response per event, routes with events (edit, shell, check) and optional
block, per-session delivery of each skill and area, adjacency validation of
route order, and an init route reminding edit-state at the VAC check entry
(487dd9c). The pin moves to 487dd9c and the repository routes now stop the
first code edit per directory for refactor-large-directory and the first VAC
commit per context for edit-state.

## Verification

487dd9c passed the full commit gate, including 10 agent_hooks Rust tests
and the Python hook, reminder, setup and memory guidance tests.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the pin and repository routes, then group lint diagnostics by cause
(K8), state the merge review criteria (K9, K10, K13) and run consumer
sessions in Claude Code and Codex (K12).
