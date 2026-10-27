# What: invoke list; Why: use the configured native runtime.
list:
    @tooling/worker/run commands

# What: invoke run; Why: use the configured native runtime.
run *args:
    @tooling/worker/run run "$@"

# What: invoke read; Why: use the configured native runtime.
read *args:
    @tooling/worker/run run read "$@"

# What: invoke write; Why: use the configured native runtime.
write *args:
    @tooling/worker/run run write "$@"

# What: run temporary code; Why: keep one-off work in its task area with recorded output.
scratch *args:
    @tooling/worker/run scratch "$@"

# What: invoke test; Why: use the configured native runtime.
test *args:
    @tooling/worker/run run test "$@"
