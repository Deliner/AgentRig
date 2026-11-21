# State

## Focus

P171 entry-use and P172 entry-bypass under the user's consent: VAC 17 moves the items of the shared vcs, diagnostics
and source_parsers entries into named files so each entry is a table of contents.

## Workspace

Branch: feature/entry-boundaries
Revision: 8cde20f062abf43af4573510036981892b1319aa

## Progress

- vcs: repository.rs holds Repository with initialize, D190 and D140; tree_entry.rs holds Entry and FileKind;
  branches.rs holds Settings; git.rs holds Revisions; export.rs adds export_revision to Repository.
- diagnostics: actionable_findings uses guidance::Guidance; source_parsers: parsing.rs holds parse with D112.
- Decision links moved: D140 and D190 to repository.rs, D179, D213 and D218 to branches.rs, D112 to parsing.rs.

## Verification

- clippy and rustfmt pass.
- Candidate measurement with architecture, file-cycle and entry-use: entry-use 96 to 80, file-cycle 0, architecture 0.

## Blockers

The checks.yaml impact-map proposal for cli_dispatch commands.rs and dispatch.rs waits for the user's approval.

## Next action

Commit VAC 17, then move the items of the structural_lint resolution rust entry into named files.
