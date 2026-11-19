# State

## Focus

P162 (B164): enable `outer-flag-filter` with its skill `check-the-flag-first` and rewrite the rig's own sites so the
flag is checked before the call. This VAC also pins ab3ccb3, the revision carrying the inert kind.

## Workspace

- Branch: feature/outer-flag-filter
- Revision: ab3ccb391036724d16abe2276368fda7d1269e7f

## Progress

- `just bootstrap HEAD` installed ab3ccb3 and wrote it to stable.txt.
- `lint.yaml`, the Rust handler, the catalog and init's skill list carry the kind; the skill lives under control-flow.
- The 30 rig sites became `flag.then(|| call).flatten()`; the let chain first tried in `project_memory/source.rs`
  failed named-if-condition, so the skill names no let chain.

## Verification

- `just lint` with the candidate passes with the kind enabled; `just run rustfmt`, `just run clippy`,
  `just lint-config-check` and `just check --only memory` pass.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, then archive P162 and B164 with the pin and run `just feature-merge`.
