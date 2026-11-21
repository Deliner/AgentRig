# State

## Focus

P174, B181: make the gate faster and the rig code simpler without changing behavior. VAC 2 runs invariant oracle
discovery once per check command and runner instead of once per invariant.

## Workspace

Branch: feature/faster-gate
Revision: 2fc5715491532812dcd950b463ac156306d54504

## Progress

VAC 1 saved and selected the card. VAC 2 groups invariant discoveries in project_memory/oracles.rs: one
`cargo test ... -- --list` or one `pytest --collect-only -q` with every target, then each target must be listed.
Static oracle checks keep their order per invariant; a failed discovery names every invariant of its group.

## Verification

memory-check on this repository took 20.3 s before and 7.8 s after with a warm build; 14.4 s of it were 22 separate
cargo discoveries. `just test project_memory::tests::oracles` passed 4 tests. `just lint` reported only the temporary
timing probe, which is removed.

## Blockers

None.

## Next action

Commit VAC 2, then read committed decision and invariant memory through one `git cat-file --batch` per revision
instead of one Git process per file; that history step takes 4.3 s of memory-check here.
