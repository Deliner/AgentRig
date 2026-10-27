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

Revision: 288f2fd8e4bda64e851c6864ae29a4980118401e

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
(487dd9c). The repository routes stop the first code edit per directory for
refactor-large-directory and the first VAC commit per context for edit-state
(8464a42). Text lint groups findings by shared cause (d73d241), and
execute-plan-feature states the merge-boundary review (288f2fd). Consumer
sessions in scratch Codex and Claude Code projects showed blocking, retries,
merged guidance and the commit reminder (K12). They also showed that
`just run read` received opaque-write guidance; the current VAC classifies
`just run` by its catalog command's read_only flag.

## Verification

The current VAC passed all 11 agent_hooks Rust tests, including the new
read-only classification test. Earlier VACs passed the full commit gate.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the classification fix, pin it with bootstrap in one command, then
archive P021 with K1–K15 evidence and activate P025 (B008).
