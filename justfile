set positional-arguments := true

runner := "python3 tooling/command_runner.py"

# What: list agent commands; Why: make the command surface discoverable by default.
default:
    @{{runner}} run default

# What: list agent commands; Why: let agents inspect the allowed command surface explicitly.
list:
    @{{runner}} run list

# What: run arbitrary argv in a read-only sandbox; Why: permit safe repository inspection.
read *args:
    @{{runner}} run read "$@"

# What: run arbitrary argv with normal repository access; Why: make mutations explicit.
write *args:
    @{{runner}} run write "$@"

# What: show concise Git status; Why: reveal pending changes without unrelated detail.
status:
    @{{runner}} run status

# What: show a Git diff; Why: support focused review of repository changes.
diff *args:
    @{{runner}} run diff "$@"

# What: run the complete repository gate; Why: verify the current working tree.
check:
    @{{runner}} run check

# What: run the gate against the staged tree; Why: verify exactly what a commit will contain.
check-staged:
    @{{runner}} run check-staged

# What: run pytest; Why: support focused behavioral verification.
test *args:
    @{{runner}} run test "$@"

# What: report command usage; Why: expose evidence for command-surface maintenance.
report:
    @{{runner}} run report

# What: reset the command-review baseline; Why: acknowledge an evidence-based command review.
review-commands:
    @{{runner}} run review-commands
