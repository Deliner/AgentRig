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

# What: invoke consent; Why: use the configured native runtime.
consent:
    @.agentrig/bin/agentrig consent --root .

# What: invoke revoke-consent; Why: use the configured native runtime.
revoke-consent:
    @.agentrig/bin/agentrig revoke-consent --root .

# What: invoke consented-pins; Why: use the configured native runtime.
consented-pins:
    @.agentrig/bin/agentrig consented-pins --root .
