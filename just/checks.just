# What: invoke check; Why: use the configured native runtime.
check *args:
    @.agentrig/bin/agentrig check "$@"

# What: invoke config-check; Why: use the configured native runtime.
config-check:
    @.agentrig/bin/agentrig config-check
