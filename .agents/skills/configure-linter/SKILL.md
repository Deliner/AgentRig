---
name: configure-linter
description: Correct worker lint configuration or add a supported rule with explicit selectors, thresholds, and repair skills.
---

# Configure lint rules

- Read the reported rule/path and tooling/worker/README.md. Use just lint-rules to inspect supported rule kinds, targets, and language applicability. Run just lint-config-check for configuration-only validation.
- Keep version 1 TOML with unique IDs, supported target/kind combinations, valid repository-relative globs, and literal extensions such as .rs. Directory rules do not accept extensions.
- Numeric rules need warning, error, or both; when both exist warning is below error. named-if-condition instead needs level = "warning" or "error" and no thresholds/overrides. Both repair skill paths must exist. Check ordered numeric overrides against affected files.
- enabled defaults to true; false skips execution but does not permit malformed configuration. Include/exclude and extensions define the selected scope. Without an extension filter, selected unsupported files fail rather than being skipped.
- Syntax rules currently support Rust (.rs) and Python (.py/.pyi). A selector does not implement a new language handler. Parse errors block even when rule findings only warn.
- Fix the reported configuration error and rerun just lint. Never raise thresholds or exclude files merely to hide a structural failure; a policy change needs its own current justification and recorded decision.
- When adding a rule implementation, update the registry, measurement, config documentation, and behavioral tests together. Provide an actionable repair skill before enabling it.
