# Portable scaffold runtime

`agentrig init` installs the runtime into a consumer project configured by the `.agentrig/` modules. Consumers need
Git, Just, their own configured tools and a read-only sandbox (bubblewrap on Linux, sandbox-exec on macOS); they do not
compile the worker or run its tests. A distributor builds both executables with
`cargo +1.98.1 build --release --locked --manifest-path Project/Cargo.toml`; see the
[independent examples](../shared/testing/examples/README.md).

## Installation

`agentrig init --root CONSUMER` copies its executable and bundled assets, pins the package version in the
configuration, rejects collisions before writing, creates a Git repository on the base branch when none exists and
registers the Git hooks through `core.hooksPath`. Options: `--language python|rust`, `--frontend codex|claude-code`,
`--source` (default `Project`), `--memory` (default `Ledger`), `--skills`, `--service`, `--base` and `--prefix`. There
is no update or rollback: a changed release is installed into a fresh project.

`paths.service` (default `.agentrig`) holds the binary, Git hooks, skills, lint settings, reminders and runtime data;
the configuration modules always stay in `.agentrig`. Adapters quote the service path; newlines and Just interpolation
(`{{`) are rejected. The root `justfile` only imports `just/commands.just`, `checks.just`, `delivery.just`,
`lint.just` and `jobs.just`, whose recipes forward to the installed binary. `frontend: codex` registers
`.codex/hooks.json` and `.codex/config.toml`; `claude-code` registers `.claude/settings.json` and a `CLAUDE.md` that
imports `AGENTS.md`.

`init --interactive` asks for the directory, language, service, source, memory and skills paths, branch naming, client
and checks; Enter accepts defaults, and `cancel`, EOF or declining the confirmation leaves the target untouched. It
prints the configuration and file list, rechecks collisions, installs and runs doctor.

## Commands

Every command accepts `--root PATH`, by default the current directory.

| Command | Result |
| --- | --- |
| `init [--interactive]` | Create configuration, memory, skills, binary and adapters without overwriting files. |
| `config-check` / `doctor` | Validate configuration; diagnose the runtime, executables, sandbox and hooks. |
| `commands` / `run NAME -- ARGS` | List or execute the catalog; arguments stay argv elements. |
| `report` | Summarize command timing, latest check evidence and repeated failures. |
| `lint` / `lint-config-check` / `lint-rules` | Analyze sources, validate lint settings, list rules. |
| `check [--staged \| --revision REV] [--only ID]` | Run the [gate](../change_delivery/gate/README.md). |
| `memory-check` / `resume` | Check memory and its links; return State, Plan, Git and check freshness as JSON. |
| `feature-start NAME` / `feature-merge` | Create or integrate a feature branch; merge keeps the branch. |
| `jobs`, `job-start`, `job-status`, `job-logs`, `job-stop`, `job-cleanup` | Manage owned background runs. |
| `hook` | Read an agent event as JSON on stdin and emit guidance or denial. |

Warnings do not fail lint; blocking findings exit 1 and configuration failures exit 2. Diagnostics name the check,
scope, cause, repair skill and a shell-quoted RERUN command.

## Configuration

Configuration is strict YAML in `.agentrig/`. The required `project.yaml` holds `version: 1`, the exact `runtime`
version, `config_skill`, `paths` (sources and the locations of service files, memory, skills, lint settings and runtime
data) and `capabilities.lint` (disabling lint requires removing lint checks). Each optional module holds one section as
its whole content; an absent module is an empty section, and `project.yaml` cannot hold one.

- `git.yaml`: base branch and feature-branch prefix.
- `commands.yaml`: per name, `argv`, `cwd`, `accepts_args`, `read_only` and `lifetime`.
- `checks.yaml`: ordered IDs with `kind` (`command`, `lint`, `memory`), command, `include`, `skill`, `warning` and
  [`affected`](../change_delivery/gate/README.md).
- `hooks.yaml`: file-to-skill routes, reminder schedule and discipline skill.
- `oracles.yaml`: per invariant, command check, runner (`pytest` or `cargo`) and exact test target.

Unknown fields are errors naming their module; project-relative paths cannot escape the root. Lint settings follow the
[worker guide](../README.md#lint).

## Delivery and memory

`feature-start` requires the base branch, a clean tree and no pending merge or rebase. `feature-merge` rebases a
divergent feature, requires a passing gate on unchanged inputs and merges into the base with an explicit merge commit.
The pre-commit hook runs `guard-commit` and `check --staged`; the latest gate attempt is stored in
`paths.runtime/checks.json`.

Memory holds Plan, Backlog, Decisions, Invariants and State. Decision links to Rust, Python or shell owners need a
`DECISION: DNNN` comment; committed decisions are immutable and superseded by new ones. Invariants link a test carrying
`INVARIANT: INNN` and a configured oracle. `resume` compares State's Branch and Revision with Git and never overwrites
State.
