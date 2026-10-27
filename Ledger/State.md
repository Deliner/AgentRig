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

Revision: 7955912785dd8490a34491ca66599cb48d9fcaf7

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
invariant (K2). The language tests add tool-directive comments (K1)
(7955912). The current VAC adds a lint CLI eval of comment forms, data
text, grouped guidance and the fixed variant (K1, K3, K6) and makes
replace-comment use norm-or-choice and allow one record for several
comments (K4).

test_delivery.py (the I002 oracle) fails since P021: its hook scenario
expects `just check` to pass, but the D047 init route now stops the first
VAC check entry for edit-state. The impact map selects that test only for
its own changes, so no gate ran it.

## Verification

The current VAC passed the language delivery tests (4). 7955912 passed
the commit gate. test_delivery.py fails for both layouts as described.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Restore the I002 oracle for the VAC check stop and map Project changes to
test_delivery.py, then archive P025 with K1–K6 evidence and start P026.
