# State

## Focus

P163 (B165): enable `max-fallback` with its skill `saturate-or-handle`. This VAC also pins 16353f6, the revision
carrying the inert kind.

## Workspace

- Branch: feature/max-fallback
- Revision: 16353f6d52691376238759c918c5f91c2d1ecb7f

## Progress

- `just bootstrap HEAD` installed 16353f6 and wrote it to stable.txt.
- `lint.yaml`, the Rust handler, the catalog and init's skill list carry the kind; the rig rule skips test files as
  init's does; the skill lives under control-flow beside `subtract-checked`.
- The rig has no site to repair: its only `unwrap_or(i32::MAX)` stands under unary minus.

## Verification

- `just lint` with the candidate passes with the kind enabled; `just run test -- project_setup` passes, 57 tests,
  after the rig rule gained init's test-file exclude; `just run rustfmt`, `just run clippy`,
  `just lint-config-check` and `just check --only memory` pass.
- The first commit gate failed `rule_examples_ship_excludes_only_where_decided`: the kind was missing from the
  test's list of kinds that skip test code. With it added, `just run test -- structural_lint::tests::discovery`
  passes.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, then archive P163 and B165 with the pin and run `just feature-merge`.
