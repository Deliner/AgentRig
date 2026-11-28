# State

## Focus

P202 (B137): resume after P199 and P200 merged. The branch is rebased onto master and its first VAC re-verified
before the pin of the rebased head under the consent of the user.

## Workspace

Branch: feature/clippy-in-lint
Revision: 836814d58c9d74b0babcc8c79f006d4c6c2f8e93

## Progress

- `git rebase --onto master 676548a b2b77fc` replayed the activation as `6db75d8` and VAC A as `836814d`; the
  pause `d47f25b` and the pin `789cfca` are dropped, so the next pin descends from the master pin `e4fcb33`.
- Conflicts resolved: Plan.md keeps the master rows with P202 active; `init` takes `CargoLayout` from
  `structural_lint` after P200 moved the manifest reader; the D136 row keeps the master manifest link and the
  D137 row links `structural_lint/clippy/profile.rs`.
- This VAC joins the `project_setup::rust` import of `bundle/template.rs` into one line, as rustfmt requires
  after the rebase removed two names from it.
- Plan/202.md records the rebase, VAC A, the measurement, the blocker and the resumption order.

## Verification

- `just candidate --version` builds; `just lint` and `just candidate lint` exit 0.
- `just run test project_setup::` 61 passed; `just run test structural_lint::` 360 passed.
- The first commit attempt passed build, candidate-lint, lint and memory and failed rustfmt on that import;
  after the repair `just run rustfmt` passes.

## Blockers

- VAC B (AC4, AC5) changes protected files and waits for `! just approve` of `b137-vac-b-proposal/`.

## Next action

Commit, run `just propose-pin HEAD` under the consent of the user, commit the pin with P202 paused, then place
the proposal in `.cache/runtime/proposal/` and wait for the approval.
