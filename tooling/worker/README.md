# Rust worker runtime and structural linter

The Rust runtime owns agent and Git hooks, command execution, feature integration, memory validation and structural lint. Configuration and gate stages come from the project worker.toml; see [the scaffold guide](SCAFFOLD.md) for their schema. Just and Git/Codex adapters only route calls into this runtime. Python is used for behavioral tests and the benchmark.

## Build and execution

Use just lint, just lint-rules, or tooling/worker/run through just write. The launcher builds a release binary with Cargo.lock and Rust 1.98.1, then executes it. Rustup, Cargo, rustfmt, Clippy, a native linker, Bash, Git, and flock must be available. First use needs toolchain/crate downloads; subsequent locked builds use the local cache. Install the pinned toolchain with rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy.

The default target is .cache/worker; WORKER_TARGET_DIR can place it outside a temporary staged tree. The launcher fingerprints source contents, compile-time reminder text, toolchain and dependency files, and serializes builds with flock. A source change, including a branch change or staged/working-tree difference, invalidates the cached binary. Build output goes to stderr so hook stdout remains JSON. No Python fallback executes when native compilation fails.

The hook registration invokes run hook with an explicit repository root. Ledger reminders remain context-only; existing command denials, complexity thresholds, retry allowance, session recovery guidance, SHA-256 state keys, file locks, and atomic state replacement are preserved. Full-refresh wording is a compile-time text asset. An existing session using an old registered command may require restart to use the native registration.

## Configuration

The lint file selected by worker.toml (or --config; standalone default lint.toml) uses TOML version 1 of the worker schema. Unknown fields, unsupported rule kinds/targets, invalid globs, duplicate IDs, missing skills, and invalid effective thresholds are errors. A configuration failure exits 2 and points at the configured repair skill; a structural error exits 1; warnings alone exit 0.

Validate independently with just lint-config-check. Use just lint-config-check --config path/to/lint.toml --json for another config and machine-readable diagnostics (an empty array means valid). Exit 0 means the configuration and current target selection are valid; exit 2 reports a configuration error and repair skill. This command checks TOML/schema, skills, selectors, supported targets/extensions and effective overrides against the current inventory, without reading or parsing source contents. It does not claim the source passes lint. Normal lint uses the same validation automatically.

Use just lint-rule function-lines for readable details, add --json for machine output or --example for a complete TOML configuration using the installed .agents/skills. Use just lint-rules to inspect each rule's target, languages, supported handler extensions and measurement. These are implementation capabilities, not user-editable claims. Language selection uses extensions; setting a suffix cannot create a handler.

Each rules entry requires:

- id: unique diagnostic identity.
- enabled: optional boolean, true by default. false skips target selection and execution; the entry still needs valid schema, capabilities and repair skills.
- kind: an implemented rule from just lint-rules.
- target: file or directory, compatible with that kind.
- include: nonempty repository-relative glob list.
- exclude: optional rule-local glob list.
- extensions: optional literal suffixes, such as .rs or .ts; only file rules accept them.
- Numeric rules require warning, error, or both: nonnegative thresholds; warning must be strictly below error when both exist. Equality passes; a greater value triggers that level. Omit error for warnings only, or warning for blocking only.
- named-if-condition instead requires level = "warning" or "error", with no numeric thresholds or threshold overrides.
- warning_skill and error_skill: existing repository SKILL.md paths with name/description frontmatter.
- overrides: optional ordered selector/threshold overrides.

Global exclude removes paths from the inventory before directory counts. Rule-local exclude only suppresses diagnostics on matching targets. Globs are case-sensitive, use / separators, and treat * as one path component and ** as recursive. Root-directory diagnostics use the path ".".

An override inherits the rule's target and repair skills. It supplies include, optional extensions, and at least one of warning/error. All matching overrides apply in declaration order; later supplied thresholds replace earlier values. Effective warning must remain below error when both exist. An omitted override threshold inherits its prior value; overrides cannot remove thresholds. Different rules are independent and can both report on a file.

Example, placed inside the file-size rule before the next rules entry:

~~~toml
[[rules.overrides]]
include = ["Project/Runtime/**"]
extensions = [".rs", ".ts"]
warning = 220
error = 400

[[rules.overrides]]
include = ["Project/Runtime/parser.rs"]
warning = 280
error = 450
~~~

Do not add exceptions merely to turn the gate green. Preserve a current requirement or record a justified policy change.

## Rules and languages

nonblank-lines supports UTF-8 text files of any language. It counts nonempty lines, including comments, and skips non-UTF-8 files. Default suffixes include Python, shell, Rust, JavaScript/TypeScript, Go, C/C++, C#, Java, and common documentation/configuration formats. Selectors can narrow or extend these suffixes without Rust changes; an empty extension list selects all files supported by the rule.

directory-entries counts immediate child names, including child directories, from the selected file inventory. It is independent of language and rejects extension selectors. Git inventories include tracked and non-ignored untracked files; exported staged trees use their physical files. Deleted files and symlinks are excluded. Empty directories are not represented in Git and are not counted.

Current defaults retain warnings above 300 nonblank lines and errors above 500; directory warnings above 10 and errors above 15. Ledger/Decisions and Ledger/Invariants remain excluded from directory-size checks. All structural diagnostics include rule ID, path, measurement, limit, severity, repair skill, and a shell-quoted rerun command. Use just lint --json for structured output.

Example syntax rule selection (inside its rules entry):

~~~toml
enabled = true
include = ["Project/**"]
exclude = ["Project/generated/**"]
extensions = [".rs", ".py"]
~~~

Setting extensions = [".sh"] on function-lines fails with an error naming supported Rust/Python handlers. The same applies to numeric override selectors. File-size remains language independent, so it can select shell scripts.

### Syntax-aware rules

The Rust engine parses each selected source once per run using Tree-sitter. Compiled handlers in src/lint/languages interpret Rust (.rs) and Python (.py, .pyi) syntax and produce measurements for the same rules. Empty extensions applies the include/exclude selection without an implicit language filter: any selected file lacking a handler is a configuration error. In mixed directories, explicitly set extensions or exclude unsupported files. An unsupported literal suffix in include (such as scripts/*.sh) or extensions is rejected even if no files exist yet. Other glob forms are checked against the current inventory; validation does not predict future files. Other file types remain available to language-independent rules. Files with parse errors block with a source location and repair skill, even when rule findings are warnings. This is syntax analysis, not type checking or name resolution.

- named-if-condition accepts one identifier or named field path, optionally parenthesized. Calls, comparisons, negations, boolean operators, indexing and literals must be assigned a meaningful name before branching. Rust else-if and Python elif, conditional expressions and comprehension filters are included. A simple Rust if let is a binding pattern and remains allowed; let chains are reported. The linter cannot prove that a Python name contains bool or that its name explains the branch.
- function-lines counts nonblank lines from the function signature through the end of its body, including comments, docstrings and nested definitions. Decorators and preceding attributes are excluded. Methods, async functions, constructors, nested functions and anonymous closures/lambdas are included; declarations without bodies have no size finding. Nested functions are also measured independently.
- parameter-count counts declared inputs, including optional and variadic parameters as one each. Generic type parameters, commas inside types/defaults, and separators do not count. Rust self receivers (including typed self) and the first bound Python method receiver are excluded. For literal @staticmethod decorators, inputs all count; bound method/classmethod receivers do not. Decorator aliases are not resolved. Python constructors are checked through explicit __init__/__new__ declarations; Rust associated constructor functions such as new are ordinary functions. Calls, class inheritance arguments, generated constructors (such as dataclass) and macro-expanded code are not inferred.

Repository policy and newly installed defaults block all three language rules: named-if-condition uses level = "error", function-lines uses error = 40, and parameter-count uses error = 4. Equality passes. Production code, tests and the benchmark follow these limits without new exclusions. D021 supersedes the initial advisory rollout; existing consumer configurations require an explicit policy update. File/directory thresholds and configurable warning capabilities remain unchanged.

Syntax diagnostics add a 1-based line and symbol to the existing JSON fields; text output renders file:line (symbol). Each finding points to name-if-condition, refactor-long-function or reduce-parameters. No automatic code transformation is performed.

The registry in src/lint/rules.rs declares supported kinds, targets and actual handlers. To add a rule, implement its measurements, validation, behavioral tests and repair skill. To support another language, add its grammar and handler, update supported extensions and registry metadata together, and test its syntax against the existing rule semantics. The extension point is compiled Rust code; there is no dynamic plugin lifecycle. Rust macros are opaque token trees, and Python decorator aliases or generated declarations require semantic tooling beyond these handlers.

Parser APIs and grammars: [Tree-sitter](https://docs.rs/tree-sitter/0.26.13/tree_sitter/), [Rust grammar](https://docs.rs/tree-sitter-rust/0.24.2/tree_sitter_rust/), [Python grammar](https://docs.rs/tree-sitter-python/0.25.0/tree_sitter_python/).

## Commit and merge gate

The pre-commit hook checks the actual exported Git index, including its Rust sources, lint config, and skill files. Structural lint runs before the other checks. The same gate runs on the integration candidate before merge and after a required rebase; errors stop the operation, warnings do not.

Each checks entry in worker.toml names its repair skill. The shared gate preserves original tool output and reports that skill, the selected scope and an executable retry command on failure; structural findings use their rule-specific skills. `just check --only CHECK_ID` retries one stage without replacing the full commit/merge gates. See [recovery and check evidence](SCAFFOLD.md#memory-and-recovery) for resume freshness and repeated-failure feedback.

Native integration tests under tooling/tests/native execute the built binary. They assert native hook responses and state transitions directly and exercise configuration, selectors, thresholds, diagnostics and staged inventories. Existing Git branch/VAC tests use the native guards.

Configuration parsing uses the [TOML serde library](https://docs.rs/toml/latest/toml/); selectors follow [globset semantics](https://docs.rs/globset/latest/globset/). Builds use Cargo's [locked dependency mode](https://doc.rust-lang.org/cargo/commands/cargo-build.html).

## Measured latency

See [repeatable measurements](examples/LATENCY.md) and [their runner](examples/measure.py) for installed Python/Rust example results and measurement limits.

## Standalone assessment

The release build also produces `discipline-lint`, a native CLI over the same
engine used by worker checks. Copy that executable outside the project to be
assessed; no worker setup, memory, hooks or MCP are required there:

```sh
discipline-lint --root /projects/consumer --config /policies/lint.toml --json
discipline-lint lint-config-check --root /projects/consumer --config /policies/lint.toml
discipline-lint lint-rules
```

Source selectors always apply to the assessed root. By default, skill references
retain their existing project-relative semantics. An external policy can set
`skill_root = "skills"`, resolved relative to its configuration file, to use a
separate skill bundle. References must still identify valid SKILL.md files inside
that selected root. Diagnostics provide their resolved paths. The assessment
writes no project files; worker and standalone findings agree for the same
configuration apart from their executable-specific retry commands.


## Adding rules and language handlers

The typed Kind and its descriptor in src/lint/rules own rule identity, target,
metric, parameter policy, repair guidance and installed defaults. Catalog and
configuration validation consume this descriptor; package setup consumes its
examples. Do not maintain separate capability or default tables.

The registrations in src/lint/languages/registry.rs bind actual parser and
inspection functions to extensions and implemented kinds. These registrations
also drive catalog language support and selector validation. Add a language only
with its implemented measurements and positive/negative behavior tests. Bash
parsing for shell hooks does not register Bash as a lint language. Shared
measurements carry Kind, locations and values; the engine owns diagnostics.

For a new rule, add its typed identity, descriptor and measurement, register the
implemented language pairs when applicable, supply a repair skill and verify
accepted/rejected source and configuration examples. Numeric rules use warning
and error thresholds; policy rules use level. Existing configuration syntax and
strict installed defaults are preserved.


Use just lint-explain src/example.rs (or discipline-lint lint-explain with
--root and --config) to see every configured rule's selection reason. --json
returns the same information for automation. Selected entries include effective
warning/error or level and zero-based matched override indexes, in application
order. Disabled, excluded, extension-mismatched, wrong-target and absent inventory
entries explain why no check runs. It reads configuration and inventory, without
parsing source or running rule measurements. Invalid effective configuration
still fails. Relative paths are resolved from the selected project root.

## Managed command visibility

Catalog commands create a unique run record before execution under the configured
runtime/jobs directory. `just jobs`, `just job-status RUN_ID` and `just resume`
show the saved command and current OS observations. Identity combines PID, boot
ID and process start ticks; a saved PID alone never establishes liveness.
Leader CPU ticks and resident bytes are point-in-time measurements, not totals
for detached descendants. Completion records the exit code and launch errors.

Set WORKER_OWNER to a stable task/session identity when launching commands outside
an agent session. Otherwise CODEX_THREAD_ID, then CODEX_SESSION_ID is used; without
either, the run gets its own owner. WORKER_PARENT_RUN optionally associates a
child invocation. Each managed payload receives its owner's WORKER_OWNER and its
run ID as WORKER_PARENT_RUN, so nested worker commands retain their immediate
parent. Branch and project are captured independently of ownership.
An orphaned run has a live child but no live runner; an interrupted run has
neither. Inspect these states before deciding what to resume or clean up.

The shared process runner records raw stdout/stderr while forwarding output to
the caller. `just job-logs RUN_ID` returns the last 64 KiB of each stream as JSON,
with byte counts and truncation flags; complete bytes remain in stdout.log and
stderr.log beside the run record. Display replaces invalid UTF-8, stored logs do
not. `report` aggregates these lifecycle records; commands.jsonl is no longer
written or read. Old completion-only logs remain untouched as historical files.

`just job-start COMMAND -- ARGS` starts a configured command in a distinct
systemd user scope and returns its run_id. It requires Linux cgroup v2 and a
working systemd user manager. The process inherits the calling environment,
freezes the selected argv/cwd/read-only policy, and retains output in the same
run directory after the caller exits. Launcher errors appear separately in
job-logs. A delayed launch response does not authorize starting a duplicate.

`just job-stop RUN_ID` requires the recorded owner identity. It checks the scope
invocation ID, requests graceful stop, then forces remaining processes and
verifies the recursive cgroup populated state. This includes descendants that
created a new process session. Cancellation intent survives runner termination;
logs remain available. Lost access to the user manager reports unverified state,
not successful completion. Ownership is coordination within a user account,
not an access-control boundary against that same user's own programs.

Commands default to `lifetime = "task"`; set `lifetime = "shared"` in a command's
worker.toml table for a service that must outlive task cleanup. `just job-cleanup`
cleans the current owner's task scopes; `--branch BRANCH` narrows that selection.
Successful feature-merge invokes the same cleanup for the merged branch. Shared
services, other owners, other branches and the active cleanup caller are retained.
The owner can explicitly stop a shared service with job-stop. Cleanup failures
are reported after merge as a separate failure with a job-cleanup retry command.
Without an owner identity, merge does not guess which runs belong to its caller.

Setup/doctor reports whether a real transient user scope can be created, including
cgroup v2 availability and the backend diagnostic. Background support is optional
for projects using `[processes] foreground = "process-group"` (the default).
Select `foreground = "systemd"` to run ordinary commands through the same scope
runner as background commands, preserving stdin, stdout, stderr and exit status.
Setup/doctor fails when this selected backend cannot create a scope; execution
does not silently fall back. The default process-group mode provides
signal forwarding; cleanup reports unfinished uncontained runs for inspection.
Nested scope launches remain separately registered under their inherited owner;
owner cleanup covers them. A single scope stop only covers that scope's cgroup.

## Delegation profiles

`just delegate config-check CONFIG` validates a separate TOML profile file.
Profile validation, asynchronous CLI execution and the MCP adapter are implemented.
Select `capabilities.delegation.config` in worker.toml and run setup to register
the configured MCP service. A successful configuration check does not run
an executor or prove model/service availability.

```toml
schema_version = 1

[profiles.reader]
frontend = "codex"
model = "your-configured-model"
reasoning_effort = "high"
mode = "read"
prompt = "prompts/reader.md"
visible_paths = ["src/**", "docs/**"]
timeout_seconds = 900
memory_bytes = 1073741824
max_processes = 64
skills = ["skills/project-guide"]

[profiles.reader.programs]
python = "/usr/bin/python3"

[profiles.reader.credentials]
codex_auth_file_env = "PROJECT_CODEX_AUTH_FILE"

[profiles.reader.mcp_servers.helper]
program = "python"
args = ["-m", "your_server"]

[profiles.reader.mcp_servers.helper.env]
SERVICE_TOKEN = "PROJECT_SERVICE_TOKEN"
```

Frontend currently accepts `codex`; modes accept `read` and `artifacts`.
Prompts, skill directories and executable program paths resolve relative to the
configuration file (absolute resource paths are also accepted). Each skill
directory must contain SKILL.md and have a distinct name. MCP servers reference
a declared program; remote host-side MCP connections are not part of this contract.
Visible-path globs describe project inputs, not configuration resource paths.
Timeout is required and positive; memory_bytes and max_processes are optional
positive limits. Memory and process limits apply to the entire systemd scope;
the Linux process limit counts threads too. Program, skill and server maps can be omitted when unused.

Credential values are environment-variable references, never secret values.
codex_auth_file_env names a variable containing the auth.json path at execution;
alternatively credentials.env.OPENAI_API_KEY names a variable holding the API key.
Other credential env entries and server env entries use the same mapping.
Configuration validation checks reference syntax without reading those values.
HOME, CODEX_HOME, PATH, SHELL and loader overrides belong to the sandbox.

Task requests separate `profile` and `task` from optional `revision`, explicit
`inputs` (sandbox input name to project-relative source file), and `contract`.
The contract contains `result_schema` (JSON Schema) and optional `artifacts`
(relative output name to positive byte limit). Read mode forbids artifacts.
`result.json` is reserved for the structured response. The task preparation and
result verification library is covered by `rust-test`.

Preparation resolves revision to a full commit and reuses the review snapshot
exporter with the profile's visible_paths. Explicit inputs also obey those globs;
they need no Git repository. Snapshots and copied files have a SHA-256 manifest.
Traversal, symlinks and sensitive development-control paths are rejected.
Preparation requires a new directory. Result verification uses the shared bounded
regular-file reader, checks JSON Schema and required artifacts, and records their
sizes and hashes. A model's declaration of completion does not satisfy this contract.

The delegated sandbox builder mounts prepared inputs at /project, /inputs and
/delegate-input, plus a writable /work and a private /codex. Only configured
programs are added under /tools; sh, bash, env and system libraries form the CLI
runtime. Configured skills are mounted read-only under /codex/skills. Host checkout,
home and user-manager sockets are not mounted. The environment starts empty and
receives fixed runtime variables and explicit credential references. Generated
Codex configuration is read-only, disables hooks and contains only the configured
MCP servers; their commands resolve inside this sandbox. Real bubblewrap/systemd
fixtures verify execution and limits. A real Codex read task has passed through
the configured MCP in an independent consumer, including input reading, result
validation and cleanup. Sandboxed MCP-service and artifact-model smoke checks
remain pending. The initial task message lists the profile's programs by sandbox
path; generic host utilities are not implicitly available.

`just delegate start CONFIG REQUEST_JSON` returns a run_id from the shared jobs
registry. `just delegate status RUN_ID` and `result RUN_ID` return OS state and the
retained report; `just delegate cancel RUN_ID` uses owner-checked scope cancellation.
The top-level outcome is authoritative: PASS requires a completed successful job,
a valid result and successful cleanup. RUNNING and UNKNOWN are not success.
Reports, input hashes, the request, profile, logs and verified artifacts remain
under the configured runtime/jobs/RUN_ID. Temporary input/private directories are
removed after saving the report. Result/status recover an interrupted report and
retry incomplete cleanup only after the job is terminal; cleanup errors remain
separate from validation findings. A failed job does not become PASS after cleanup.
While a background launcher is waiting for scope adoption, its PID/start/boot
identity keeps the run observable after the start caller exits. Cancellation in
this interval records a stop request; status remains stopping until the launcher
exits, and adoption refuses to execute the task. Temporary inputs are retained
until OS observations establish termination.

`worker delegate --root PROJECT mcp CONFIG` serves the same runner over stdio MCP.
Its tools are delegate_start, delegate_status, delegate_result and delegate_cancel.
The start schema lists configured profile names and accepts the same task contract
as the CLI request. Retain the returned run_id; disconnecting the MCP client leaves
the managed task running, and another connection can retrieve or cancel it.
Status/result can recover terminal reports and cleanup, so they are not read-only
operations. Tool errors and unsuccessful outcomes are returned with isError.
