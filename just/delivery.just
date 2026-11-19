# What: invoke resume; Why: use the configured native runtime.
resume:
    @.agentrig/bin/agentrig resume

# What: invoke report; Why: use the configured native runtime.
report:
    @.agentrig/bin/agentrig report

# What: invoke feature-start; Why: use the configured native runtime.
feature-start *args:
    @.agentrig/bin/agentrig feature-start "$@"

# What: invoke feature-merge; Why: use the configured native runtime.
feature-merge:
    @.agentrig/bin/agentrig feature-merge
