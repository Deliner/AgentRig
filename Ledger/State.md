# State

## Focus

Deliver all of P004 under the AgentRig goal in .tmp/agentrig-yaml-environments-plan.md, applying complexity-discipline throughout.

## Workspace

Branch: feature/agentrig-environments

Revision: b06927c

Started from clean master. P003 is integrated; its earlier pending-integration State was stale. No merge/rebase or upgrade is active.

## Progress

YAML, AgentRig naming, service placement and the composition resolver are committed. Current VAC adds setup --preview using the existing preparation, validation and reconciliation path. It reports actual file changes and modes, Git/Codex registration, runtime directories and declared dependencies without installation or dependency execution. The generated Just adapter forwards preview arguments. Composition still leaves resource strings untouched; external-config setup, resource installation, fixed-input updates, interactive init, custom delegate environments and complete migration acceptance remain outstanding in P004.

## Verification

Last committed full gate passed 305 native, 27 review/YAML and 17 worker Rust tests. All 16 setup tests pass; three preview-focused checks pass after final path-resolution changes and formatting. New checks prove consumer bytes remain unchanged during preview, reported hashes/modes match subsequent installation, repeated preview has no file changes, Just forwards --preview, conflicts preserve originals, and unrelated credential values are omitted. Lint has no blockers; current full staged gate remains pending. Real 0.2.0 baseline binary/digest remains .tmp/agentrig-baseline-0.2.0, pinned to a3e4b4f4d83538e21fecc7ed30393cdcf576ed30.

## Blockers

None observed.

## Next action

Commit the setup-preview VAC through the full staged gate and correct any failures. Connect composition to this same preparation path with explicit external --config input and selected target root. Materialize referenced resources from recorded origins, preserve project-relative selectors, include configuration/resource digests and explicit updates, and share resource assembly with configured delegates. Complete interactive init, custom hooks/skills/MCP and migration acceptance including external resources and selected delegate ownership. Verify every P004 criterion before integration; these preview commands alone do not complete the goal.
