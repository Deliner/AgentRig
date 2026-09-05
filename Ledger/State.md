# State

## Focus

Deliver P002 from the current goal attachment: portable worker capabilities, separate configuration, declarative setup and shared updates. Apply complexity-discipline.

## Workspace

Branch: feature/worker-capabilities

Revision: 08c8f90

P001 is integrated on master with a clean starting tree. This branch implements the user's corrected product boundary.

## Progress

Read the new attachment and current package, configuration, lint, review and upgrade entrypoints. Lint already supports an external root plus explicit config without worker setup. Review is a separate crate under Project and is not included by package init. D025 records the corrected ownership, superseding D008 for this repository. P002 retains all eight requested outcomes.

## Verification

The starting integration passed 208 worker and 21 review tests. This VAC changes scope documentation and delivery memory only; its normal staged gate remains pending. No new setup or packaging behavior is claimed.

## Blockers

None. Reuse package manifests, file ownership and upgrade operations; do not create a parallel installer or duplicate engines.

## Next action

Commit the ownership VAC, then move review under tooling/worker and connect the main worker executable to the same library. Preserve existing review tests and standalone lint behavior. Add capability configuration, setup and consumer upgrade/rollback verification before completing P002.
