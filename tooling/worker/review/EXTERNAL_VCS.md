# Private VCS read protocol

The shared Rust API `vcs::external::Adapter` implements the read side of a private
VCS process boundary. A separate executable can implement it without linking
AgentRig or publishing its implementation. Project/review/delegate configuration
selection and delivery writes are still pending P007 work; this component does
not yet make an external backend selectable through the AgentRig CLI.

An adapter declaration contains one argv vector, for example:

```yaml
command: [python3, -B, /absolute/path/to/external_vcs.py]
```

No shell interprets this vector. The executable must be nonempty, and arguments
cannot contain NUL. Each call starts one process with the consumer repository as
its working directory. AgentRig uses bubblewrap with the filesystem mounted
read-only; the adapter must not require repository cache writes for reads.
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

Paths must be canonical relative paths. Empty components, `.`, `..`, NUL,
absolute paths and `.git`/`.hg` control components are rejected. Duplicate tree
entries fail; path lists are sorted and deduplicated. File bytes are read from
the selected revision, not from a possibly modified working file. Snapshot
visibility and private-material restrictions remain the snapshot owner's job;
valid protocol data alone is not permission to expose a file.

## Independent example and evidence

[external_vcs.py](../examples/external_vcs.py) implements all listed reads over
Mercurial using only Python's standard library and the `hg` executable. It imports
no AgentRig code. It is an example of the boundary, not a proprietary VCS or a
second native implementation shipped for ordinary Mercurial consumers.

`tests/external_vcs.rs` invokes this separate process against real Mercurial
revisions, including binary files, symlinks, renames and a dirty working tree.
It compares every repository file, including metadata, before and after reads.
Other cases reject malformed replies, invalid versions and types, unsafe paths,
duplicate entries and invalid file kinds; a process attempting to overwrite a
working file fails and the original bytes remain unchanged.
