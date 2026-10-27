# State

## Focus

On 2026-10-27 the user deleted unneeded Backlog cards and selected the rest
for execution: [Plan](Plan.md) orders B001 → B003 → B002 → B008 → B009 →
B014. B001 is complete as [P018](Archive/Plan/018.md). P019 (project
structure that reflects its capabilities, [B003](Backlog/003.md)) is active.
References to the removed review runner are assessed by the executor at
acceptance ([D033](Decisions/033.md)).

## Workspace

Branch: feature/target-state

Revision: 1628d677f6a9d707e7626d52e9b3f831ae0c987a

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
52471a0 (D046 recovery), installed by `just bootstrap` under
.cache/development. Change the pin and run bootstrap in one shell command:
a pin without its install makes every hook fail. A stray branch
feature/init-probe at a14f284 remains from an install probe; guard-reference
refuses its deletion. Agent shells lack cargo and uv on PATH; prefix
`PATH=/Users/admin/.cargo/bin:...` inside `just write --`.

## Progress

P018 delivered hook configuration recovery ([D045](Decisions/045.md),
[D046](Decisions/046.md)) and is archived with K1–K8 evidence. The first
P019 VAC (B003 K3) gives shared behavior its own owners: glob patterns move to
shared/paths/patterns.rs, skill reference validation to
shared/diagnostics/skills.rs and source parsing to shared/syntax, so
agent_hooks, change_delivery and project_memory no longer reach into
structural_lint for them (e921f47). The next VAC moves feature start and
checked integration from shared/vcs into change_delivery/git/integration.rs,
their only consumer; shared/vcs keeps repository access with new
Repository run, command and ancestor methods. Open: unclear names and the
blind checks.

## Verification

e921f47 passed the full commit gate. For the workflow VAC: clippy with -D
warnings, candidate `just lint` without errors and
Project/change_delivery/git/tests (10) pass on the working tree. K7 consumer sessions passed in Claude Code and Codex as
recorded in P018.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the rename of project_setup/setup to project_setup/initialization,
then rerun the blind name and structure checks against B003 K1–K5.
project_memory/evidence stays: moving it to change_delivery would create a
cycle with resume, and it depends on project_config, so it cannot be shared.
