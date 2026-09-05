# Portable scaffold runtime

The portable entry point is `worker.toml` in the selected project root. Initial distribution targets Linux. The runtime is a Rust binary; consumer projects need their own configured tools, Git, and bubblewrap for read-only commands. Just is a thin optional command interface. The consumer does not compile the worker or run the worker repository's tests.

A distributor builds the pinned crate with `cargo build --release --locked --manifest-path tooling/worker/Cargo.toml` and supplies the resulting `discipline-worker` executable. `init` copies its running executable and bundled assets into a consumer, pinning the package version in the generated configuration. See [independent examples](examples/README.md) for complete bootstrap commands. Updates and automatic migrations are outside this version.

## Commands

All project commands accept `--root PATH`; otherwise the current directory is the root. `--version` and `--help` do not need a project.

| Command | Result |
| --- | --- |
| `init` | Create standard config, memory, skills, binary and hook adapters; reject collisions before writing. Options select language, source, memory, skills, base branch and branch prefix. |
| `config-check` | Validate schema, cross-references, skills and lint applicability without analyzing source contents. |
| `doctor` | Diagnose the installed runtime, configured executables, sandbox and hooks. |
| `commands` / `run NAME -- ARGS` | List or execute the shared catalog. Arguments remain argv elements. |
| `report` | Summarize command calls, failures and elapsed time from the configured runtime directory. |
| `lint` / `lint-config-check` / `lint-rules` | Analyze sources, validate only lint settings, or list actual rule/language capabilities. |
| `check` / `check --staged` | Sequential gate over the working tree or exported index, including config and skills from that tree. |
| `memory-check` | Check the four memory files, links, decision history and executable invariant targets. |
| `resume` | Return State, Plan and observed Git branch/revision/status as JSON without changing the project. |
| `feature-start NAME` / `feature-merge` | Create or integrate a branch according to configured base/prefix. Integration runs the gate and retains the branch. |
| `hook` | Read an agent event as JSON from stdin and emit guidance or denial. |

Warnings do not fail lint; blocking findings exit 1 and configuration failures exit 2. External checks retain their process exit codes; a check configured with `warning = true` can report a nonzero exit without failing the gate. Interruptions still stop it. External output is preserved and failures identify the configured repair skill.

## Configuration ownership

`worker.toml` has schema `version = 1` and an exact `runtime` package version. Unknown fields are errors. Project-relative filesystem paths cannot escape the root. Source and check selectors are globs. The generated file is a complete editable example.

- `paths`: source selectors and locations of memory, skills, lint configuration and transient runtime data.
- `git`: base branch and working-branch prefix.
- `commands.NAME`: `argv`, `cwd` (default `.`), `accepts_args` and `read_only`. An empty argv requires forwarded arguments. Shell evaluation happens only if the catalog explicitly invokes a shell.
- `checks`: ordered IDs, `kind` (`command`, `lint`, `memory`), optional command reference, `include`, repair `skill`, and `warning`. A check with no matching files is skipped.
- `hooks`: file-to-skill routes, optional reminder JSON and the corresponding discipline skill. The hook and runner use the same command catalog.
- `oracles.ID`: command check, runner (`pytest` or `cargo`) and exact test target. Discovery uses that command's cwd, sandbox and shared process execution. Pytest targets are relative to the command cwd; keep its collection root aligned (for example, configure --rootdir . when a nested pytest config changes that root).

Lint keeps its existing TOML schema and compiled Rust/Python handlers; [rule semantics](README.md#rules-and-languages) describe counting, selectors and parser limits. The installed template supplies all five rules and focused repair skills. Unsupported selected languages are configuration errors, not silently ignored files. Numeric warning/error limits and named-condition severity are editable project policy.

Read-only commands require functioning Linux bubblewrap. Worker does not fall back to unrestricted execution when isolation is unavailable. Command records are append-only JSONL in `paths.runtime`; they describe process results, not task lifecycle. Staged checks run in a disposable exported tree, so their transient logs are disposable too. If you relocate `paths.runtime`, add the new directory to the consumer Git ignore rules; initialization supplies an ignore rule for the default location.

## Memory and recovery

The initialized indexes are `Plan.md`, `Decisions.md` and `Invariants.md`; details use stable numeric IDs and matching paths such as `Plan/001.md`. `State.md` is a compact snapshot with Focus, Workspace, Progress, Verification, Blockers and Next action sections.

Plan rows contain ID, Status, Depends on, Feature and User capability. Status is pending, active, paused or complete; dependencies must exist and be acyclic, with completed prerequisites for active/complete features. Details contain Feature, User capability and Acceptance; paused/complete details also contain Delivery. At most one feature is active.

Decision rows contain ID, Decision and Applies in. Details contain Context, Chosen, Rejected, Rationale and Consequences. Supported source application links need a real `DECISION: DNNN` comment. Committed identities, statements and detail contents are preserved; superseding a choice adds a new decision. Application links describe current owners and can move during refactoring; Git retains previous ownership. Rust, Python and shell application markers are parsed as comments.

Invariant rows contain ID, Invariant and Enforced by. Details contain Predicate and Oracle. The enforcement link names a function in a Rust/Python source file, with its `INVARIANT: INNN` comment before the function (attributes/decorators may intervene). Its configured oracle must name that function, including its class/module scope, and be discoverable by the actual test runner. Multiple declarations of that qualified name are ambiguous to syntax-only linking and are reported rather than treated as one marked target. Unsupported source languages are reported explicitly.

`resume` compares explicit `Branch: ` and `Revision: ` claims, optionally wrapped in backticks, with Git. It reports current, stale or unverified snapshot status. It does not infer completed acceptance, select a new feature, or overwrite State. The agent reconciles the snapshot with live evidence and current instructions.

The four editing skills and one route handler provide pre-edit guidance. Complexity reminders retain session/compaction accounting and retry behavior using the configured schedule. The worker repository uses the same runtime; its worker.toml supplies repository-specific paths, commands and oracles. Canonical skills live in assets/skills, are embedded at build time, and are exposed to this repository through .agents/skills.
