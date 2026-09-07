# Private VCS protocol

Project checks, recovery, review and delegation can select a private VCS read adapter through their existing
configuration. The shared `vcs::Backend`/`Source` owner dispatches native Git,
native Mercurial and `vcs::external::Adapter` reads into the same snapshot and
visibility checks. A separate executable can implement the protocol without
linking AgentRig or publishing its implementation. Private setup, preview and
doctor, commit guards and feature integration use this boundary. Independent
installed consumers verify delivery through both native Mercurial and the example.

An adapter declaration contains one argv vector, for example:

```yaml
command: [python3, -B, /absolute/path/to/external_vcs.py]
```

Select the source in project `agentrig.yaml`:

```yaml
vcs:
  backend:
    command: [python3, -B, /absolute/path/to/external_vcs.py]
  base: team/main
  prefix: task/
```

`config-check` validates the declaration without executing the adapter. Project
`check`, `check --revision`, `memory-check`, `report` and `resume` use the selected
source for committed inputs, historical memory, content evidence and observations.
The gate's file selection and embedded lint use its working inventory; exported
checks inspect the exported files. Direct lint commands select an adapter with
`--vcs-config` as described below. An adapter failure never
falls back to a native repository found beside it. Native project selection now
requires matching repository metadata; omitted selection still means Git.
Private branch naming belongs to the adapter; configuration only requires
nonempty base and prefix. `feature-start` supports the write operation below.
Setup generates hooks and harness/MCP commands, previews files and registrations,
initializes the repository and registers hooks through the adapter. Configure a
private backend in YAML and use `setup`; the `init --vcs` choices remain native.
Commit guards inspect the selected backend's native commit context. Feature
integration runs shared mandatory checks between adapter preparation and completion.

`check --staged` is a native-index operation. Preparation fingerprints and exports
the native index before loading its staged configuration; an invalid unstaged
YAML file does not replace that configuration. The selected staged backend must
support index reads before any adapter or delivery check runs. Mercurial has no
staging index, and protocol v1 exposes no private index operation: use `check`
or `check --revision REV` instead. Saved check evidence uses the configured source
for subsequent index comparisons, rather than rediscovering a nearby repository.

For direct lint commands, put the Backend declaration alone in a YAML file:

```yaml
# vcs.yaml
command: [python3, -B, /absolute/path/to/external_vcs.py]
```

Both `agentrig lint` and `agentrig-lint` accept `--vcs-config vcs.yaml` alongside
`--root` and `--config`. The same option works for `lint-config-check` and
`lint-explain PATH`. This is a Backend declaration, not a complete project
configuration; a scalar `git` or `mercurial` also selects that native backend.
The declaration path is relative to the consumer root, while its command argv
retains the protocol's repository working-directory semantics. Diagnostics retain
the absolute selection-file path in their rerun command. Invalid declarations,
unsupported operations and failed adapters produce errors without native fallback.
Without this option, direct lint retains native discovery or physical-directory
inspection. It does not implicitly read project VCS settings; the project `check`
gate supplies its own selected source and handles exported inputs separately.

Set `repository.vcs` in the existing review project YAML:

```yaml
repository:
  vcs:
    command: [python3, -B, /absolute/path/to/external_vcs.py]
  visible_paths: [src/**]
  contract_paths: []
```

Set top-level `vcs` in the existing delegation YAML to the same command object.
The strings `git` and `mercurial` retain their previous meaning and serialized
form; omitted selection still defaults to Git. Invalid commands, unknown native
names, duplicate fields and unknown adapter fields fail configuration parsing or
validation. Command arguments are literal; relative paths are interpreted from
the source repository, so an absolute adapter path is useful across consumers.

Review reports retain the selection in `project_configuration`. Re-review must
retain that selection; external adapters must also retain the canonical source
root because opaque IDs may be local to one repository. A changed source requires
an explicit new review boundary. Native content hashes retain their existing
cross-checkout behavior. Delegation stores the selected source with its inputs
and code result. Read, artifacts and code modes keep their existing isolation and
checks; code mode still constructs its result patch using the internal Git
workspace and does not modify the source repository through the private adapter.

No shell interprets this vector. The executable must be nonempty, and arguments
cannot contain NUL. Each call starts one process with the consumer repository as
its working directory. AgentRig uses bubblewrap with the filesystem mounted
read-only for reads; the adapter must not require repository cache writes for reads.
For `start-feature`, `initialize`, `register-hooks`, `prepare-integration` and
`finish-integration`, the consumer repository root is mounted writable;
An owned temporary directory is also writable and supplied as `TMPDIR`, allowing
native hooks to export and check pending revisions. It is removed after the call;
the rest of the filesystem remains read-only.
Credentials and executable dependencies remain the caller's environment, outside
the protocol payload. This is a filesystem write restriction, not a claim that
the configured executable cannot access network services.

## Request and response

The process reads exactly one JSON request from stdin:

```json
{"version":1,"operation":"resolve","arguments":{"reference":"release"}}
```

On success it exits zero and emits only the response JSON on stdout:

```json
{"version":1,"result":"revision-42"}
```

Diagnostics belong on stderr. Exit 64 means the requested operation is
unsupported; AgentRig reports the operation name and stderr. Other nonzero exits
are failures. Unknown response fields, malformed JSON, wrong result types and
unsupported protocol versions are errors. A successful process exit alone does
not establish a valid result. No daemon, request registry or persistent adapter
session is required.

| Operation | Arguments | Result |
| --- | --- | --- |
| `resolve` | `reference` string | One exact revision ID string; ambiguous selections fail. |
| `head` | Empty object | Exact current revision ID, or null for an unborn repository. |
| `observe` | Empty object | `{branch, revision, status, merge_in_progress, rebase_in_progress}`; the first three are strings, the last two booleans. Empty revision means unborn. |
| `commit-context` | `revision` string or null | `[branch, merge]`; a branch string and boolean identifying whether this commit is a merge. |
| `start-feature` | `branch` string and `expected: {branch, revision}` strings | Null after creating the requested feature at the expected revision. |
| `prepare-integration` | `base` and `prefix` policy strings | `{feature, base, candidate}`; feature branch and exact input revision IDs for the prepared or resumed native integration. |
| `finish-integration` | `policy: {base, prefix}` and `expected: {feature, base, candidate}` | Null after committing the checked integration and retaining the feature reference. |
| `initialize` | `base` string | Null after ensuring a repository exists; an existing repository is preserved. |
| `repository-present` | Empty object | Boolean identifying whether this backend's repository exists; mismatched repositories fail. |
| `generate` | `binary` shell expression, `directory` relative hook path, `ignored` array of relative runtime paths | `{root_command, files}`; a shell command string and a map of relative file paths to UTF-8 contents. |
| `registration` | `directory` string | Nonempty array of `[key, current, desired]` string triples describing managed native settings. |
| `register-hooks` | `directory` string | Null after registering hooks without replacing conflicting user settings. |
| `parents` | `revision` string | Array of exact parent IDs; empty for a root revision. |
| `tree` | `revision` string | Array of `{path, kind, object}` entries. |
| `read` | `revision` and `path` strings | Byte array: integer values from 0 through 255. |
| `changed-paths` | `base` and `candidate` strings | Array of paths, including deletions and both sides of a rename. |
| `working-files` | Empty object | Tracked present files and non-ignored untracked files. |
| `diff` | `base` and `candidate` strings | Native change representation as a UTF-8 string. |

Revision and object IDs are opaque nonempty strings without control characters;
they need not be Git hashes. The adapter owns their immutable meaning. Tree kinds
are `file`, `executable`, `symlink` and `submodule`; consumers retain their existing
restrictions on exporting submodules or exposing symlinks. A symlink read returns
its target bytes, not the target file's contents.
Observation identifies the actual working branch, native status text and pending
operations. Its backend identity comes from configuration, never the reply.
Recovery resolves recorded revision references through this source, including
opaque non-hexadecimal IDs.

Commit context describes the commit being guarded, not an unrelated working-copy
branch. Backends requiring a pending revision must reject null; the Mercurial
example requires its changeset ID. `guard-commit --revision` selects the configured
source, resolves the reference and uses that exact ID for the exported configuration
and commit context. Branch policy remains shared: feature-prefix commits are
permitted, while the configured base permits merge commits only. The separate
mandatory check still runs against the pending revision through the installed hook.

Integration preparation must preserve unrelated changes, reject an unready feature,
and retain native conflict or failed-check state for recovery. It must also resume
an existing integration. AgentRig validates the returned context and runs its
mandatory full gate, including checked-input freshness. Only a passing gate calls
`finish-integration`, which must revalidate the expected native context and honor
native commit hooks. Failed operations are preserved, never automatically reset.
After completion AgentRig requires the configured base, clean status and no pending
operation. The context is transient; recovery belongs to the native VCS.

The Mercurial example uses its two merge parents, checks the current unique base
head and feature branch, and verifies the committed parents. These are Mercurial
semantics, not a requirement that every private backend implement two-parent merges.

Before `start-feature`, AgentRig requires the configured base branch, a clean
working copy and no pending merge or rebase. The adapter must reject stale
expected state, invalid or existing native branch names, and honor native hooks.
After success, AgentRig observes the repository again and requires the requested
branch, unchanged revision, clean status and no pending operation. A failure
retains the native state for inspection; AgentRig does not reset or undo it.
The example rechecks expected state before invoking `hg branch`; this is not an
atomic lock against concurrent external commands.

`Backend::initialize` dispatches native and private repository creation. The
private adapter initializes the requested base only when no repository exists;
repeated calls must preserve existing branches, revisions, user files and native
configuration. The example refuses a Git repository and uses native Mercurial
writes for a new repository. AgentRig observes the result to ensure it is readable.
Failed initialization retains its state for repair. This operation does not
install hooks; setup combines initialization with generated files and registration.

`Source` owns registration conflict checks for native and private backends. Each
external registration key must be unique and nonempty, with a nonempty desired
value. Empty current values mean unconfigured; a different nonempty value is a
conflict. The read-only `registration` operation must also work before a repository
exists, reporting the settings that will be needed. The write operation rechecks
conflicts before mutation, preserves unrelated settings and is idempotent.
AgentRig rereads external registration after success and requires every current
value to equal its desired value. It preserves failed native state for repair.
Setup validation, installation and doctor's registration check use this shared owner.

Mercurial registration also appends `include:<directory>.hgignore` to the root
`.hgignore` when absent, preserving existing rules and avoiding duplicates on
repeat setup. A symlink there is rejected without changing its target. This
include makes managed runtime exclusions visible to isolated reads, which disable
local hgrc commands and extensions. The existing hgrc ignore registration is
retained; it alone is insufficient for isolated status and inventory. Keep the
root ignore file under version control along with the generated environment.

`generate` is read-only and runs from the consumer root, including when an external
configuration file is used or the CLI starts elsewhere. `binary` is the supplied
shell expression for the installed AgentRig executable, evaluated after the hook
sets `root`. The adapter owns the executable hook contents and `root_command`;
the latter is embedded in generated harness and MCP launchers. Unlike the adapter
argv, these returned shell fragments are executable configuration and are reviewed
with the generated files. The root command must be nonempty and contain no NUL.

Files must use canonical relative paths within `directory`, or sibling auxiliary
files named with the `directory.` prefix, such as `.agentrig/hooks.hgignore`.
Traversal, control metadata, replacing the hook directory itself and unrelated
installation paths are rejected. The existing installer checks collisions,
ownership, permissions and changed resources. Native and external Mercurial
generation produce the same hook and runtime-ignore bytes. The external adapter
does not write these files itself. Preview reports their hashes and permissions;
registration reports use native strings for native backends and setting triples
for external backends.

Paths must be canonical relative paths. Empty components, `.`, `..`, NUL,
absolute paths and `.git`/`.hg` control components are rejected. Duplicate tree
entries fail; path lists are sorted and deduplicated. File bytes are read from
the selected revision, not from a possibly modified working file. Snapshot
visibility and private-material restrictions remain the snapshot owner's job;
valid protocol data alone is not permission to expose a file.

`Source::export_revision` exports complete committed inputs for delivery checks
through the same owner as native revision exports. It resolves the reference once
and uses that exact ID for tree and file reads, preserving binary bytes and
executable modes. Symlinks are created after regular files to prevent writes
through them; invalid paths, submodules and conflicting file/link trees fail.
This full export is not the restricted review snapshot. The project CLI uses it
for `check --revision`, retaining exact-export checks and input-mutation rejection.

## Independent example and evidence

[external_vcs.py](../examples/external_vcs.py) implements all listed operations over
Mercurial using only Python's standard library and the `hg` executable. It imports
no AgentRig code. It is an example of the boundary, not a proprietary VCS or a
second native implementation shipped for ordinary Mercurial consumers.

`tests/external_vcs.rs` invokes this separate process against real Mercurial
revisions, including binary files, symlinks, renames and a dirty working tree.
It compares every repository file, including metadata, before and after reads.
Other cases reject malformed replies, invalid versions and types, unsafe paths,
duplicate entries and invalid file kinds; a process attempting to overwrite a
working file fails and the original bytes remain unchanged.

`tests/registration.rs` registers the external Mercurial hook, invokes it through
a real native commit, preserves custom configuration and checks repeat registration,
both managed-setting conflicts, invalid descriptions and false success replies.
That hook is a test fixture; installed delivery is exercised separately below.
Generation tests compare native and external Mercurial output and reject escaped
paths and invalid root commands. Python consumers exercise private setup, repeat
installation, preserved settings, asset/harness conflicts and a real MCP handshake
through the generated launcher. A relative adapter is resolved from the consumer
root even when the CLI runs elsewhere; the installed binary passes doctor and
repeat preview. Installed private Mercurial hooks also reject direct base commits,
permit feature commits, roll back a failing mandatory check and retain unselected
working changes through a subsequent successful selected commit. A synthetic adapter
without Git/Mercurial metadata proves opaque-ID export and context use the same
resolved revision and retain branch/merge policy. Integration fixtures additionally
exercise failed gates, native commit rejection, mutated gate inputs and conflict
recovery through the example.

`test_installed_mercurial_delivery_recovers_failed_integration` separately installs
the binary in independent native/private consumers, copies the private adapter
into its consumer and uses generated Just recipes. It verifies bootstrap and
feature integration, installed commit hooks, failed gate recovery, exact merge
parents, retained branches, doctor and subsequent feature creation. The examples
seed a committed base before hook installation and retain ordinary language-cache
ignore rules. These checks establish VCS delivery, not real model-client acceptance.

Configuration-upgrade consumers select the private adapter before setup, update a
command and shipped skill, preserve consumer memory and native hook registration,
then roll back to the exact original file snapshot. A local skill conflict blocks
application until explicitly resolved; keeping the local file preserves its bytes.
These use the same upgrade/rollback implementation as native consumers.

Configured review tests run a failing review, a repair and a successful re-review
through the example, then reject a changed adapter or source root. Snapshot tests
retain scope and symlink restrictions. Native Python tests run read/artifacts/code
delegation through native Mercurial and the external example, preserve dirty
source files, exercise failed code checks and apply the returned patch to a
separate base checkout. The critic and delegate clients in these tests are
deterministic fixtures; these checks prove VCS routing, not real client acceptance.
