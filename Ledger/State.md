# State

## Focus

P171 B176 entry-use and P172 B178 entry-bypass, delivered autonomously under consent. VAC 21 moves the last items
the parts take from their entries: the lint config schema, the selected target and the structure texts.

## Workspace

Branch: feature/entry-boundaries
Revision: a7ac9032327e7473a19b0f767e926dbc0ed5fee7

## Progress

- VAC 21: lint_config splits into schema.rs and validation.rs, its entry offers Config, Rule, Thresholds and
  extension matching; Selected moves into threshold_overrides.rs; Texts moves into corpus.rs; language_support
  imports Kind from rule_identities. Decision links follow the moved markers: D016, D017, D018, D156, D198 to
  schema.rs, D156, D198, D225, D227, D228 to validation.rs, D018 selection to threshold_overrides.rs.

## Verification

- clippy, rustfmt and structural_lint tests 346 pass.
- Candidate measurement with architecture, file-cycle, entry-use and entry-bypass: entry-use 6 to 0, all four
  rules report nothing.

## Blockers

- The checks.yaml impact-map proposal in .cache/runtime/proposal waits for the user approve.

## Next action

Commit VAC 21, then add entry-use and entry-bypass to .agentrig/lint.yaml and catalog them for init.
