# State

## Focus

Autonomous maintenance (speed-ups without business-logic change): the runtime runs git from its exec path, so the
macOS xcrun shim at /usr/bin/git is resolved once per process instead of on every git call.

## Workspace

Branch: feature/git-exec-path
Revision: f18c4862d8141613ba5fb6fbe06d928f01ae04c1

## Progress

Project/shared/vcs/git.rs: program() asks `git --exec-path` once (OnceLock) and spawns <exec-path>/git by absolute
path for context_text and command; when the lookup fails or the directory holds no git, it spawns `git` through PATH
as before. The absolute program keeps posix_spawn; a PATH set in the child environment forced fork+exec (+~20 ms).

## Verification

Probe A/B on an installed fixture with /usr/bin/git first in PATH: check ~285 -> ~184 ms, feature-merge ~1185 ->
~1005 ms, feature-start ~240 -> ~210 ms; with the exec path already first, timings equal the baseline. Agent hook
events spawn no git, so hooks pay nothing. Full suite: 633 passed twice with the candidate. A test-side absolute git
path showed no measurable gain (50-53.5 s either way) and was dropped.

## Blockers

None.

## Next action

Commit through the gate, propose the pin for the user's `! just approve`, then feature-merge.
