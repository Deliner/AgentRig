# State

## Focus

P184 (B191): an existing absolute path is resolved by one `canonicalize` call instead of one per component.

## Workspace

Branch: feature/fast-resolve
Revision: 29a41afb05e828b06db21a11d24b2e6245c49e1d

## Progress

VAC 2: `resolve` in `Project/shared/paths/symlinks.rs` returns `canonicalize` for an absolute path that exists; a
relative path or a failed `canonicalize` takes the former walk. The resolver test adds an existing path through a
relative link and `..`.

## Verification

Full suite: 616 passed. `just lint`: clean. Release candidate on AgentRig, three warm runs with the step probe, since
removed: memory check 742, 783 and 734 ms on `15d8bee` against 472, 475 and 467 ms.

## Blockers

None.

## Next action

Commit VAC 2 through the hook, then write Delivery and archive P184 and B191.
