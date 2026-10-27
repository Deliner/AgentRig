# What: invoke check; Why: use the configured native runtime.
check *args:
    @tooling/worker/run check "$@"

# What: invoke config-check; Why: use the configured native runtime.
config-check:
    @tooling/worker/run config-check
