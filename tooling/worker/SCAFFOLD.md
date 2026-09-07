# Portable scaffold runtime

The portable entry point is `agentrig.yaml` in the selected project root. Initial distribution targets Linux. The runtime is a Rust binary; consumer projects need their own configured tools, the selected VCS executable, and bubblewrap for read-only commands. Just is a thin optional command interface. The consumer does not compile the worker or run the worker repository's tests.

A distributor builds the pinned crate with `cargo +1.98.1 build --release --locked --manifest-path tooling/worker/Cargo.toml` and supplies the resulting `agentrig` and `agentrig-lint` executables. Both lint interfaces use the same engine; standalone lint accepts an external root and policy without installing worker files there. `init` copies its running executable and bundled assets into a consumer, pinning the package version in the generated configuration. See [independent examples](examples/README.md) for complete bootstrap commands. See upgrades below for the supported transition and recovery commands.

`paths.service` selects the service directory and defaults to `.agentrig`.
For example, `agentrig init --root CONSUMER --service 'team rig'` places the
binary, VCS hooks and installation receipt there. Generated defaults also place
skills, lint settings, reminders and runtime data under that directory; their
individual paths remain configurable. Adapters quote the selected path, including
spaces and shell metacharacters. Newlines and Just interpolation syntax (`{{`)
are rejected. Changing this field in an existing installation does not relocate
its files: setup reports conflicting adapters before writing.

Project capabilities are selected in `agentrig.yaml`:

```yaml
capabilities:
  lint: true
  review:
    config: .agentrig/review/config/review.yaml
  delegation:
    config: agents/profiles.yaml
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
`agentrig init --interactive --root CONSUMER` starts the setup wizard. It asks for
the project directory, language, service/source/memory/skills paths, branch naming,
review, VCS, an optional existing delegate-profile YAML path relative to the project,
and selected checks. Enter accepts defaults. `cancel`, EOF or declining the final
confirmation leaves the target untouched, including when its directory is absent.
The wizard shows ordinary `agentrig.yaml` and the shared setup preview before
writing. It uses the same templates, validators, reconciliation and installation
as declarative setup; no separate settings database is created. Existing projects
with `agentrig.yaml` use setup or upgrade instead. Bare `init` and setup remain
fully noninteractive. Dependency diagnostics still run after installation.

`agentrig setup --root CONSUMER` reads the existing declaration and
prepares the environment. To obtain a starting declaration and assets, use
`init --root CONSUMER --review true`, edit the generated settings, then run setup.
Setup can also start from only agentrig.yaml and any custom referenced resources.
It installs missing stock assets, validates a temporary preview, registers native VCS
hooks and the `worker_review` MCP server, creates runtime/report directories and
runs doctor. A new consumer receives a repository of the selected kind on the configured base.
Authentication is separate: provide Codex authentication through CODEX_HOME;
REVIEW_CODEX_BIN can select the installed native CLI. These environment variables
are forwarded to MCP; setup never copies credentials into the project.

Repeated setup preserves configuration, memory, file permissions, comments and unrelated Codex
settings. Unchanged stock assets can be refreshed within the pinned release.
Locally modified shipped skills/adapters, conflicting hook registration or conflicting
MCP settings stop setup before file installation and identify the preserved
conflict. Reconcile that named file/setting and retry. Disabling a capability requires
disabling or removing its existing worker_review/worker_delegation MCP entry; setup reports the
conflict instead of silently replacing it. Missing dependencies are reported by
doctor after installation; fix them and repeat setup. Release changes use upgrade.
Setup uses the installation manifest and the same atomic writer as upgrade.

`agentrig setup --preview --root CONSUMER` prepares and validates the same files
and registrations without installing them, initializing a repository or invoking doctor.
It prints JSON with changed file paths, before/after SHA-256 and resulting modes,
VCS/Codex registrations, runtime directories and required dependencies. Existing
conflicts are reported before writes, just as in ordinary setup. The report includes
only managed MCP settings, so unrelated settings and credential values remain in
their original files. Dependencies are requirements, not successful runtime probes;
doctor verifies their availability after installation. Preview is recomputed from
current inputs on each call; it creates no independent saved installation plan.

The generated MCP entry sets the documented
[Codex stdio settings and tool timeout](https://learn.chatgpt.com/docs/extend/mcp?surface=cli).
Its timeout exceeds the review deadline by 60 seconds; the launcher resolves the
consumer root through the selected VCS, so the command does not contain the worker repository path.

Select `init --vcs git|mercurial` or set `vcs.backend` in YAML. The `vcs` section
also owns `base` and `prefix`; omitted backend means Git. Existing `git` sections
remain accepted as an alias, but declaring both sections is an error. Setup rejects
a backend that differs from the existing repository. Mercurial setup registers
`hooks.pretxncommit.agentrig` and `ui.ignore.agentrig` in `.hg/hgrc`, preserving
unrelated settings, hooks and the user's `.hgignore`. The additional ignore file
excludes configured runtime data and default review runtime/reports. Repeated
setup leaves matching registrations unchanged; conflicts stop before installation.
Review presets generated for Mercurial select its native snapshot backend.
Mercurial consumers need `hg` and working bubblewrap. Native reads mount the
filesystem read-only because Mercurial can otherwise refresh cache files and
dirstate during inspection. Initialization and hook registration remain explicit
writes. Delegation also requires Git for its internal patch builder.

New installations write `<paths.service>/manifest.json` (default `.agentrig/manifest.json`) with manifest_version, package_version, config_schema and a files map. Each relative path records SHA-256, ownership and its executable flag. Ownership is runtime, asset, configuration, editable (skills/adapters), or memory. The receipt excludes itself; it describes shipped contents, so local edits do not silently change that baseline. User-created files are not added automatically. Missing receipts in older installations must not be treated as proof that their files are stock.

## Commands

All project commands accept `--root PATH`; otherwise the current directory is the root. `--version` and `--help` do not need a project.

| Command | Result |
| --- | --- |
| `setup [--config CONFIG_YAML] [--preview]` | Prepare an external or installed declaration; preview changes or install/reconcile it and run doctor. |
| `init` | Create standard config, memory, skills, binary and hook adapters; reject collisions before writing. Options select language, source, memory, skills, service directory (`--service`), base branch, branch prefix and `--review true|false`. Review defaults to false; enabling it installs editable presets and the standard review skill. |
| `config-check` | Validate schema, cross-references, skills and lint applicability without analyzing source contents. |
| `config-resolve CONFIG_YAML` | Preview local package composition as JSON: values, declaring files and configuration digests. Does not install resources or replace capability validation. |
| `config-inspect CONFIG_YAML` | Validate the complete prepared environment and print JSON with root settings, selected lint/review/delegate configuration documents, package provenance and resource digests. Accepts `--root PATH` for the consumer; does not install files, execute hooks or resolve credential values. |
| `doctor` | Diagnose the installed runtime, configured executables, sandbox and hooks. |
| `commands` / `run NAME -- ARGS` | List or execute the shared catalog. Arguments remain argv elements. |
| `report` | Summarize command timing, latest check evidence and repeated check failures from the configured runtime directory. |
| `lint` / `lint-config-check` / `lint-rules` | Analyze sources, validate only lint settings, or list actual rule/language capabilities. |
| `check [--staged \| --revision REV] [--only CHECK_ID]` | Sequential gate over the working tree, Git index or exact Git/Mercurial revision, including config and skills from that tree; --only repeats one configured stage. |
| `memory-check` | Check the four memory files, links, decision history and executable invariant targets. |
| `resume` | Return State, Plan, observed native VCS operations, State revision comparison and check freshness as JSON without changing the project. |
| `feature-start NAME` / `feature-merge` | Create or integrate a branch according to configured base/prefix. Integration runs the gate and retains the branch. |
| `hook` | Read an agent event as JSON from stdin and emit guidance or denial. |

Warnings do not fail lint; blocking findings exit 1 and configuration failures exit 2. External checks retain their process exit codes; a check configured with `warning: true` can report a nonzero exit without failing the gate. Interruptions still stop it. External output is preserved. Failure diagnostics identify the check, location or selected scope, cause, configured repair skill and a shell-quoted RERUN command. Lint JSON also includes rerun. In a project requiring catalogued shell operations, execute that command through `just run write --`; `just check --only CHECK_ID` is the shorter gate retry. Commit and integration adapters always run the full gate.

Historical memory validation reads the checked-out Git or Mercurial revision
through the shared VCS owner. Published decision identities and detail contents
remain immutable when the memory directory moves. Plain directories and unborn
repositories have no historical baseline; backend errors are reported rather
than treated as absent history. This applies to `memory-check` and memory gates;
Mercurial setup installs the commit hook described below. `feature-start NAME`
supports Git and Mercurial through the shared VCS owner. It requires the configured
base branch, a clean working tree and no pending merge/rebase. Git creates a new
branch reference; Mercurial sets the working directory's named branch, recorded
permanently by the next commit. Native branch commands reject invalid or existing
names and honor consumer hooks.

Configuration validates `vcs.base` and the names formed by `vcs.prefix` using the
selected backend's rules, without creating a repository. Git reference syntax
remains enforced. Mercurial permits names such as `main line` and a prefix such as
`task `; reserved labels, integer names, forbidden characters and surrounding
whitespace are rejected. Surrounding whitespace would be trimmed by its native
CLI and would no longer match the configured base or prefix. The prefix must be
nonempty and distinguish feature branches from the configured base.

`feature-merge` dispatches through the shared VCS owner. Git rebases divergent
features with merge history preserved, checks the candidate and merges it into
the configured base with an explicit merge commit. Both backends require a full
passing gate whose revision and content still match before completing integration.
A check that changes those inputs cannot authorize integration even if it exits
successfully. Git command interruptions and failures retain native exit codes and
recovery state; the feature branch is retained.

Mercurial `feature-merge` selects the base branch's single head, updates to that
exact revision and merges the committed feature using the native internal merge
tool. Multiple base heads and uncommitted feature names are rejected before
switching. The merged working tree must pass the full gate with unchanged inputs;
the commit retains both selected parents and the feature's named-branch history.
Configured native hooks run normally, including setup's pending-changeset gate.
A conflict or failed gate preserves the pending merge. Resolve conflicts with
native `hg resolve`, repair failing checks, handle retained backup/untracked files,
then repeat `feature-merge` from the base branch. To discard an attempted merge,
first preserve any work you need, then use native `hg merge --abort`. If the base
advanced, abort and retry from the retained feature instead of committing against
the outdated base. Complete multi-VCS delivery, including private adapters, is not
yet accepted.

## Configuration ownership

`agentrig.yaml` has schema `version: 1` and an exact `runtime` package version. Unknown fields are errors. Project-relative filesystem paths cannot escape the root. Source and check selectors are globs. The generated file is a complete editable example.

- `paths`: source selectors and locations of memory, skills, lint configuration and transient runtime data.
- `vcs`: backend, base branch and working-branch prefix (`git` remains a YAML alias).
- `commands.NAME`: `argv`, `cwd` (default `.`), `accepts_args` and `read_only`. An empty argv requires forwarded arguments. Shell evaluation happens only if the catalog explicitly invokes a shell.
- `checks`: ordered IDs, `kind` (`command`, `lint`, `memory`), optional command reference, `include`, repair `skill`, and `warning`. A check with no matching files is skipped.
- `hooks`: file-to-skill routes, optional reminder JSON and the corresponding discipline skill. The hook and runner use the same command catalog.
- `environment`: selected custom skills, programs, frontend hooks and MCP servers; uses the shared resource declarations described below.
- `oracles.ID`: command check, runner (`pytest` or `cargo`) and exact test target. Discovery uses that command's cwd, sandbox and shared process execution. Pytest targets are relative to the command cwd; keep its collection root aligned (for example, configure --rootdir . when a nested pytest config changes that root).

Lint uses strict YAML and compiled Rust/Python handlers; [rule semantics](README.md#rules-and-languages) describe counting, selectors and parser limits. The installed template supplies all five rules and focused repair skills. Language rules block unnamed conditions, functions above 40 nonblank lines and signatures above 4 counted inputs. Unsupported selected languages are configuration errors, not silently ignored files. Numeric warning/error limits and named-condition severity are editable project policy.

Read-only commands require functioning Linux bubblewrap. Worker does not fall back to unrestricted execution when isolation is unavailable. Command records are append-only JSONL in `paths.runtime`; they describe process results, not task lifecycle. Staged command timing logs remain in the disposable exported tree; the latest gate evidence is saved in the original project runtime directory. If you relocate `paths.runtime`, add the new directory to the consumer Git ignore rules; initialization supplies an ignore rule for the default location.

## Configuration composition preview

Use `agentrig config-inspect /path/to/agentrig.yaml --root /path/to/project`
to validate the whole selected environment through the setup preparation path.
The JSON `configuration` contains prepared root settings; `configurations` maps
installed relative paths to selected lint, review, review-material and delegate
settings. `composition` records declaring files, package versions, digests and
nested configuration provenance. Resource paths show their prepared locations;
credential references remain references. Inspection accepts a changed declaration
without modifying or reconciling the current installation. It does not execute
hooks, model calls or source checks; use setup preview to inspect installation
conflicts and doctor to check runtime dependencies.

The shared resolver is available through `agentrig config-resolve declaration.yaml`.
It reads YAML, writes JSON to stdout and does not install or execute anything.
A declaration contains ordinary configuration fields plus `packages` and optional
`overrides`. A reusable package has this envelope:

```yaml
schema_version: 1
id: common-checks
version: "1.0"
configuration:
  commands:
    test:
      argv: [python3, -m, pytest]
```

For example, a consumer declaration can select and override that command:

```yaml
packages:
  - path: ../shared/package.yaml
    id: common-checks
    version: "1.0"
overrides: [/commands/test/argv]
commands:
  test:
    argv: [cargo, test]
```

Packages can themselves declare `packages` and `overrides`. Paths to packages
resolve relative to the declaring file, including outside the consumer root.
IDs and versions on imports are optional exact assertions; there is no version
range resolver or network registry. Repeated references to the same canonical
package at the same destination apply once. Different files cannot claim the same package ID. Cycles,
missing files, unknown envelope fields, invalid YAML and mismatched assertions
are errors identifying the import chain.

An import may specify `into`, a JSON pointer to a mapping destination. Omit it
to merge at the root. A resource package whose `configuration` directly contains
`skills`, `programs`, `hooks` and `mcp_servers` can be imported with
`into: /environment` in a project and `into: /profiles/reader` in a delegate
configuration. Importing it again into `/profiles/writer` supplies that profile
independently, without copying the package or giving it another ID.

Nested imports inherit their parent's destination; their own `into` appends to
it. Package overrides are relative to that package's destination; root overrides
use complete resolved addresses, for example `/profiles/writer/hooks/guide/args`.
Destinations use nonempty mapping keys with the same `~0`/`~1` escapes as overrides;
they do not index lists or select source fragments. Provenance and resource paths
continue to refer to the file declaring the value. Scoping does not relax cycle,
identity or conflict validation.

Setup checks package identity across the complete assembly, including the root
declaration and nested lint, review, material and delegate configurations. The
same canonical package can contribute to several configurations; a different
file or digest claiming that ID is rejected before installation writes anything.

Mappings compose recursively. Repeated scalar or ordinary list definitions
require an explicit override, even when their values agree. Overrides replace
the addressed value in full; list elements are never appended implicitly.
Top-level `checks` and `rules` are named lists: IDs must be unique, new IDs append
in declaration order, and replacing an existing ID requires its own override,
such as `/checks/tests`. An override of `/checks` replaces the whole list;
`checks: []` can remove it from the resolved declaration. Capability validators
and mandatory runtime controls still decide whether the resulting policy is valid.
Unused or duplicate overrides fail. Addresses escape `/` as `~1` and `~` as `~0`;
named lists use IDs instead of positional indexes.

Output contains `configuration`, `provenance`, `packages`, `root` and `root_digest`.
Provenance records the declaring file for each retained value; a container merged
from multiple files has only its children's origins. Package records contain ID,
version, canonical path and SHA-256 of the exact YAML bytes read. Digests describe
configuration inputs, not yet their referenced resource contents. Composition
leaves resource strings untouched and does not grant filesystem access to delegates.

`agentrig setup --config /path/to/agentrig.yaml --root CONSUMER --preview`
resolves an external declaration and validates the selected capabilities before
writing. Remove `--preview` to install. Relative `--config` arguments use the
invocation directory; resource references use their declaring configuration's
directory, while source selectors continue to describe the consumer.

Setup copies selected guidance, lint policies, review prompts/contracts/project
settings and delegate prompts/skill directories/programs into the installation.
Declared stock guidance retains its canonical installed path; other inputs use
content-addressed paths under the configured service directory. Skill support
files and executable modes are retained. Nested resource-directory symlinks are
rejected. Missing references to shipped defaults use those embedded defaults;
missing custom resources are errors. Secrets remain environment references.

The installed root declaration contains resolved portable resource references.
`composition.json` records configuration provenance and source/resource digests;
runtime operation and ordinary repeated setup do not require the source tree.
Relative review output paths remain relative to the installed review configuration.
The shipped `../runtime` and `../reports` defaults therefore use `inputs/runtime`
and `inputs/reports` after import; the service's `.gitignore` excludes these outputs
while retaining configuration and prompt inputs. Projects choosing other output
paths manage their Git exclusions as part of their own repository settings.
Repeating external setup accepts identical inputs. Changed external inputs require
`agentrig upgrade plan --config /path/to/agentrig.yaml --root CONSUMER`.
It uses the same resource builder and writes a reviewed `plan.json` and `diff.txt`
under the existing recovery directory. Configuration changes are explicit;
local file conflicts require `resolution: keep` or `resolution: replace` in the
JSON plan. Memory is retained, and unrelated Codex settings remain in place.
Apply with `agentrig upgrade apply PLAN --root CONSUMER`; all existing post-upgrade
checks run. `upgrade rollback` restores the previous installation. The plan keeps
its payloads, so applying it does not require the external source tree.

Configuration updates retain the runtime version, service directory and recovery
runtime location. Use a release upgrade for a runtime version change. A subsequent
configuration update archives the completed recovery operation; rollback targets
the latest operation. Invalid or stale plans cannot replace that recovery record.
Project and delegate resources share the same typed environment declarations and
resource builder, with different placement and execution permissions.

The external setup/update builder also resolves `packages` and `overrides` in
lint policies, review runner settings, review material settings and delegate
profile files. These use the same package schema and conflict rules as the root
declaration. A delegate declaration can, for example, import a package defining
`profiles.reader` and explicitly override `/profiles/reader/model`.

Review/delegate resource references follow the configuration or package that
declares each value. Imported lint repair skills likewise follow their declaring
package; an explicit `skill_root` supplies the shared base and is itself resolved
relative to its declaring file. Without `skill_root`, skills in the root lint
policy retain the existing consumer-relative convention. Source selectors remain
consumer-relative. Installed capability configs are fully resolved YAML, read by
the ordinary component validators and runtimes. The `configurations` section of
`composition.json` records each nested composition's values, provenance and package
digests; changes to those inputs require an explicit update too.

## Custom project environments

The optional root `environment` contains `skills`, `programs`, `hooks` and
`mcp_servers`. These fields have the same schema as the corresponding fields in
[delegate profiles](README.md#delegation-profiles). For example:

```yaml
environment:
  skills: [skills/project-guide]
  programs:
    helper: tools/helper
  hooks:
    guide:
      event: SessionStart
      program: helper
      args: [context]
      timeout_seconds: 10
  mcp_servers:
    project_tools:
      program: helper
      args: [mcp]
      env:
        API_KEY: PROJECT_TOOLS_TOKEN
```

External setup resolves resources relative to their declaring YAML file, including
package origins. It installs custom skill directories under `.agents/skills` for
native frontend discovery; `paths.skills` still selects the shipped worker guidance.
Skill support files remain editable. Programs are copied with executable modes into
the shared input bundle. Installed operation does not need the original package tree.
Duplicate skill names, missing programs, unknown fields and unsupported hook events
are configuration errors. MCP names `worker_review` and `worker_delegation` are
reserved for the worker capabilities.

Setup appends the chosen hooks to the mandatory worker routes and generates MCP
registrations. The native `environment-hook NAME` and `environment-mcp NAME`
adapters resolve the selected binding and replace themselves with its program;
the frontend owns their stdio, process lifetime and hook timeout. Project hooks
run with the project frontend's permissions. Delegate hooks instead run inside
the delegate sandbox. Both use the same event contract and argv rendering.
MCP credentials resolve from the named environment variables only when launched;
setup and preview store references, never their values. Updating selections uses
`upgrade plan --config`: removed managed MCP services are disabled and unrelated
Codex settings are preserved. Custom hooks do not replace mandatory runner gates.

## Memory and recovery

The initialized indexes are `Plan.md`, `Decisions.md` and `Invariants.md`; details use stable numeric IDs and matching paths such as `Plan/001.md`. `State.md` is a compact snapshot with Focus, Workspace, Progress, Verification, Blockers and Next action sections.

Plan rows contain ID, Status, Depends on, Feature and User capability. Status is pending, active, paused or complete; dependencies must exist and be acyclic, with completed prerequisites for active/complete features. Details contain Feature, User capability and Acceptance; paused/complete details also contain Delivery. At most one feature is active.

Decision rows contain ID, Decision and Applies in. Details contain Context, Chosen, Rejected, Rationale and Consequences. Supported source application links need a real `DECISION: DNNN` comment. Committed identities, statements and detail contents are preserved; superseding a choice adds a new decision. Application links describe current owners and can move during refactoring; Git retains previous ownership. Rust, Python and shell application markers are parsed as comments.

Invariant rows contain ID, Invariant and Enforced by. Details contain Predicate and Oracle. The enforcement link names a function in a Rust/Python source file, with its `INVARIANT: INNN` comment before the function (attributes/decorators may intervene). Its configured oracle must name that function, including its class/module scope, and be discoverable by the actual test runner. Multiple declarations of that qualified name are ambiguous to syntax-only linking and are reported rather than treated as one marked target. Unsupported source languages are reported explicitly.

`resume` compares explicit `Branch: ` and `Revision: ` claims, optionally wrapped
in backticks, with the discovered native repository. Commit hashes may be full or
unambiguous abbreviations. It reports current, stale or unverified snapshot status,
an explicit state_revision.head_changed comparison (null when unknown), and a
`vcs` object containing backend, branch, revision, native status and merge/rebase
facts. Git consumers retain the `git` alias; Mercurial reports its own identity.
Plain directories have `vcs: null`; unborn repositories have an empty revision.
Repository observation errors are reported, not interpreted as a clean checkout.
Git operation paths support linked worktrees. Mercurial uses working-directory
parents for pending merges and its native rebase state marker. It does not infer
completed acceptance, select a new feature, or overwrite State. The agent
reconciles the snapshot with live evidence and current instructions.

The latest gate attempt is atomically replaced in `paths.runtime/checks.json`, including start time, native revision, input fingerprint, scope, optional selected check, results and completion state. This is disposable verification evidence, not a VAC registry or task history. An unfinished record means no completion was recorded; it does not establish that a process is still alive. A working-tree or staged check that changes inputs or the checked-out revision is marked inputs-changed. Results survive temporary exports and a fresh worker process.

`check --revision REV` resolves one immutable Git or Mercurial revision and checks its full exported tree, including its configuration and skills. It cannot be combined with --staged. Memory history is compared against each parent of that revision, with no prior baseline for a root revision. The report records the resolved hash in revision and sets revision_export; diagnostic retries retain that hash. Fingerprints use that revision's tracked paths, excluding paths.runtime, and detect changes to exported inputs during checks. A changed export retains inputs-changed evidence and exits 2 even when external commands returned zero. Changes to the original checkout do not alter the selected revision or its result. The report is stored under the original root using the selected configuration's runtime path. Current/full_gate_passed still require the original checkout's revision and files to match, and --only remains a partial check. Full exports preserve symlinks and executable bits; submodules and VCS control paths are rejected explicitly. This command does not provide the restricted isolation boundary used by review.

Mercurial setup registers the generated adapter in `.hg/hgrc` (default service path):

```ini
[hooks]
pretxncommit.agentrig = sh .agentrig/hooks/pretxncommit
```

The adapter runs `guard-commit --revision "$HG_NODE"` followed by the full
`check --revision "$HG_NODE"`. Mercurial exposes the pending changeset to this
hook and rolls its transaction back on a nonzero exit. The guard uses the selected
changeset's configuration and branch; direct base commits are rejected. The gate
checks exactly that changeset, including a partial commit's selected files, while
unrelated working changes remain untouched. The retained report identifies the
attempted hash even after rollback; that rejected hash is no longer available for
revision-based retries until a new commit is attempted. Native source reads disable
repository-configured Mercurial commands, so they do not recursively execute hooks.
Setup separately reads trusted local registration settings to preserve conflicts.

`resume.checks` exposes last_run, revision_matches, worktree_matches, index_matches, current and full_gate_passed. Current requires matching native revision and content plus recorded completion; staged evidence also requires the same index. Index identity is captured before export and compared again after checks. A passing selected check never sets full_gate_passed. After a commit changes the revision, matching content remains visible separately; do not confuse content equality with a check executed against the new revision. No additional mandatory gate is introduced. Missing evidence is absent; unreadable or malformed evidence is unavailable, never a passing result. The fingerprint covers Git or Mercurial tracked and non-ignored untracked paths (only indexed paths for a Git staged run), file bytes, executable bits and symlink targets, excluding paths.runtime. Plain directories use physical files and cannot claim a matching VCS revision. Mercurial has no staging index and rejects --staged explicitly; ordinary working-tree checks are supported. This does not fingerprint external tools, environment variables, ignored dependencies or remote services; retain their original output when relevant to acceptance.

`report` shows consecutive failures of a check and its presented skill and retry command. Counts reset on success or when that check is absent from the previous attempt; they describe check attempts, not identical errors or proof of reading guidance. If an applied skill fails to help with the same concrete error, use the repair skill to correct the canonical instruction or diagnostic and verify the actual failed scenario. VAC intent and observed verification belong in context and commit descriptions; there is no manager, scheduler or extra checkpoint.

The four editing skills and one route handler provide pre-edit guidance. Complexity reminders retain session/compaction accounting and retry behavior using the configured schedule. The worker repository uses the same runtime; its agentrig.yaml supplies repository-specific paths, commands and oracles. Canonical skills live in assets/skills, are embedded at build time, and are exposed to this repository through .agents/skills.


## Upgrades

The development 0.3.0 executable prepares the explicit 0.2.0 → 0.3.0 transition:
`/path/to/new/agentrig upgrade plan /path/to/new/agentrig --root PROJECT`.
The release argument is a local executable. Its existing `init` exports stock
content into a temporary directory using the project's memory and skill paths.
A 0.2.0 installation without a receipt reconstructs its stock baseline using
its installed executable; the plan identifies that origin.

The command writes `plan.json`, `diff.txt` and checksummed before/after blobs
under the configured runtime directory's `upgrade/plan-*` directory. It shows
local conflicts and the required config-check, doctor and project check steps.
Project settings move from worker.toml to agentrig.yaml, with an updated runtime
pin and YAML resource references. The transition retains the original service
placement through explicit `paths.service: .worker`, so process data and recovery
journals stay at their existing locations. Lint, review/project and delegate
configurations are explicitly converted. Values and memory are preserved; original formatting
and comments remain in reviewed preimages for rollback, rather than in the YAML
output. An existing YAML destination is a conflict. External review material
configurations and their selected contracts are copied through the shared resource
bundle into `.worker/inputs`; configuration-relative contract references are
rewritten for the installed copy. External source files are never modified or
deleted. The plan contains the imported payloads and includes their hashes and
ownership in the installation manifest. Rollback removes those installed copies
and restores the original references. Keep external sources available if the
restored 0.2.0 installation must run after rollback.
Delegate profiles are validated through the shared profile resolver during explicit
migration. External prompts, complete skill directories and declared programs are
copied into the same input bundle, retaining support files and executable modes.
Resources already inside the consumer remain there with portable relative references.
Credential references remain references. Converted profiles and imported files enter
the installation manifest, and the target's generated guidance includes the selected
delegate capability. Local instruction changes remain reviewed conflicts; rollback
restores the old profile and removes imported copies.
Planning leaves installed files unchanged. Review the diff and set each conflicting entry's `resolution` in `plan.json` to
`"keep"` or `"replace"`. There is no automatic conflict merge. Leave the remaining
plan fields intact. Kept local contents are recorded separately from stock
checksums in the new receipt. For customized Git adapters, the planner recognizes
the stock `exec "$root/.worker/bin/discipline-worker"` call and proposes changing
that executable while preserving surrounding custom code and comments. Review the
diff: `replace` selects this migrated adapter, and `keep` retains the original.
A kept adapter containing that executable call is rejected before installation;
update other custom invocation forms explicitly when reviewing their replacement.
This is a targeted migration, not analysis of arbitrary shell behavior. Doctor
accepts hashes of explicitly reviewed custom adapters, including migrated replacements,
while still checking executable permissions and registration; later unreviewed
adapter changes fail diagnosis.

Apply with `/path/to/new/agentrig upgrade apply /path/to/plan.json --root PROJECT`.
Files must still match their reviewed paths, bytes and modes. Before changing
them, apply freezes the resolved plan and copies verified preimages into
`upgrade/operation`. Atomic writes update `journal.json` after each completed
step. Configuration validation, installation diagnosis and the full configured
project check must pass before the operation is marked applied. A failed check
leaves a resumable operation: fix the project and repeat the same apply command.

Use `/path/to/new/agentrig upgrade rollback --root PROJECT` to restore
previous files. Repeating rollback continues an interrupted restoration. It
refuses to overwrite subsequent user edits; preserve or resolve those edits
before retrying. A new apply after rollback retains the earlier journal under
`upgrade/restored-*`. Unrelated project files and memory edits are not reset.

Keep the supplied new executable available throughout recovery. Upgrade
commands work even when interruption leaves binary and configuration versions
out of step. The installed `just upgrade` adapter forwards these commands;
`resume` and session hooks expose the operation and next action. Commit and
merge reject an unfinished operation. The journal is technical recovery data,
separate from agent State. The transition supports only 0.2.0 → 0.3.0, keeps schema
version 1, and performs no memory-format migration. Normal commands load only
agentrig.yaml. During an active upgrade with that file not yet installed, resume
and SessionStart expose technical recovery guidance without loading legacy task
settings. Historical memory checks still read the committed legacy memory location
so a format change cannot remove the prior decisions baseline.

P004 migration and independent consumer acceptance are complete. See
[the acceptance map](examples/PORTABILITY.md) for tests and real MCP evidence.
