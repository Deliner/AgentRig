# State

## Focus

P202 (B137) VAC A: lint and the gate's lint check run Clippy with the AgentRig profile on the library, binaries and
examples of the Cargo.toml that the optional `manifest` of `.agentrig/rust.yaml` names (AC1-AC3); init is unchanged.

## Workspace

Branch: feature/clippy-in-lint
Revision: 286a13f3917cded3d19299798875dec16b3d5b54

## Progress

- `rust.yaml` takes an optional relative `manifest`; `Context::manifest` resolves it.
- `structural_lint/clippy` owns the profile, moved from `project_setup/rust/clippy.rs` with its D132, D137, D152,
  D153 and D180 markers. It reads `cargo metadata` to add `--lib` only for a package with a library, runs
  `cargo clippy --bins --examples --message-format=json` with `-D warnings` and the profile denials through the
  caller's launcher and parses compiler messages into rule, path, line and text; a run that cannot check the
  project becomes one `clippy` message naming its cause.
- `Mode::Lint { clippy }` adds each message as an error finding with the configuration skill.
- The CLI `lint` and the gate's lint check pass a Clippy run when the manifest is set; `command_runner::output`
  runs it unconfined with the project rustflags and the staged cargo target.
- The init template still writes the `clippy` command, now from `structural_lint::clippy_denials()`.
- Decisions D132, D137, D152, D153 and D180 link the profile at `structural_lint/clippy/profile.rs`.
- Tests: `structural_lint/tests/clippy.rs` covers AC1-AC3 through `lint --json`; a gate test in
  `change_delivery/gate/tests/outcomes.rs` blocks a staged Clippy finding. Both README files describe `manifest`.

## Verification

- `just run test`: 642 passed before the record parser dropped serde `tag` and `other` and before the README edits.
- After the parser change: `just run test -- structural_lint::tests::clippy` 6 passed; `just lint` and
  `just run clippy` exit 0; lint ran before the README edits.
- `just candidate memory-check` passes with the known D112, D175 and D231 warnings.
- The first commit attempt passed build, candidate-lint, lint and memory and failed rustfmt; after the formatting
  repair `just run rustfmt` passes. The commit gate reruns all checks and the affected test groups.

## Blockers

- VAC B (AC4, AC5) changes protected files: it drops the rig's `clippy` command and check, adds
  `manifest: Project/Cargo.toml` and drops the Clippy entries of `rig_differences.yaml`; it waits for approval.
- The successor decision, likely D239, waits for B206's approved Decisions index exclusion; the pin needs approval.

## Next action

Commit VAC A through the gate, prepare the VAC B proposal in the scratchpad, measure the Clippy time and pause P202.
