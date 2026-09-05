set positional-arguments := true

# What: invoke list; Why: use the configured native runtime.
list:
    @tooling/worker/run commands

# What: invoke run; Why: use the configured native runtime.
run *args:
    @tooling/worker/run run "$@"

# What: invoke read; Why: use the configured native runtime.
read *args:
    @tooling/worker/run run read "$@"

# What: invoke write; Why: use the configured native runtime.
write *args:
    @tooling/worker/run run write "$@"

# What: invoke test; Why: use the configured native runtime.
test *args:
    @tooling/worker/run run test "$@"

# What: invoke check; Why: use the configured native runtime.
check *args:
    @tooling/worker/run check "$@"

# What: invoke config-check; Why: use the configured native runtime.
config-check:
    @tooling/worker/run config-check

# What: invoke resume; Why: use the configured native runtime.
resume:
    @tooling/worker/run resume

# What: invoke report; Why: use the configured native runtime.
report:
    @tooling/worker/run report

# What: invoke lint; Why: use the configured native runtime.
lint *args:
    @tooling/worker/run lint "$@"

# What: invoke lint-rules; Why: use the configured native runtime.
lint-rules:
    @tooling/worker/run lint-rules

# What: invoke lint-config-check; Why: use the configured native runtime.
lint-config-check *args:
    @tooling/worker/run lint-config-check "$@"

# What: invoke feature-start; Why: use the configured native runtime.
feature-start *args:
    @tooling/worker/run feature-start "$@"

# What: invoke feature-merge; Why: use the configured native runtime.
feature-merge:
    @tooling/worker/run feature-merge

# What: invoke upgrade; Why: plan, apply or recover a scaffold update.
upgrade *args:
    @tooling/worker/run upgrade "$@"

# What: invoke the shared review runner; Why: use the same review contract from Just and MCP.
review *args:
    @tooling/worker/run run review -- "$@"

# What: describe a lint rule; Why: inspect supported parameters and valid examples.
lint-rule *args:
    @tooling/worker/run lint-rule "$@"
