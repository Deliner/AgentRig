# State

## Focus

P181 (B188): the maintained file list is built once per memory check and without the directory walk where only
files are needed. VAC 1 saves card B188 and selects it as P181.

## Workspace

Branch: feature/file-listing
Revision: e7ad50ef22c7f0dc8206de119b1f4ecb53275a30

Uncommitted temporary timers in Project/cli_dispatch/main.rs and Project/shared/testing/tests/mod.rs serve the
before and after measurement; they stay out of every commit and are restored before the code VAC.

## Progress

Card B188, Backlog row, Plan row P181 and Plan/181.md are written. Baseline on master e7ad50e in a new project:
memory check median 112 and 120 ms; one listing 31 ms, of it git ls-files 11 ms and the directory walk 18 ms.

## Verification

Memory check of the card pending.

## Blockers

None.

## Next action

Run `just check --only memory`, commit VAC 1, then split a files-only listing out of maintained_files, use it in the
memory check once, in the gate file list and in the architecture check.
