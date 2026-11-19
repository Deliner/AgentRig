# State

## Focus

P166 (B173), VAC 2: the runtime pin moves only at the user's approval (D218). `propose-pin REVISION` writes a forward
pin proposal and lists changed sources; `approve` runs the `git.install` installer before writing the pin; the hook
refuses that installer with arguments; the gate accepts a changed pin only when approved and forward.

## Workspace

Branch: feature/pin-approval
Revision: c052f32a106ec619c643d359bbea4b7320f76e77

## Progress

- Product: change_delivery/git/pin.rs, approval.rs, overlay propose/proposed, `git.install` setting and validation,
  `Paths::source_matcher`, the `propose-pin` command, the shell guard refusal and the skill text.
- Tests: git/tests/pin.rs (gate and install at approval), shell.rs hook refusal, project_config installer validation.
- D218 detail, index row and precedence note.
- Not yet: phase 2 protected proposals (git.yaml `install`, commands.yaml `propose-pin`, development.just recipe,
  bootstrap.sh install only, rig differences), launchers test, AGENTS.md and README texts, archive, final pin.

## Verification

- `just lint`, `just run rustfmt`, `just run clippy` and `just candidate check --only memory` pass.
- `just run test -- pin approval shell_guard an_agent_installs an_installer_naming overlay`: 21 passed.

## Blockers

None.

## Next action

Commit VAC 2, then ask the user to run `! just bootstrap HEAD` so the installed runtime knows `propose-pin`, and
prepare the phase 2 proposals for `! just approve`.
