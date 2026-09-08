# Selective commit tests

Command checks run their full configured command by default. A check may declare
`affected` groups to append selected test targets during `check --staged`:

```yaml
commands:
  test:
    argv: [python3, -m, pytest]
    accepts_args: true
checks:
  - id: tests
    kind: command
    command: test
    skill: .agents/skills/repair/SKILL.md
    affected:
      - include: [src/lint/**, tests/lint/**]
        targets: [tests/lint]
      - include: [src/delegate/**, tests/delegate/**]
        targets: [tests/delegate]
      - include: [docs/**]
        targets: []
```

Groups match project-relative staged paths against the previous commit. Added
and deleted paths participate; renames include both old and new paths. Unstaged
changes do not select groups. The map itself comes from the exported index.

If every changed path matches at least one group, the union of target strings is
appended to the command as literal arguments, with duplicates removed. Targets
are interpreted by that command from its configured working directory. This is
an explicit impact map, not inferred dependency or coverage analysis. Include
consumer tests when a module change can affect them.

An empty target union skips the command. Use empty groups only for paths that
require no tests. Any unmatched changed path retains the full command. Leave
shared components, test infrastructure and other broadly affecting paths unmapped
when the full suite is required. No groups also retains the full command.

`check`, `check --revision REV` and the gate invoked by `feature-merge` run full
commands. Other checks, including lint, memory and formatting, retain their
existing behavior. `include` still controls whether a check applies to the
project inventory; it is not a changed-path selector.

The console prints `SELECT [check-id]` and the selected target arguments.
Saved check evidence sets `selective: true` whenever a command is narrowed or
skipped by this map. Such a result never sets `full_gate_passed`, even when it
passes. An unmatched-path fallback can establish full evidence. The normal
staged rerun command applies the same staged map to the current index.

The command must accept arguments. Group include lists must be nonempty valid
globs, and each target must be a nonempty, NUL-free string. Invalid maps fail
configuration validation. Existing configurations without groups retain full
verification.

AgentRig's own repository uses a catch-all group, `include: ['**']`, with the fast
configuration test file, alongside its affected feature groups. Thus every commit path
has a selection: an unknown path runs smoke tests rather than the full suite.
All lint source paths select lint consumers, including strict defaults and memory
syntax checks. Delegation selects its tests and setup consumers. Scaffold changes
select scaffold/upgrade tests; the long complete-project integration scenarios
are selected on commit when their test file or independent examples change. Merge runs all
configured tests. Keep feature mappings current when ownership moves; smoke-only
verification does not establish that every affected behavior was tested.

Keep pytest targets nonoverlapping: use the same whole-file or whole-directory
target in every group that needs it. Passing both a file and a node ID inside it
can narrow collection to that node; directory/file overlaps can also drop cases.
Check combined collection against the union of individually collected targets
when changing these groups.

Repository lint is a command check invoking the candidate through
`tooling/worker/build lint`. The pinned runtime still orchestrates the gate and
its exported index; the candidate analyzes syntax added after that runtime was
pinned. Lint always checks its configured tree, without an affected-path subset.
`just lint` uses the same configured command. Consumer defaults retain their
built-in lint check against the installed consumer binary.
