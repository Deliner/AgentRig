# What: invoke candidate lint; Why: verify the architecture supported by the product under development.
lint *args:
    @.agentrig/bin/agentrig run lint -- "$@"

# What: invoke lint-rules; Why: use the configured native runtime.
lint-rules:
    @.agentrig/bin/agentrig lint-rules

# What: invoke lint-config-check; Why: use the configured native runtime.
lint-config-check *args:
    @.agentrig/bin/agentrig lint-config-check "$@"

# What: record the lint baseline; Why: accept existing errors and fail only when they grow.
lint-baseline *args:
    @.agentrig/bin/agentrig lint-baseline "$@"

# What: describe a lint rule; Why: inspect supported parameters and valid examples.
lint-rule *args:
    @.agentrig/bin/agentrig lint-rule "$@"

# What: explain lint selection; Why: inspect exclusion reasons and effective thresholds.
lint-explain *args:
    @.agentrig/bin/agentrig lint-explain "$@"

# What: show measured edges of a directory; Why: trace the dependents and cycles architecture findings count.
lint-edges *args:
    @.agentrig/bin/agentrig lint-edges "$@"

# What: print a directory as a tree of names; Why: see the structure of the lint inventory without file contents.
lint-tree *args:
    @.agentrig/bin/agentrig lint-tree "$@"

# What: score how the names of a Rust tree show what each directory does; Why: find where the structure is unclear.
lint-structure *args:
    @.agentrig/bin/agentrig lint-structure "$@"
