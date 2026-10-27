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

Revision: 0233a93654d7fcd89e99859c34f15afb3311f236

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
52471a0 (D046 recovery), installed by `just bootstrap` under
.cache/development. Change the pin and run bootstrap in one shell command:
a pin without its install makes every hook fail. A stray branch
feature/init-probe at a14f284 remains from an install probe; guard-reference
refuses its deletion. Agent shells lack cargo and uv on PATH; prefix
`PATH=/Users/admin/.cargo/bin:...` inside `just write --`.

## Progress

P018 delivered hook configuration recovery ([D045](Decisions/045.md),
[D046](Decisions/046.md)) and is archived with K1–K8 evidence. P019 VACs:
shared glob patterns, skill references and parsing got their own owners
(e921f47); feature start and integration moved to change_delivery/git
(b9e4135); project_setup/setup became initialization (0233a93). Blind checks
then ran: a tree-only reader placed behaviors a–j with confidence 3–5; a
fresh Explore agent reached all ten through architecture.yaml contracts
(confidence 4–5) and reported stale contract text and two process.rs files.
The current VAC corrects those contracts and renames
command_runner/process.rs to sandbox.rs. Moving check evidence to a
first-level feature was tried and reverted: Project would exceed its hard
limit of 15 entries; its consumers outside project_memory already depend on
it, so the contracts now name check evidence as project memory.

## Verification

e921f47, b9e4135 and 0233a93 passed the full commit gate. For the current
VAC: clippy with -D warnings and candidate `just lint` without errors on the
working tree. K7 consumer sessions passed in Claude Code and Codex as
recorded in P018.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the contract and sandbox VAC, record B003 K1–K5 evidence in P019 and
archive it, then start P021 (B002).
