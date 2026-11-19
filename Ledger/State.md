# State

## Focus

P163 (B165), first code VAC: the inert Rust kind `max-fallback`, its detector, two tests and decision D209. The kind
is not yet in the rig's `lint.yaml`, the Rust handler, the catalog or init.

## Workspace

- Branch: feature/max-fallback
- Revision: 12874eb7071e1b71b133dfa501c27c83b24c0937

## Progress

- `std_forms/max_fallback.rs` reports a binary `+` or `*` with an operand `.unwrap_or(MAX)` or `.unwrap_or(T::MAX)`;
  `std_forms/mod.rs` tries it after the epsilon form for binary expressions, outside test code and justified lines.
- `Reference::MaxFallback` maps to the descriptor `MAXFALLBACK` with the future skill `saturate-or-handle`.

## Verification

- `just run test -- std_forms` passes, 14 tests; `just run rustfmt`, `just run clippy`, `just lint` and
  `just check --only memory` pass.

## Blockers

None.

## Next action

Commit this VAC, run `just bootstrap HEAD`, then enable the kind with its skill and repair any rig sites.
