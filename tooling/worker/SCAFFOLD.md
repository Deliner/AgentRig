# Portable scaffold runtime

The portable entry point is `worker.toml` in the selected project root. Initial distribution targets Linux. The runtime is a Rust binary; consumer projects need their own configured tools, Git, and bubblewrap for read-only commands. Just is a thin optional command interface. The consumer does not compile the worker or run the worker repository's tests.

A distributor builds the pinned crate with `cargo build --release --locked --manifest-path tooling/worker/Cargo.toml` and supplies the resulting `discipline-worker` and `discipline-lint` executables. Both lint interfaces use the same engine; standalone lint accepts an external root and policy without installing worker files there. `init` copies its running executable and bundled assets into a consumer, pinning the package version in the generated configuration. See [independent examples](examples/README.md) for complete bootstrap commands. See upgrades below for the supported transition and recovery commands.

Project capabilities are selected in `worker.toml`:

```toml
[capabilities]
lint = true

[capabilities.review]
config = ".worker/review/config/review.yaml"

[capabilities.delegation]
config = "agents/profiles.yaml"
```

Lint defaults to enabled for existing projects and uses `paths.lint`. Disabling
it requires removing lint checks from the configured gate. Review is enabled by
its configuration reference, relative to the project root. `config-check` validates
every enabled capability, including review resources and contracts; unknown
capability keys fail the schema check. `review config-check`, `review mcp` and
`review run REQUEST_JSON` use the project's configured review. Explicit review
config paths remain available for standalone calls. `--root` selects the consumer
for configured calls; resource paths inside review configs remain config-relative.
Delegation is enabled by its own configuration reference. Create that profile file
and its referenced prompts/programs/skills using the [delegation format](README.md#delegation-profiles).
`delegate config-check`, `delegate mcp` and `delegate start REQUEST_JSON` then use
the selected file; explicit config arguments remain supported. Profile resources
resolve relative to the profile file. Setup validates these consumer-owned resources
and registers `worker_delegation`; it does not rewrite the profiles or copy secrets.
It installs delegate-task under `paths.skills` and routes the generated project
instructions to that skill. Capability-specific skills use the same installation
ownership and conflict handling as the other shipped guidance.
Its environment forwards the declared credential/MCP variable references, executor
override and existing job owner/parent identifiers. Use the same WORKER_OWNER after
reconnecting to cancel owned tasks. The MCP tool timeout is 60 seconds; tasks run
asynchronously and their own timeouts remain profile settings. Doctor requires
working systemd scopes, bubblewrap and native Codex when delegation is enabled.
`discipline-worker setup --root CONSUMER` reads the existing declaration and
prepares the environment. To obtain a starting declaration and assets, use
`init --root CONSUMER --review true`, edit the generated settings, then run setup.
Setup can also start from only worker.toml and any custom referenced resources.
It installs missing stock assets, validates a temporary preview, registers Git
hooks and the `worker_review` MCP server, creates runtime/report directories and
runs doctor. A new consumer receives a Git repository on the configured base.
Authentication is separate: provide Codex authentication through CODEX_HOME;
REVIEW_CODEX_BIN can select the installed native CLI. These environment variables
are forwarded to MCP; setup never copies credentials into the project.

Repeated setup preserves configuration, memory, file permissions, comments and unrelated Codex
settings. Unchanged stock assets can be refreshed within the pinned release.
Locally modified skills/adapters, conflicting hook registration or conflicting
MCP settings stop setup before file installation and identify the preserved
conflict. Reconcile that named file/setting and retry. Disabling a capability requires
disabling or removing its existing worker_review/worker_delegation MCP entry; setup reports the
conflict instead of silently replacing it. Missing dependencies are reported by
doctor after installation; fix them and repeat setup. Release changes use upgrade.
Setup uses the installation manifest and the same atomic writer as upgrade.

The generated MCP entry sets the documented
[Codex stdio settings and tool timeout](https://learn.chatgpt.com/docs/extend/mcp?surface=cli).
Its timeout exceeds the review deadline by 60 seconds; the launcher resolves the
consumer Git root, so the command does not contain the worker repository path.

New installations write `.worker/manifest.json` with manifest_version, package_version, config_schema and a files map. Each relative path records SHA-256, ownership and its executable flag. Ownership is runtime, asset, configuration, editable (skills/adapters), or memory. The receipt excludes itself; it describes shipped contents, so local edits do not silently change that baseline. User-created files are not added automatically. Missing receipts in older installations must not be treated as proof that their files are stock.

## Commands

All project commands accept `--root PATH`; otherwise the current directory is the root. `--version` and `--help` do not need a project.

| Command | Result |
| --- | --- |
| `setup` | Install or reconcile the existing declaration, register adapters/MCP and diagnose dependencies; preserve settings and report conflicts. |
| `init` | Create standard config, memory, skills, binary and hook adapters; reject collisions before writing. Options select language, source, memory, skills, base branch, branch prefix and `--review true|false`. Review defaults to false; enabling it installs editable presets and the standard review skill. |
| `config-check` | Validate schema, cross-references, skills and lint applicability without analyzing source contents. |
| `doctor` | Diagnose the installed runtime, configured executables, sandbox and hooks. |
| `commands` / `run NAME -- ARGS` | List or execute the shared catalog. Arguments remain argv elements. |
| `report` | Summarize command timing, latest check evidence and repeated check failures from the configured runtime directory. |
| `lint` / `lint-config-check` / `lint-rules` | Analyze sources, validate only lint settings, or list actual rule/language capabilities. |
| `check [--staged] [--only CHECK_ID]` | Sequential gate over the working tree or exported index, including config and skills from that tree; --only repeats one configured stage. |
| `memory-check` | Check the four memory files, links, decision history and executable invariant targets. |
| `resume` | Return State, Plan, observed Git operations, State revision comparison and check freshness as JSON without changing the project. |
| `feature-start NAME` / `feature-merge` | Create or integrate a branch according to configured base/prefix. Integration runs the gate and retains the branch. |
| `hook` | Read an agent event as JSON from stdin and emit guidance or denial. |

Warnings do not fail lint; blocking findings exit 1 and configuration failures exit 2. External checks retain their process exit codes; a check configured with `warning = true` can report a nonzero exit without failing the gate. Interruptions still stop it. External output is preserved. Failure diagnostics identify the check, location or selected scope, cause, configured repair skill and a shell-quoted RERUN command. Lint JSON also includes rerun. In a project requiring catalogued shell operations, execute that command through `just run write --`; `just check --only CHECK_ID` is the shorter gate retry. Commit and integration adapters always run the full gate.

## Configuration ownership

`worker.toml` has schema `version = 1` and an exact `runtime` package version. Unknown fields are errors. Project-relative filesystem paths cannot escape the root. Source and check selectors are globs. The generated file is a complete editable example.

- `paths`: source selectors and locations of memory, skills, lint configuration and transient runtime data.
- `git`: base branch and working-branch prefix.
- `commands.NAME`: `argv`, `cwd` (default `.`), `accepts_args` and `read_only`. An empty argv requires forwarded arguments. Shell evaluation happens only if the catalog explicitly invokes a shell.
- `checks`: ordered IDs, `kind` (`command`, `lint`, `memory`), optional command reference, `include`, repair `skill`, and `warning`. A check with no matching files is skipped.
- `hooks`: file-to-skill routes, optional reminder JSON and the corresponding discipline skill. The hook and runner use the same command catalog.
- `oracles.ID`: command check, runner (`pytest` or `cargo`) and exact test target. Discovery uses that command's cwd, sandbox and shared process execution. Pytest targets are relative to the command cwd; keep its collection root aligned (for example, configure --rootdir . when a nested pytest config changes that root).

Lint uses strict YAML and compiled Rust/Python handlers; [rule semantics](README.md#rules-and-languages) describe counting, selectors and parser limits. The installed template supplies all five rules and focused repair skills. Language rules block unnamed conditions, functions above 40 nonblank lines and signatures above 4 counted inputs. Unsupported selected languages are configuration errors, not silently ignored files. Numeric warning/error limits and named-condition severity are editable project policy.

Read-only commands require functioning Linux bubblewrap. Worker does not fall back to unrestricted execution when isolation is unavailable. Command records are append-only JSONL in `paths.runtime`; they describe process results, not task lifecycle. Staged command timing logs remain in the disposable exported tree; the latest gate evidence is saved in the original project runtime directory. If you relocate `paths.runtime`, add the new directory to the consumer Git ignore rules; initialization supplies an ignore rule for the default location.

## Memory and recovery

The initialized indexes are `Plan.md`, `Decisions.md` and `Invariants.md`; details use stable numeric IDs and matching paths such as `Plan/001.md`. `State.md` is a compact snapshot with Focus, Workspace, Progress, Verification, Blockers and Next action sections.

Plan rows contain ID, Status, Depends on, Feature and User capability. Status is pending, active, paused or complete; dependencies must exist and be acyclic, with completed prerequisites for active/complete features. Details contain Feature, User capability and Acceptance; paused/complete details also contain Delivery. At most one feature is active.

Decision rows contain ID, Decision and Applies in. Details contain Context, Chosen, Rejected, Rationale and Consequences. Supported source application links need a real `DECISION: DNNN` comment. Committed identities, statements and detail contents are preserved; superseding a choice adds a new decision. Application links describe current owners and can move during refactoring; Git retains previous ownership. Rust, Python and shell application markers are parsed as comments.

Invariant rows contain ID, Invariant and Enforced by. Details contain Predicate and Oracle. The enforcement link names a function in a Rust/Python source file, with its `INVARIANT: INNN` comment before the function (attributes/decorators may intervene). Its configured oracle must name that function, including its class/module scope, and be discoverable by the actual test runner. Multiple declarations of that qualified name are ambiguous to syntax-only linking and are reported rather than treated as one marked target. Unsupported source languages are reported explicitly.

`resume` compares explicit `Branch: ` and `Revision: ` claims, optionally wrapped in backticks, with Git. Commit hashes may be full or unambiguous abbreviations. It reports current, stale or unverified snapshot status, an explicit state_revision.head_changed comparison (null when unknown), and merge/rebase facts resolved through Git so linked worktrees are supported. It does not infer completed acceptance, select a new feature, or overwrite State. The agent reconciles the snapshot with live evidence and current instructions.

The latest gate attempt is atomically replaced in `paths.runtime/checks.json`, including start time, observed HEAD, input fingerprint, staged/working-tree scope, optional selected check, results and completion state. This is disposable verification evidence, not a VAC registry or task history. An unfinished record means no completion was recorded; it does not establish that a process is still alive. A check that changes inputs or HEAD is marked inputs-changed. Results survive the temporary staged export and a fresh worker process.

`resume.checks` exposes last_run, revision_matches, worktree_matches, index_matches, current and full_gate_passed. Current requires matching HEAD and content plus recorded completion; staged evidence also requires the same index. Index identity is captured before export and compared again after checks. A passing selected check never sets full_gate_passed. After a commit changes HEAD, matching content remains visible separately; do not confuse content equality with a check executed against the new revision. No additional mandatory gate is introduced. Missing evidence is absent; unreadable or malformed evidence is unavailable, never a passing result. The fingerprint covers Git tracked and non-ignored untracked paths (only indexed paths for a staged run), file bytes, executable bits and symlink targets, excluding paths.runtime. Non-Git projects use physical files and cannot claim a matching Git revision. This does not fingerprint external tools, environment variables, ignored dependencies or remote services; retain their original output when relevant to acceptance.

`report` shows consecutive failures of a check and its presented skill and retry command. Counts reset on success or when that check is absent from the previous attempt; they describe check attempts, not identical errors or proof of reading guidance. If an applied skill fails to help with the same concrete error, use the repair skill to correct the canonical instruction or diagnostic and verify the actual failed scenario. VAC intent and observed verification belong in context and commit descriptions; there is no manager, scheduler or extra checkpoint.

The four editing skills and one route handler provide pre-edit guidance. Complexity reminders retain session/compaction accounting and retry behavior using the configured schedule. The worker repository uses the same runtime; its worker.toml supplies repository-specific paths, commands and oracles. Canonical skills live in assets/skills, are embedded at build time, and are exposed to this repository through .agents/skills.


## Upgrades

The 0.2.0 executable can prepare the explicit 0.1.0 → 0.2.0 transition:
`/path/to/new/discipline-worker upgrade plan /path/to/new/discipline-worker --root PROJECT`.
The release argument is a local executable. Its existing `init` exports stock
content into a temporary directory using the project's memory and skill paths.
A 0.1.0 installation without a receipt reconstructs its stock baseline using
its installed executable; the plan identifies that origin.

The command writes `plan.json`, `diff.txt` and checksummed before/after blobs
under the configured runtime directory's `upgrade/plan-*` directory. It shows
local conflicts and the required config-check, doctor and project check steps.
Settings and memory stay intact; the runtime pin and lint path change while root
TOML comments are preserved. The legacy lint policy is explicitly converted to
YAML with its values preserved; its original formatting and comments remain in
the reviewed preimage for rollback. An existing YAML destination is a conflict.
Planning leaves installed files unchanged. Review the diff and set each conflicting entry's `resolution` in `plan.json` to
`"keep"` or `"replace"`. There is no automatic conflict merge. Leave the remaining
plan fields intact. Kept local contents are recorded separately from stock
checksums in the new receipt. Doctor accepts explicitly kept Git adapter hashes
while still checking executable permissions and registration; later unreviewed
adapter changes fail diagnosis.

Apply with `/path/to/new/discipline-worker upgrade apply /path/to/plan.json --root PROJECT`.
Files must still match their reviewed paths, bytes and modes. Before changing
them, apply freezes the resolved plan and copies verified preimages into
`upgrade/operation`. Atomic writes update `journal.json` after each completed
step. Configuration validation, installation diagnosis and the full configured
project check must pass before the operation is marked applied. A failed check
leaves a resumable operation: fix the project and repeat the same apply command.

Use `/path/to/new/discipline-worker upgrade rollback --root PROJECT` to restore
previous files. Repeating rollback continues an interrupted restoration. It
refuses to overwrite subsequent user edits; preserve or resolve those edits
before retrying. A new apply after rollback retains the earlier journal under
`upgrade/restored-*`. Unrelated project files and memory edits are not reset.

Keep the supplied new executable available throughout recovery. Upgrade
commands work even when interruption leaves binary and configuration versions
out of step. The installed `just upgrade` adapter forwards these commands;
`resume` and session hooks expose the operation and next action. Commit and
merge reject an unfinished operation. The journal is technical recovery data,
separate from agent State. This release supports only 0.1.0 → 0.2.0, keeps schema
version 1, and performs no memory-format migration.
