# What: invoke list; Why: use the configured native runtime.
list:
    @.agentrig/bin/agentrig commands --root .

# What: invoke run; Why: use the configured native runtime.
run *args:
    @.agentrig/bin/agentrig run --root . "$@"

# What: invoke read; Why: use the configured native runtime.
read *args:
    @.agentrig/bin/agentrig run read --root . "$@"

# What: invoke write; Why: use the configured native runtime.
write *args:
    @.agentrig/bin/agentrig run write --root . "$@"

# What: invoke test; Why: use the configured native runtime.
test *args:
    @.agentrig/bin/agentrig run test --root . "$@"
