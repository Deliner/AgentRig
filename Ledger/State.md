# State

## Focus

P185 (B192): the memory history reads the fork revision once and keeps one reader of committed memory.

## Workspace

Branch: feature/history-reads
Revision: fbb479005ad65d213a885aae018499641074742f

## Progress

VAC 2: `revision_memory.rs` owns a revision snapshot with its tree, memory directory and cached texts. `history.rs`
loads one snapshot per checked revision and hands the fork snapshot to `invariant_history.rs`, whose `Prior` no
longer repeats the reading code. The temporary probe is removed.

## Verification

`just test` passes 616 tests, as on master `a362a1c`. `just lint` reports nothing. History step median 202 → 157 ms.

## Blockers

None.

## Next action

Commit VAC 2, then archive P185 and B192 with the timings and acceptance results and run `just feature-merge`.
