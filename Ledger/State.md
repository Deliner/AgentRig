# State

## Focus

P203 (B138): each Clippy profile lint leads to the skill that repairs it. VAC 5 of 5: the expect-the-panic skill
says how an exception keeps `clippy::should_panic_without_expect` through justify-allow, as AC1 requires of each
profile skill.

## Workspace

- Branch: feature/clippy-lint-skills
- Revision: 69b8885
- Changed: Project/shared/skills/references/false-pass/expect-the-panic/SKILL.md, its Clippy profile section.
- Unstaged: tooling/distribution/stable.txt pins 69b8885 under the consent of the user; the pin of the VAC 5 head
  replaces it and is committed with the archive.

## Progress

- VAC 1 29508a5, VAC 2 56c4d81, VAC 3 a8dbc2a and VAC 4 69b8885 are committed.
- `just propose-pin HEAD` pinned 69b8885 under the consent of the user.
- Review for the archive: 26 of the 27 profile skills say when an exception keeps their lint, through justify-allow
  or that none does; expect-the-panic named only the exception of its rig rule, which does not lift a Clippy
  finding. VAC 5 adds the justify-allow form to its Clippy profile section.

## Verification

- 69b8885: the commit gate and the gate of propose-pin passed, 645 Rust tests each.
- VAC 5 changes skill text only; the commit gate runs lint and the affected tests.

## Blockers

None.

## Next action

Commit VAC 5, run `just propose-pin HEAD` under the consent of the user, archive P203 and B138 with that pin, run
`just check --only memory`, commit, then run `just feature-merge` and compact.
