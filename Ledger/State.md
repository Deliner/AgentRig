# State

## Focus

P171 B176 entry-use and P172 B178 entry-bypass, delivered autonomously under consent. VAC 22 is the second phase:
the rig lint.yaml adds the entry-use and entry-bypass rules after file-cycle, and init catalogs both kinds.

## Workspace

Branch: feature/entry-boundaries
Revision: bfbe7f580db7ccbff4f6d205ca539f2f41756f41

## Progress

- lint.yaml appends the two rules only, keeping committed rules unchanged and in order, per D226.
- project_setup/bundle/lint.rs catalogs EntryUse and EntryBypass, marked D227 and D228; both decisions link the
  rig rule and the init rule.
- The shipped architecture test now builds alpha and beta as packages with __init__.py entries, since init's
  entry-bypass reports the old import of beta/api.py past a missing entry.

## Verification

- project_setup tests 58 pass, including I023 rig parity; clippy and rustfmt pass.
- Installed lint at pin 452a44b with the entry rules on: no findings.

## Blockers

- The checks.yaml impact-map proposal in .cache/runtime/proposal waits for the user approve.

## Next action

Commit VAC 22, then align refactor-large-directory, anchor-at-crate and the architecture README with the rules.
