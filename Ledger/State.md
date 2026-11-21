# State

## Focus

P171 B176 entry-use and P172 B178 entry-bypass, delivered autonomously under consent. VAC 23 aligns the skills
and the architecture README with the entry rules, per B176 AC5 and B178 AC5.

## Workspace

Branch: feature/entry-boundaries
Revision: 2a00c72ade97ae629b05d7f73a5977bfc8a13d06

## Progress

- refactor-large-directory: the entry is the table of contents, own code uses the owning files, other
  directories go through the entry, group: true, and a reexport is API only as the directory's promise.
- anchor-at-crate: the crate path names the owning file, the parent reexport step is replaced by a ban on
  naming an ancestor entry's item; its descriptions in architecture.yaml follow.
- architecture README: group: true described, the child using a parent-owned type removed, 70 lines.
- move-out-of-entry: choose the file so that no file cycle appears.

## Verification

- Installed lint on the worktree passes, README within 70 nonblank lines; project_setup tests 58 pass.

## Blockers

- The checks.yaml impact-map proposal in .cache/runtime/proposal waits for the user approve.

## Next action

Commit VAC 23, then verify each AC of B176 and B178, archive P171, P172, B176, B178 and pin under consent.
