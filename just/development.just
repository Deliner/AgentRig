# What: install the pinned development runtime; Why: bootstrap without relying on candidate code.
bootstrap:
    @bash tooling/distribution/bootstrap.sh

# What: build and invoke the candidate; Why: test product changes separately from the installed runtime.
candidate *args:
    @tooling/worker/run run candidate -- "$@"

# What: package candidate binaries and licenses; Why: produce the platform release through the command catalog.
release:
    @tooling/worker/run run release
