# State

## Focus

On 2026-10-27 the user deleted unneeded Backlog cards and selected the rest
for execution: [Plan](Plan.md) orders B001 → B003 → B002 → B008 → B009 →
B014. B001, B003, B002 and B008 are complete as [P018](Archive/Plan/018.md),
[P019](Archive/Plan/019.md), [P021](Archive/Plan/021.md) and
[P025](Archive/Plan/025.md). P026 (condition naming, [B009](Backlog/009.md))
is active. References to the removed review runner are assessed by the
executor at acceptance ([D033](Decisions/033.md)).

## Workspace

Branch: feature/target-state

Revision: b4511b9936ae0e0b0eb64772e16b152895489c72

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
c1dde2c (D047 hook coordination), installed by `just bootstrap` under
.cache/development. Change the pin and run bootstrap in one shell command:
a pin without its install makes every hook fail. A stray branch
feature/init-probe at a14f284 remains from an install probe; guard-reference
refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `PATH=/Users/admin/.cargo/bin:...` inside `just write --`.

## Progress

P021 and P025 are archived with per-criterion evidence. P025 shared the
exact reference format between lint and memory-check, made memory-check
report references naming no record, added CLI evals and refined
replace-comment. During P025 the I002 delivery oracle, broken since P021's
D047 init route, was restored and is now selected for every Project change
(6032647). Archiving P025 gave Ledger/Archive/Plan sixteen entries;
[D048](Decisions/048.md) applies D029's flat-registry size exemption to that
exact directory. P026 changes the existing name-if-condition skill; its K5
requires a fixed set of real conditions and criteria before evaluation.

## Verification

6032647 passed the commit gate with test_delivery.py selected (260 Python
tests). The archive VAC changes only Ledger records.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Run the K5 evaluation fixed in P026's Delivery: build blind materials for
the seven cases at b4511b9, have a clean agent apply name-if-condition,
review each result against B009 K1–K4 and correct gaps in the skill.
