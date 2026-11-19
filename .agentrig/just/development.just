# What: install a runtime revision, then pin it, or the pinned one; Why: work without any installed runtime.
bootstrap *args:
    @bash tooling/distribution/bootstrap.sh "$@"

# What: build and invoke the candidate; Why: test product changes separately from the installed runtime.
candidate *args:
    @tooling/worker/run run candidate -- "$@"

# What: package candidate binaries and licenses; Why: produce the platform release through the command catalog.
release:
    @tooling/worker/run run release
