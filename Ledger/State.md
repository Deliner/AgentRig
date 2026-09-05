# State

## Focus

Implement the user's upgrade MVP from attachment 8f70329c-af70-4315-bcd3-cdc743504bcd: ownership manifest; upgrade plan/apply/rollback; preserved settings and memory; visible conflicts; stale-plan checks; recovery journal and post-update verification. Support one explicit adjacent-version transition. P001 remains paused.

## Workspace

Branch: feature/scaffold-upgrades

Revision: 83a81d9

This is the pre-upgrade baseline. The prior agent-native goal is fully integrated and complete. Current work is a new authorized worker capability, planned in .tmp/scaffold-upgrades-plan.md.

## Progress

The first VAC adds installation receipts with package/schema versions, ownership, SHA-256 and executable flags. Receipt generation and installation use one executable classification. Init still validates collisions before writing. D023 and I020 record and verify this baseline contract. Upgrade commands and migration/recovery are not implemented yet.

## Verification

All nine package and independent-consumer tests passed; strict lint reports no errors. The receipt test compares every installed entry and detects a subsequent skill edit. Full staged verification remains part of committing this VAC.

## Blockers

None. Release input will be a local new-version executable; reuse its existing init command in an isolated directory to obtain stock files. Existing consumers without manifests need a justified baseline reconstruction, not guessed ownership.

## Next action

Commit the installation-receipt VAC through the full staged gate, then implement reviewable plans, apply/recovery/rollback and the explicit adjacent-version transition. Preserve current project settings and memory, and keep the upgrade journal outside agent State. Complete independent interrupted/failing upgrade tests and final integration before marking the goal complete.
