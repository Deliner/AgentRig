# State

## Focus

Maintenance under the user's overnight speed-up authorization, no business logic change: a catalog job registers
its record once with its argv, access and lifetime instead of saving a placeholder record and then preparing it, so
each command pays one synchronized record write fewer and no record shows the default access of an unprepared job.

## Workspace

Branch: feature/cwd-git-hooks
Revision: 4069d7cd92ec3919443ec9ae4333ace818567b63

Staged: `Project/command_runner/jobs/{job.rs,mod.rs,architecture.yaml}` (`Job::create` takes a `Launch`; `prepare`
is gone) and `Project/command_runner/mod.rs` (foreground and background callers). Unstaged and kept for the next
VAC: the Git hooks taking the root from the working directory, which waits for the user's `! just approve`.

## Progress

Implemented; the record keeps its fields and later attach, launch and finish saves.

## Verification

`just run test -- command_runner::` passed 30 tests; `just check --only` clippy, rustfmt and lint pass. A temporary
probe (removed) ran `run read -- wc -c AGENTS.md` with the release candidate on this repository: 4 x 30 rounds
averaged 37.0-41.5 ms before and 34.5-35.1 ms after; one synchronized save measured 3.2 ms.

## Blockers

None for this commit.

## Next action

Commit through the gate, then describe the Git hook VAC again in State.
