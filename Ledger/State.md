# State

## Focus

P202 (B137), second VAC (AC4, AC5): `init --language rust` writes `manifest` into `.agentrig/rust.yaml` and no
`clippy` command or check; the rig drops its own `clippy` command and check and names `Project/Cargo.toml`;
`allow-justification` stops reading command `argv` and keeps `rustflags`; D239 records the choice.

## Workspace

Branch: feature/clippy-in-lint
Revision: 026ecd5517a3854d22f4e6592ffc86707debd7de
This VAC commits the four protected files the user approved with `! just approve` (`.agentrig/checks.yaml`,
`.agentrig/commands.yaml`, `.agentrig/rust.yaml`, `rig_differences.yaml`) with the code, tests, guides and Ledger.

## Progress

- `bundle/template.rs`: init writes `manifest` (the found `Cargo.toml`, else `Project/Cargo.toml`), no `clippy`
  command or check; `clippy_denials` leaves the `structural_lint` API.
- `languages/flag_list.rs`: only `rustflags` sequences are read; the descriptor and `justify-allow` say so.
- Tests: `project_setup/tests/clippy.rs` checks AC1 and AC4 on an initialized example crate; `initialized("rust")`
  and the delivery tests copy the example crate through one `example` helper; the flag-list test checks AC5.
- D239 row and detail; D184 keeps only its card, delivery and related links; `clippy profile` links of D060, D068,
  D107, D109, D127 and D129 move to `profile.rs`; D159's link moves to `run.rs`; D132, D137, D152, D153 and D180 lose
  `dev gate`; succession note added.

## Verification

- `just run test project_setup::`: 60 passed in 15.1 s (14.8 s for 61 before this VAC).
- `just run test structural_lint::`: 359 passed.
- `just lint` (installed runtime, now running Clippy on `Project/Cargo.toml`) and `just candidate lint`: clean.
- `just run rustfmt`, `just config-check` and `just check --only memory`: pass.

## Blockers

None.

## Next action

Commit this VAC through the gate, then `just propose-pin HEAD` under the user's consent and commit the pin; archive
P202 and B137 and run `just feature-merge`.
