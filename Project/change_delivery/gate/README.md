# Gate checks and affected tests

`check` runs the configured checks over the worktree, `--staged` over the exported index and `--revision REV` over a
commit; `--only ID` selects one check. The pre-commit hook runs `check --staged`, and `feature-merge` runs the gate on
the integration candidate. Lint, memory and other non-command checks always run in full.

A staged run keeps cargo builds between commits. Each exported file whose staged content is unchanged since the previous
staged run gets that run's modification time, any other file the current time, and the commands build into
`<runtime>/staged/target` through `CARGO_TARGET_DIR`; worktree builds never write there, so an old time always means the
content that target was built from. Staged runs of one project wait for each other.

A command check runs its whole command unless it declares `affected` groups:

```yaml
- id: tests
  kind: command
  command: test
  skill: .agents/skills/repair/SKILL.md
  affected:
  - include: [src/lint/**, tests/lint/**]
    targets: [tests/lint]
  - include: [src/hooks/**, tests/hooks/**]
    targets: [tests/hooks]
  - include: [docs/**]
    targets: []
```

On commit, groups match the staged paths against the previous commit; on merge, after any rebase, every path changed
between the base branch and the feature head. Added, deleted and both sides of renamed paths participate. When every
changed path matches a group, the deduplicated union of targets is appended to the command as literal arguments,
interpreted from its working directory, and an empty union skips the command. Any unmatched path, or no groups, keeps
the full command; `check` and `check --revision` always do. This is an explicit impact map, not dependency or coverage
analysis: include consumer tests when a change can affect them.

The console prints `SELECT [id]` with the chosen targets. Narrowed or skipped evidence is saved with `selective: true`
and never sets `full_gate_passed`; `feature-merge` needs only its selected gate to pass on unchanged inputs. The command
must accept arguments, includes must be nonempty valid globs and targets nonempty NUL-free strings.

Keep pytest targets nonoverlapping: a file together with a node inside it narrows collection to that node, and
directory and file overlaps can drop cases.

`config-check` reports, as errors, the gaps a map leaves for a check whose command runs `cargo … test` or `pytest`;
other commands are not inspected. Among the inventory files with that language's extension (`.rs` or `.py`) that the
check's include matches, it names
each file that matches only groups without targets, because changing it selects no tests, and each test that no target
selects: a `test_*.py` file outside every target path, resolved from the command's working directory, or a Rust file
containing `#[test]` whose module path neither contains a target nor ends in whole segments that begin one. The path
follows `mod` declarations and `#[path]` from the crate roots of the manifest the command names, so `game::tests::`
selects the tests of `native::host::game`; a file that no crate root reaches is not reported. Repair by adding the
path to a group whose targets exercise it, or by mapping it to `::` or `.` when its consumers are unknown. A file that
matches no group is not reported, since it already runs the whole command.
