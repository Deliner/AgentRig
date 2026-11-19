# State

## Focus

P167 (B174), VAC 2: the exception `DECISION: DNNN RULE` is parsed and its decision lists it (AC2, D219).

## Workspace

Branch: feature/named-exceptions
Revision: d92f5243073ceea579078d88be423f1a345006ac

## Progress

- `source_parsers::exceptions` reads named exceptions from Rust, Python and shell comments and YAML and TOML lines.
- `reference()` accepts the named form, so it counts as a reference; `INVARIANT:` with a rule is no reference.
- The memory check refuses an unlisted exception and a listed `[exception RULE](FILE)` that the file lacks.
- Lint still lifts any rule under any marker; strict lifting comes in a later VAC.

## Verification

- `just run test -- project_memory:: structural_lint::languages::tests::rust_accepts`: 44 passed.
- `just candidate check --only memory`: PASS. `just lint --json`: no findings.

## Blockers

None.

## Next action

Commit VAC 2, then VAC 3: the gate refuses a new exception until the user approves it (AC4).
