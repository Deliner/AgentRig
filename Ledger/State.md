# State

## Focus

P163 (B165): a Rust lint kind `max-fallback` reports a `MAX` fallback that is an operand of an addition or a
multiplication. This VAC only makes P163 active.

## Workspace

- Branch: feature/max-fallback
- Revision: a8254aaf34231cff9954aaab78abed9a7cfb6f98

## Progress

- P162 merged into master; the batch continues with P163 in numeric order.

## Verification

- `just check --only memory` after the Plan edit.

## Blockers

None.

## Next action

Commit the activation, then add the inert kind, its detector, tests and decision as the first code VAC.
