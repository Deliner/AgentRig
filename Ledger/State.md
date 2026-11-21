# State

## Focus

P176 delivers B183: each Project code file that lint warns about by file-size is split by responsibility, or stays
with its reason, behavior unchanged. VAC 3 moves two detector groups out of the Rust handler entry rust/mod.rs.

## Workspace

Branch: feature/cohesive-files
Revision: 9f46e1b09e51731c665f30a3b9607c160252b542

## Progress

VAC 1 saved and selected the card; VAC 2 split the silent-error detector. VAC 3 moves the discarded-result and spawn
detector, D090, D099, D114 and D155, into errors/discarded_result.rs, whose tests already sit in tests/errors, and
the eager fallback, D089 and D168, into std_forms/eager_fallback.rs; std_forms::call offers it to the call dispatch
and the macro body scan like ownership::call. rust/mod.rs falls from 441 to 265 lines; rust/ keeps 15 entries.

## Verification

just test languages:: passed 182 tests without build warnings; rustfmt on the staged files and memory passed.

## Blockers

None.

## Next action

Commit VAC 3 through the gate, then split the next oversized file and rerun lint with warnings.
