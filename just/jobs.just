# What: list managed runs; Why: inspect ongoing commands and recovered state.
jobs:
    @.agentrig/bin/agentrig jobs --root .

# What: inspect a managed run; Why: verify process liveness from its OS identity.
job-status *args:
    @.agentrig/bin/agentrig job-status --root . "$@"

# What: read command output; Why: recover logs after a session disconnects.
job-logs *args:
    @.agentrig/bin/agentrig job-logs --root . "$@"

# What: start a background command; Why: preserve a managed run across sessions.
job-start *args:
    @.agentrig/bin/agentrig job-start --root . "$@"

# What: stop an owned run; Why: clean its contained descendants without affecting other owners.
job-stop *args:
    @.agentrig/bin/agentrig job-stop --root . "$@"

# What: clean completed work's runs; Why: retain other owners and shared services.
job-cleanup *args:
    @.agentrig/bin/agentrig job-cleanup --root . "$@"
