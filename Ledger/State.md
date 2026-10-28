# State

## Focus

On 2026-10-28 the user asked to hide lint warnings by default and show them
only under a separate flag, leaving only blocking errors in normal output.
This is recorded directly as [D049](Decisions/049.md); the selected Backlog
set is complete and the [Plan](Plan.md) is empty.

## Workspace

Branch: feature/lint-warnings-flag

Revision: 1555651ada244a0d5130f34f3725862aa84d5485

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
937d3f0, installed by bootstrap under .cache/development. Change the pin and
run bootstrap in one shell command: a pin without its install makes every
hook fail. A stray branch feature/init-probe at a14f284 remains from an
install probe; guard-reference refuses its deletion. `just read`/`just
write` run only the programs in .agentrig/commands.yaml. Agent shells lack
cargo on PATH, so commits run through `just scratch -- 'export
PATH=/Users/admin/.cargo/bin:...; git commit ...'`.

## Progress

`lint` drops warning findings from text and JSON output unless `--warnings`
is given; the gate's lint check uses the default. Exit codes and thresholds
are unchanged. The rule-test helper requests warnings; the guides describe
the flag.

## Verification

On the uncommitted change the candidate passed Clippy and 217 Python tests
(structural_lint, setup lint defaults, gate). On this repository default
lint printed nothing and exited 0; `--warnings` printed 2 warning groups and
exited 0.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Commit the change through the gate, run `just feature-merge` through
scratch and report the results to the user.
