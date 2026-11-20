# What: install a runtime revision, or the pinned one; Why: work without an installed runtime and install approved pins.
bootstrap *args:
    @bash tooling/distribution/bootstrap.sh "$@"

# What: move the runtime pin; Why: only the user's approval or standing consent moves the runtime that judges commits.
propose-pin *args:
    @tooling/worker/run run propose-pin -- "$@"

# What: build and invoke the candidate; Why: test product changes separately from the installed runtime.
candidate *args:
    @tooling/worker/run run candidate -- "$@"

# What: package candidate binaries and licenses; Why: produce the platform release through the command catalog.
release:
    @tooling/worker/run run release
