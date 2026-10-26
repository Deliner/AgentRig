# State

## Focus

Bring the repository toward the user's remembered target state recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26". First-level features sit directly in Project under two-word
names ([D042](Decisions/042.md)), awaiting the user's review. The user then
stated that contracts hold only purpose, files and directories, without
listing themselves ([D043](Decisions/043.md)), and recalled architecture types
for leaf features (hexagonal, clean, onion), recorded in Requests.md but not
specified. The full B003 revision draft is not agreed.

## Workspace

Branch: feature/target-state

Revision: 078873c1c9713ba1c48157bc3400cdb2c0b70224

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
af4beb1, installed by `just bootstrap` under .cache/development. A stray
branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `env PATH=/Users/admin/.cargo/bin:/Users/admin/.local/bin:...`.

## Progress

Committed on this branch: removal [D033](Decisions/033.md), macOS port
[D034](Decisions/034.md), affected-only merge tests [D035](Decisions/035.md),
Markdown and comment rules [D036](Decisions/036.md)/[D037](Decisions/037.md),
.agentrig modules [D038](Decisions/038.md), Just groups
[D039](Decisions/039.md), Ledger/Project init defaults
[D040](Decisions/040.md), the worker under Project
[D041](Decisions/041.md), pinned as af4beb1, and first-level features in
Project [D042](Decisions/042.md) (078873c).

This VAC applies D043: the architecture rule drops allow, deny and public
with their forbidden-dependency and private-access findings, and no longer
expects a contract to list itself. All 99 contracts were stripped; resolver
tests now prove a measured edge by closing a cycle. Plan P018–P023
reconciliation remains open.

## Verification

The D042 commit gate passed (68 Rust tests, 355 selected pytest, all static
checks). For D043, 36 architecture Rust tests and 104 architecture pytest
cases pass against the candidate; the commit gate verifies the whole change.

## Blockers

The installed hook validates .agentrig/lint.yaml on every event, so rules
of a new kind can be configured only after the runtime that knows the kind is
pinned and installed. Otherwise the hook rejects every edit, including the
revert (B001). memory-check cannot run inside `just read`.

## Next action

Ask the user what a leaf feature's architecture type must determine and which
types exist, then draft it in chat under D032. The rest of the feature scheme
and the B003 draft also await agreement before Backlog, Plan or further
structure changes. Independent explicit work meanwhile: faster tests (Python
to Rust, coverage).
