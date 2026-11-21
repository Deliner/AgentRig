# State

## Focus

P171 B176 entry-use and P172 B178 entry-bypass, delivered autonomously under consent. VAC 20 lets the languages,
neutral metrics, exemptions and std forms parts use the files that own their items instead of their entries.

## Workspace

Branch: feature/entry-boundaries
Revision: 0a0a69e2e18bf7053e2207999a6f27fa1af2988f

## Progress

- VAC 20: flag_list, manifest and registry import Analysis and Code from neutral_metrics; conditions and nesting
  import from record.rs; exemptions gains attributes.rs and path_names.rs; std_forms gains call_syntax.rs for the
  method, sole argument and call search helpers; the architecture.yaml inventories describe the new files.

## Verification

- clippy, rustfmt and languages tests 183 pass.
- Candidate measurement with entry-use, file-cycle and architecture: entry-use 21 to 6, file-cycle 0, architecture 0.

## Blockers

- The checks.yaml impact-map proposal in .cache/runtime/proposal waits for the user approve.

## Next action

Commit VAC 20, then move the lint config schema, lint target selection and structure texts out of their entries.
