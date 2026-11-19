# What: invoke resume; Why: use the configured native runtime.
resume:
    @.agentrig/bin/agentrig resume --root .

# What: invoke report; Why: use the configured native runtime.
report:
    @.agentrig/bin/agentrig report --root .

# What: invoke feature-start; Why: use the configured native runtime.
feature-start *args:
    @.agentrig/bin/agentrig feature-start --root . "$@"

# What: invoke feature-merge; Why: use the configured native runtime.
feature-merge:
    @.agentrig/bin/agentrig feature-merge --root .

# What: invoke approve; Why: use the configured native runtime.
approve:
    @.agentrig/bin/agentrig approve --root .
