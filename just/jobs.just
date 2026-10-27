# What: list managed runs; Why: inspect ongoing commands and recovered state.
jobs:
    @tooling/worker/run jobs

# What: inspect a managed run; Why: verify process liveness from its OS identity.
job-status *args:
    @tooling/worker/run job-status "$@"

# What: read command output; Why: recover logs after a session disconnects.
job-logs *args:
    @tooling/worker/run job-logs "$@"

# What: start a background command; Why: preserve a managed run across sessions.
job-start *args:
    @tooling/worker/run job-start "$@"

# What: stop an owned run; Why: clean its contained descendants without affecting other owners.
job-stop *args:
    @tooling/worker/run job-stop "$@"

# What: clean completed work's runs; Why: retain other owners and shared services.
job-cleanup *args:
    @tooling/worker/run job-cleanup "$@"
