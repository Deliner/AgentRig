# What: invoke resume; Why: use the configured native runtime.
resume:
    @tooling/worker/run resume

# What: invoke report; Why: use the configured native runtime.
report:
    @tooling/worker/run report

# What: invoke feature-start; Why: use the configured native runtime.
feature-start *args:
    @tooling/worker/run feature-start "$@"

# What: invoke feature-merge; Why: use the configured native runtime.
feature-merge:
    @tooling/worker/run feature-merge
