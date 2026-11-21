# State

## Focus

P171 (B176 entry-use) and P172 (B178 entry-bypass), one branch from master, autonomous under the user's consent
(D226). VAC 7: inner structural_lint parts go through their entries or declared groups.

## Workspace

Branch: feature/entry-boundaries
Revision: e3701e0d9434fd77a2f17aed5d5cc67f990a50a8

## Progress

- VACs 1 to 6 and the consented pin of dae94a4 are committed.
- `rule_set`, `targeting` and `architecture/resolution` declare `group: true`; the resolution result moved
  to `resolution/resolved.rs` and the path normalization to the Rust module tree, so the entry only declares.
- `languages` exports `program_language` and `program_languages`; `exemptions` exports its async checks;
  `extraction/rust` exports `prelude_origin`; the D227 resolution link follows `resolved.rs`.
- A parent cannot reexport a child that uses a file of the parent directory: that is a directory cycle, so
  `architecture` keeps `contract_check` public until its shared records move out of the parent.
- Scratch measurement: entry-use 225 to 219, entry-bypass 277 to 171, no architecture finding.

## Verification

- `just test structural_lint::` passed 345; `just run rustfmt` passed.

## Blockers

None.

## Next action

Commit this VAC, then move the `architecture` records and core files so `contract_check` can go behind the entry.
