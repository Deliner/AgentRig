# State

## Focus

P172 delivers B178 entry-bypass. VAC 13 routes the uses of change_delivery and of the Rust setup support through
their entries: gate, git, approval, consent, cargo, clippy, flags and toolchain are declared privately.

## Workspace

Branch: feature/entry-boundaries
Revision: 5f7d1bfd7ca1d491e26b05d7e713890eca75e2ce

## Progress

- change_delivery offers the check, feature branch, guard, approval and consent functions under distinct names.
- project_setup/rust offers the crate layout, Clippy settings, build flags and toolchain.
- agent_hooks, cli_dispatch and project_setup use them only through these entries.
- Reexporting init from project_setup would form the cycle project_setup to initialization to template.rs, because
  initialization uses the bundle generator in the parent files; it stays for the next VAC.

## Verification

- `just run rustfmt` clean, `just run clippy` clean.
- Candidate lint with the entry rules: entry-bypass 38 to 2, entry-use 153, architecture clean.

## Blockers

None.

## Next action

Commit VAC 13, then move the bundle generator into project_setup/bundle so that initialization uses a sibling and
project_setup can offer init through its entry.
