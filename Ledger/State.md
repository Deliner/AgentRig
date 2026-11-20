# State

## Focus

P171 (B176 entry-use) and P172 (B178 entry-bypass), one branch from master, autonomous under the user's consent
(D226). VAC 4: outside its parent, the name of a private Rust module means the item its parent reexports under it.

## Workspace

Branch: feature/entry-boundaries
Revision: f9e4f3cb2f335c4c934903f81407ffcfd9143a81

## Progress

- VACs 1 to 3 and the consented pin of dae94a4 are committed.
- Extraction records whether a `mod` declaration is private; the resolver walks into a private module only from a
  path written inside its parent, so `mod take; pub use take::take;` resolves to the entry from outside.
- The scratch measurement fell from 1434 to 1428 entry-bypass findings; entry-use stays at 225.

## Verification

- `just test structural_lint::` passed 345; `just run rustfmt` passed.

## Blockers

None.

## Next action

Commit this VAC, then route the uses of `shared` parts through their entries and declare `shared` a group.
