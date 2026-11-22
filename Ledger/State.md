# State

## Focus

Maintenance authorized by the user's autonomous goal: speed up development without changing behavior. VAC 1: the
release profile compiles incrementally (D232), so candidate rebuilds after a change of `Project` take seconds.

## Workspace

Branch: feature/faster-candidate-build
Revision: 89d947fb1726670f4057de7e11746b931748c361

## Progress

`Project/Cargo.toml` sets `[profile.release] incremental = true` with marker D232; decision D232 and its index row
record the measurements and rejected alternatives.

## Verification

Measured in `.cache/worker`: a one-line code change rebuilt in 2.7–4.0 s instead of 11.3 s; candidate lint median
0.638 s incremental against 0.644 s non-incremental over five runs each. The commit gate checks the rest.

## Blockers

None. Moving the pin needs the user's `! just approve`; the change does not alter the installed runtime's behavior.

## Next action

Commit VAC 1, run `just propose-pin HEAD`, then `just feature-merge`.
