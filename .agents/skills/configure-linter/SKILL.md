---
name: configure-linter
description: Correct worker lint configuration or add a supported rule with explicit selectors, thresholds, and repair skills.
---

# Configure lint rules

- Read the reported rule/path and tooling/worker/README.md. Use just lint-rules to inspect supported rule kinds, targets, and language applicability.
- Keep version 1 TOML with unique IDs, supported target/kind combinations, valid repository-relative globs, and literal extensions such as .rs. Directory rules do not accept extensions.
- Set warning below error; both levels require existing repository SKILL.md paths. Check ordered overrides against affected files, including overlapping selectors.
- Fix the reported configuration error and rerun just lint. Never raise thresholds or exclude files merely to hide a structural failure; a policy change needs its own current justification and recorded decision.
- When adding a rule implementation, update the registry, measurement, config documentation, and behavioral tests together. Provide an actionable repair skill before enabling it.
