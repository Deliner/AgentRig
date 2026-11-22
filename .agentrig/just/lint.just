# What: invoke candidate lint; Why: verify the architecture supported by the product under development.
lint *args:
    @.agentrig/bin/agentrig run lint --root . -- "$@"

# What: invoke lint-rules; Why: use the configured native runtime.
lint-rules:
    @.agentrig/bin/agentrig lint-rules --root .

# What: invoke lint-config-check; Why: use the configured native runtime.
lint-config-check *args:
    @.agentrig/bin/agentrig lint-config-check --root . "$@"

# What: describe a lint rule; Why: inspect supported parameters and valid examples.
lint-rule *args:
    @.agentrig/bin/agentrig lint-rule --root . "$@"

# What: explain lint selection; Why: inspect exclusion reasons and effective thresholds.
lint-explain *args:
    @.agentrig/bin/agentrig lint-explain --root . "$@"

# What: show measured edges of a directory; Why: trace the dependents and cycles architecture findings count.
lint-edges *args:
    @.agentrig/bin/agentrig lint-edges --root . "$@"

# What: print a directory as a tree of names; Why: see the structure of the lint inventory without file contents.
lint-tree *args:
    @.agentrig/bin/agentrig lint-tree --root . "$@"

# What: score how the names of a Rust tree show what each directory does; Why: find where the structure is unclear.
lint-structure *args:
    @.agentrig/bin/agentrig lint-structure --root . "$@"
