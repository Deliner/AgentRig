# Portable scaffold runtime

`agentrig init` installs the runtime into a consumer project configured by the `.agentrig/` modules. Consumers need
Git, Just, their own configured tools and a read-only sandbox (bubblewrap on Linux, sandbox-exec on macOS); they do not
compile the worker or run its tests; see the [independent examples](../shared/testing/examples/README.md). A distributor
builds both executables with `cargo +1.98.1 build --release --locked --manifest-path Project/Cargo.toml`.

## Installation

`agentrig init --root CONSUMER` copies its executable and bundled assets, pins the package version in the
configuration, rejects collisions before writing, creates a Git repository on the base branch when none exists and
registers the Git hooks through `core.hooksPath`. Options: `--language python|rust`, `--frontend codex|claude-code`,
`--source` (default `Project`), `--memory` (default `Ledger`), `--skills`, `--service`, `--base` and `--prefix`. There
is no update or rollback: a changed release is installed into a fresh project. Rust projects also receive a `clippy`
command and check that runs the Clippy profile of the AgentRig development gate on the project's own toolchain.

`paths.service` (default `.agentrig`) holds the runtimes, Git hooks, skills, lint settings, reminders and runtime data;
the configuration modules always stay in `.agentrig`. The root `justfile` imports the recipes under `just/`, which
forward to the launcher. `frontend: codex` registers `.codex/hooks.json` and `.codex/config.toml`; `claude-code`
registers `.claude/settings.json` and a `CLAUDE.md` that imports `AGENTS.md`.

`bin/agentrig` runs the committed runtime `bin/PLATFORM/agentrig` for `uname -sm`, such as `macos-arm64` or
`linux-x86_64`. Without one, hooks and recipes name the platform and `agentrig runtime-add`, to run in the project root
with an AgentRig built for it. The first merge that brings a platform to the base runs on a platform the base has.

`init --interactive` asks for each option and the checks; Enter accepts defaults, and `cancel`, EOF or declining leaves
the target untouched. It shows the configuration and files, rechecks collisions, installs and runs doctor.

## Commands

Every command accepts `--root PATH`, by default the current directory.

| Command | Result |
| --- | --- |
| `init [--interactive]` | Create configuration, memory, skills, runtime and adapters without overwriting files. |
| `runtime-add` | Install the launcher and this executable as this platform's runtime; converts a single binary. |
| `config-check` / `doctor` | Validate configuration and affected maps; diagnose runtime, executables, sandbox, hooks. |
| `commands` / `run NAME -- ARGS` | List the tool registry or execute a command; arguments stay argv elements. |
| `scratch -- TEXT` | Run temporary shell code as a branch task job; `$AGENTRIG_SCRATCH` is its task area. |
| `report` | Summarize command timing, latest check evidence and repeated failures. |
| `lint` / `lint-config-check` / `lint-rules` | Analyze sources, validate lint settings, list rules. |
| `lint-baseline` | Record or lower the accepted error counts of ratchet mode. |
| `check [--staged \| --revision REV] [--only ID]` | Run the [gate](../change_delivery/gate/README.md). |
| `memory-check` / `resume` | Check memory and its links; return State, Plan, Git and check freshness as JSON. |
| `feature-start NAME` / `feature-merge` | Create or integrate a feature branch; merge keeps the branch. |
| `jobs`, `job-start`, `job-status`, `job-logs`, `job-stop`, `job-cleanup` | Manage owned background runs. |
| `hook` | Read an agent event as JSON on stdin and emit guidance or denial. |

Lint shows warnings only with `--warnings`, and they never fail it; blocking findings exit 1 and configuration
failures exit 2. Diagnostics name the check, scope, cause, repair skill and a shell-quoted RERUN command.

## Configuration

Configuration is strict YAML in `.agentrig/`. The required `project.yaml` holds `version: 1`, the exact `runtime`
version, `config_skill`, `paths` (sources and the locations of service files, memory, skills, lint settings and runtime
data) and `capabilities.lint` (disabling lint requires removing lint checks). Each optional module holds one section as
its whole content; an absent module is an empty section, and `project.yaml` cannot hold one.

- `git.yaml`: base branch and feature-branch prefix.
- `commands.yaml`: per name, `purpose`, `doc`, `argv`, `cwd`, `accepts_args`, `read_only`, `lifetime`, `programs` and
  `output`. An empty `argv` (`read`, `write`) runs only listed programs, each limited to its `subcommands` and refusing
  `deny` argument prefixes. `output` holds `success` and `failure` views of lines containing a `matching` part plus the
  last `tail` lines; such a run prints only its view, exit status and log directory.
- `checks.yaml`: ordered IDs with `kind` (`command`, `lint`, `memory`, `tools`), command, `include`, `skill`, `warning`
  and [`affected`](../change_delivery/gate/README.md).
- `tools.yaml`: `recognize` globs besides files with a shebang or executable bit, permitted `locations`, `entries`
  started outside the catalog, and `removed` tools with `purpose`, `reason`, `path` and the `revision` holding them. The
  `tools` check reports misplaced, unregistered or undescribed tools and stale references before merge, not on commits.
- `hooks.yaml`: ordered skill routes with paths, events (`edit`, `shell`, `check`) and optional `block`, the
  reminder schedule and discipline skill. A block stops the first event per skill and area in a context window.
- `oracles.yaml`: per invariant, command check, runner (`pytest` or `cargo`) and exact test target.

Unknown fields are errors naming their module; project-relative paths cannot escape the root. Lint settings follow the
[worker guide](../README.md#lint).

## Delivery and memory

`feature-start` requires the base branch, a clean tree and no pending merge or rebase. `feature-merge` rebases a
divergent feature, requires a passing gate on unchanged inputs and merges into the base with an explicit merge commit,
then empties the branch scratch area except `results/` and deletes its scratch run logs.
The pre-commit hook runs `guard-commit` and `check --staged`; the latest gate attempt is stored in
`paths.runtime/checks.json`.

Memory holds Plan, Backlog, Decisions, Invariants and State. Decision links to Rust, Python or shell owners need a
`DECISION: DNNN` comment; committed decisions are immutable and superseded by new ones. Invariants link a test carrying
`INVARIANT: INNN` and a configured oracle. `resume` compares State's Branch and Revision with Git and never overwrites
State.
