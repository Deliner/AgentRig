# State

## Focus

P159 (B161): a lint error for an array literal with calls among its elements indexed by a non-literal.

## Workspace

- Branch: feature/array-index-literal
- Revision: 96db9407ed62bb1f34a57c059d4482b00c27d82e

## Progress

- P159 is active; P158 is merged into master.

## Verification

- `just check --only memory` after activation.

## Blockers

None.

## Next action

Commit the activation, then add the inert kind with its detector, tests and decision, pin it, and enable it in
lint.yaml with its repair skill.
