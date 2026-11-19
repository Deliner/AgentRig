# What: invoke check; Why: use the configured native runtime.
check *args:
    @.agentrig/bin/agentrig check --root . "$@"

# What: invoke config-check; Why: use the configured native runtime.
config-check:
    @.agentrig/bin/agentrig config-check --root .
