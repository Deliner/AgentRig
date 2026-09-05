# State

## Focus

Deliver P002: portable worker capabilities, standalone lint, separate configuration, declarative setup and shared update ownership. Apply complexity-discipline.

## Workspace

Branch: feature/worker-capabilities

Revision: eb4654b

The review installation VAC is committed. The current VAC adds declarative setup using the existing bundle, manifest and atomic writer.

## Progress

Setup reads worker.toml, preserves configuration/memory and checks a temporary installation preview before writing. It detects local asset/adapter and registration conflicts, initializes a missing consumer Git repository, installs components, registers MCP, creates runtime/report directories and runs dependency diagnostics. Codex TOML comments and unrelated settings survive reconciliation. Authentication remains external. Lint defaults now follow all configured source globs and disabled lint does not require a generated policy.

## Verification

The review installation VAC passed its full staged gate. Six new setup cases and ten capability/package tests pass: fresh setup and generated MCP tool discovery, byte-identical repeat, preserved edited review/memory, and pre-write rejection of edited assets, hook conflicts and unknown capabilities. The linter identified an overlong registration function; its configuration-reading responsibility was extracted. Full staged verification is pending.

## Blockers

None. Real external-consumer model review, consumer upgrade/rollback and final acceptance remain required. Setup and integration refinements remain within P002. Reuse the existing package manifest and ownership rules.

## Next action

Finish the setup VAC and its full gate. Verify real external-consumer MCP review and the complete setup/update/rollback scenario before completing P002.
