# Private VCS protocol

Project checks, recovery, review and delegation can select a private VCS read adapter through their existing
configuration. The shared `vcs::Backend`/`Source` owner dispatches native Git,
native Mercurial and `vcs::external::Adapter` reads into the same snapshot and
visibility checks. A separate executable can implement the protocol without
linking AgentRig or publishing its implementation. Project VCS setup, commit and
integration operations for private backends remain pending P007 work.

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
Setup, generated hooks/MCP registration, commit and feature integration currently
report explicit unimplemented private-operation errors.

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
For `start-feature`, `initialize` and `register-hooks`, the consumer repository root is mounted writable;
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
| `start-feature` | `branch` string and `expected: {branch, revision}` strings | Null after creating the requested feature at the expected revision. |
| `initialize` | `base` string | Null after ensuring a repository exists; an existing repository is preserved. |
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
install hooks; the complete private setup flow remains pending until generated
files and repository discovery are supported throughout setup.

`Source` owns registration conflict checks for native and private backends. Each
external registration key must be unique and nonempty, with a nonempty desired
value. Empty current values mean unconfigured; a different nonempty value is a
conflict. The read-only `registration` operation must also work before a repository
exists, reporting the settings that will be needed. The write operation rechecks
conflicts before mutation, preserves unrelated settings and is idempotent.
AgentRig rereads external registration after success and requires every current
value to equal its desired value. It preserves failed native state for repair.
Setup validation, installation and doctor's registration check use this shared
owner. Generated hook files, root lookup and MCP command generation still require
the remaining private setup work.

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
That hook is a test fixture; full installed delivery-gate acceptance remains separate.

Configured review tests run a failing review, a repair and a successful re-review
through the example, then reject a changed adapter or source root. Snapshot tests
retain scope and symlink restrictions. Native Python tests run read/artifacts/code
delegation through native Mercurial and the external example, preserve dirty
source files, exercise failed code checks and apply the returned patch to a
separate base checkout. The critic and delegate clients in these tests are
deterministic fixtures; these checks prove VCS routing, not real client acceptance.
