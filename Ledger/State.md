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

Revision: bbedc1a208ef3555a2da8daf27b13eba872e5dc0

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
bbedc1a, installed by bootstrap under .cache/development. Change the pin and
run bootstrap in one shell command: a pin without its install makes every
hook fail. A stray branch feature/init-probe at a14f284 remains from an
install probe; guard-reference refuses its deletion. `just read`/`just
write` now run only the programs in .agentrig/commands.yaml. Agent shells
lack cargo on PATH, so commits run through `just scratch -- 'export
PATH=/Users/admin/.cargo/bin:...; git commit ...'`.

## Progress

P027 delivered K3's base and temporary routes: 77860e8 restricts read and
write to configured programs, subcommands and denied argument forms
(optional field, because the previous runtime rejected it); bbedc1a adds
`scratch -- TEXT` with a per-branch task area. The pin to bbedc1a and this
repository's programs were installed together. Still open in P027: require
programs for empty argv, K9 compact output, K1/K5 tool placement lint, K2
registry listing, K4 skill, K6 task-completion cleanup, K7/K8 removal.

## Verification

bbedc1a passed the commit gate (77 Rust, 186 Python tests). After
bootstrap, `just config-check` accepted this repository's configuration
and `just read -- git status --short` ran through the new base rules.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Require programs for commands with empty argv (validation and its CLI
assertion), commit through scratch, then continue P027 with K9 compact
command output.
