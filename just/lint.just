# What: invoke candidate lint; Why: verify the architecture supported by the product under development.
lint *args:
    @tooling/worker/run run lint -- "$@"

# What: invoke lint-rules; Why: use the configured native runtime.
lint-rules:
    @tooling/worker/run lint-rules

# What: invoke lint-config-check; Why: use the configured native runtime.
lint-config-check *args:
    @tooling/worker/run lint-config-check "$@"

# What: record the lint baseline; Why: accept existing errors and fail only when they grow.
lint-baseline *args:
    @tooling/worker/run lint-baseline "$@"

# What: describe a lint rule; Why: inspect supported parameters and valid examples.
lint-rule *args:
    @tooling/worker/run lint-rule "$@"

# What: explain lint selection; Why: inspect exclusion reasons and effective thresholds.
lint-explain *args:
    @tooling/worker/run lint-explain "$@"
