# Portable scaffold runtime

The portable entry point is the `.agentrig/` configuration in the selected project root. The runtime is a Rust binary;
consumer projects need their own configured tools, Git, and a read-only sandbox for read-only commands.
Just is a thin command interface. The consumer does not compile the worker or run its tests.

A distributor builds the pinned crate with
`cargo +1.98.1 build --release --locked --manifest-path tooling/worker/Cargo.toml` and supplies the
resulting `agentrig` and `agentrig-lint` executables. Both lint interfaces use the same engine; standalone
lint accepts an external root and policy without installing worker files there. See
[independent examples](examples/README.md) for complete bootstrap commands.

## Installation

`agentrig init --root CONSUMER` copies its running executable and bundled assets into the consumer and
pins the package version in the generated configuration. It rejects collisions before writing, creates a
Git repository on the configured base when none exists and registers the Git hooks through
`core.hooksPath`. Options select `--language python|rust`, `--frontend codex|claude-code`, `--source`,
`--memory`, `--skills`, `--service`, `--base` and `--prefix`. There is no separate setup, update or
rollback step: a changed release is installed into a fresh project.

`paths.service` defaults to `.agentrig` and holds the binary, Git hooks, skills, lint settings, reminders
and runtime data; individual paths remain configurable. The configuration modules always stay in `.agentrig`. Adapters quote the selected path, including
spaces and shell metacharacters. Newlines and Just interpolation syntax (`{{`) are rejected.

`frontend: codex` (default) registers `.codex/hooks.json` and `.codex/config.toml`; `frontend: claude-code`
registers `.claude/settings.json` and a `CLAUDE.md` that imports the shared `AGENTS.md`.

`agentrig init --interactive --root CONSUMER` asks for the project directory, language, service, source,
memory and skills paths, branch naming, client and checks. Enter accepts defaults. `cancel`, EOF or
declining the final confirmation leaves the target untouched. The wizard prints the resulting
configuration and file list, rechecks collisions after confirmation, installs and runs doctor.

## Commands

All project commands accept `--root PATH`; otherwise the current directory is the root.

| Command | Result |
| --- | --- |
| `init [--interactive]` | Create config, memory, skills, binary and hook adapters without overwriting files. |
| `config-check` | Validate schema, cross-references, skills and lint applicability. |
| `doctor` | Diagnose the installed runtime, configured executables, sandbox and hooks. |
| `commands` / `run NAME -- ARGS` | List or execute the shared catalog. Arguments remain argv elements. |
| `report` | Summarize command timing, latest check evidence and repeated check failures. |
| `lint` / `lint-config-check` / `lint-rules` | Analyze sources, validate lint settings, list rules. |
| `check [--staged \| --revision REV] [--only ID]` | Run the gate over the worktree, index or a revision. |
| `memory-check` | Check memory files, links, decision history and invariant oracles. |
| `resume` | Return State, Plan, Git observation and check freshness as JSON. |
| `feature-start NAME` / `feature-merge` | Create or integrate a feature branch; merge retains the branch. |
| `jobs`, `job-start`, `job-status`, `job-logs`, `job-stop`, `job-cleanup` | Manage owned background runs. |
| `hook` | Read an agent event as JSON from stdin and emit guidance or denial. |

Warnings do not fail lint; blocking findings exit 1 and configuration failures exit 2. Failure diagnostics
identify the check, scope, cause, configured repair skill and a shell-quoted RERUN command.

## Configuration

Configuration is strict YAML in `.agentrig/`; there is no `agentrig.yaml`. The required `project.yaml`
has schema `version: 1`, an exact `runtime` version, `config_skill`, `paths` (source selectors and
locations of service files, memory, skills, lint configuration and runtime data) and `capabilities.lint`
(disabling lint requires removing lint checks from the gate). Each optional module holds one section as
its whole content; an absent module means an empty section, and `project.yaml` cannot hold one.

- `git.yaml`: base branch and feature-branch prefix.
- `commands.yaml`: per name, `argv`, `cwd`, `accepts_args`, `read_only` and `lifetime`.
- `checks.yaml`: ordered IDs, `kind` (`command`, `lint`, `memory`), command, `include`, `skill`, `warning`.
- `hooks.yaml`: file-to-skill routes, reminder schedule and discipline skill.
- `oracles.yaml`: per ID, command check, runner (`pytest` or `cargo`) and exact test target.

Unknown fields are errors naming their module, and project-relative paths cannot escape the root.
Lint uses strict YAML and compiled Rust/Python handlers; see [rule semantics](README.md#lint).

## Delivery and memory

`feature-start` requires the base branch, a clean tree and no pending merge or rebase. `feature-merge`
rebases a divergent feature, requires a passing gate with tests affected by the branch on unchanged
inputs and merges into the base with an explicit merge commit. The pre-commit hook runs `guard-commit` and `check --staged`; checks
select [affected tests](CHECKS.md). The latest gate attempt is stored in `paths.runtime/checks.json`.

Memory holds Plan, Backlog, Decisions, Invariants and State. Decision links to Rust, Python or shell
owners need a `DECISION: DNNN` comment; committed decisions are immutable and superseded by new ones.
Invariants link a test carrying `INVARIANT: INNN` and a configured oracle. `resume` compares State's
Branch and Revision claims with Git and never overwrites State.
