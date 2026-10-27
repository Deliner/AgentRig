# State

## Focus

On 2026-10-27 the user deleted unneeded Backlog cards and selected the rest
for execution: [Plan](Plan.md) orders B001 → B003 → B002 → B008 → B009 →
B014. B001, B003 and B002 are complete as [P018](Archive/Plan/018.md),
[P019](Archive/Plan/019.md) and [P021](Archive/Plan/021.md). P025 (comment
references, [B008](Backlog/008.md)) is active. References to the removed
review runner are assessed by the executor at acceptance
([D033](Decisions/033.md)).

## Workspace

Branch: feature/target-state

Revision: 8b8ce49a4fc77926b8cc375fe85f0e5093c231e8

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
c1dde2c (D047 hook coordination), installed by `just bootstrap` under
.cache/development. Change the pin and run bootstrap in one shell command:
a pin without its install makes every hook fail. A stray branch
feature/init-probe at a14f284 remains from an install probe; guard-reference
refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `PATH=/Users/admin/.cargo/bin:...` inside `just write --`.

## Progress

P021 delivered [D047](Decisions/047.md) hook coordination, its repository
routes, grouped lint diagnostics, the merge-boundary review text and the
`just run` read_only classification; K1–K15 evidence is in its archive card.
P025 starts from [D037](Decisions/037.md)'s existing comment-reference lint
rule for Rust and Python. Its first VAC moves the exact reference format to
shared/syntax, where lint and memory-check share it; memory-check now reports
each reference in the configured sources that names no recorded decision or
invariant (K2). The language tests add tool-directive comments (K1).

## Verification

The current VAC passed the project_memory and language Python tests (92),
the language Rust tests and memory-check on this repository with the
candidate. 8b8ce49 passed the commit gate.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the K1/K2 VAC, then check the comment finding's text (K3), the
replace-comment skill (K4, K5) and add CLI evals through the lint and
memory-check commands (K6).
