# State

## Focus

On 2026-10-27 the user deleted unneeded Backlog cards and selected the rest
for execution: [Plan](Plan.md) orders B001 → B003 → B002 → B008 → B009 →
B014. B001, B003, B002, B008 and B009 are complete as
[P018](Archive/Plan/018.md), [P019](Archive/Plan/019.md),
[P021](Archive/Plan/021.md), [P025](Archive/Plan/025.md) and
[P026](Archive/Plan/026.md). P027 (tool registry, [B014](Backlog/014.md)) is
the remaining card and is active. References to the removed review runner are
assessed by the executor at acceptance ([D033](Decisions/033.md)).

## Workspace

Branch: feature/target-state

Revision: 3cced20c905a58ba381d31e5d843eedb30cafecc

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
bbedc1a, installed by bootstrap under .cache/development. Change the pin and
run bootstrap in one shell command: a pin without its install makes every
hook fail. A stray branch feature/init-probe at a14f284 remains from an
install probe; guard-reference refuses its deletion. `just read`/`just
write` now run only the programs in .agentrig/commands.yaml. Agent shells
lack cargo on PATH, so commits run through `just scratch -- 'export
PATH=/Users/admin/.cargo/bin:...; git commit ...'`.

## Progress

P027 delivered K3 (77860e8, bbedc1a, 4d039cc: configured base programs,
`scratch -- TEXT`, programs required for empty argv) and K9 (3cced20:
configured success/failure output views with the log path). The current VAC
adds the candidate's remaining criteria: catalog `purpose`/`doc`, the
`.agentrig/tools.yaml` module (recognize, locations, entries, removed), the
registry listing in `commands`, a `tools` gate check that the staged VAC
commit skips, the develop-tools skill and merge cleanup of the branch
scratch area. This repository's configuration does not use them yet: the
installed bbedc1a rejects the new fields.

## Verification

Before this VAC commit: Clippy and rustfmt clean, candidate lint without
errors, 77 Rust lib tests and 246 Python tests of the command, delivery,
setup, config, hooks, memory and launcher groups passed.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the VAC through scratch. Then pin it in one scratch command
(stable.txt = HEAD, purposes and output views in commands.yaml, the tools
check in checks.yaml, .agentrig/tools.yaml, bootstrap), commit the pin and
configuration, run the tools check, record K1–K9 evidence in P027 and
archive it.
