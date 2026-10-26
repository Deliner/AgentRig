# Developing and distributing AgentRig

The repository uses the verified revision in [stable.txt](stable.txt) for development commands, Git hooks and agent
hooks. `just bootstrap` builds that exact Git tree and installs it through the existing `init` command under
`.cache/development/<revision>`. It never builds the current checkout as a fallback. Generated installations are local
and excluded from publication.

After cloning the full Git history, install the prerequisites listed in the scaffold guide, Rust 1.98.1 and uv; on macOS
also install `flock` (for example `brew install flock`). Run `just bootstrap`, then `just resume`. Bootstrap is the
explicit shell entrypoint available before the worker exists. It registers the repository's .githooks and rejects a
different existing hooksPath. Repeating it retains the same installation.

`just candidate --version` builds the current product through tooling/worker/build. Its cache is `.cache/worker`; native
tests always select the candidate. `just run test -- PATH` performs focused tests. Commits run the configured
exported-index gate with [affected test groups](../worker/CHECKS.md) plus smoke tests. Paths outside feature groups
select only the smoke tests, never an automatic full-suite fallback. `just feature-merge` runs the tests affected by the
whole branch; `just check` runs the full suite. `tooling/worker/run` performs no compilation. Repository configuration,
memory and canonical skill sources remain versioned project policy.

Repository lint also invokes the candidate through the configured command check,
because its architecture analysis develops independently of the pinned runtime.
The installed runtime retains gate orchestration and snapshot isolation.

To promote a development runtime, first commit the candidate and pass `just check` on it. In a subsequent feature
change, put its full commit hash in stable.txt and run `just bootstrap`. Retained installation directories allow an
explicit pin rollback. This development pin is separate from a consumer's versioned configuration.

If agent hooks use the pinned launcher, changing the pin to an uninstalled revision also prevents those hooks from
running. In that case run `just bootstrap` from an ordinary terminal; the agent's PreToolUse hook can otherwise reject
bootstrap itself before the recipe starts.

The CI workflow runs on Ubuntu 24.04 and macOS. It installs Rust 1.98.1, Just 1.58.0, uv 0.10.10 and cargo-about 0.9.2,
prepares bubblewrap on Linux and flock on macOS, bootstraps the pinned runtime and runs `just check`. It then runs
`just release` to package the candidate executables, revision, MIT and dependency attribution. A release tag must match
the crate version. Tag builds publish the verified archives and SHA-256 files; branch and pull-request builds retain
them as CI artifacts.

For local packaging, run `cargo +1.98.1 install --locked cargo-about --version 0.9.2 --features cli`, then
`just release` after acceptance from a clean committed checkout. The CLI feature is required to install the executable.
Outputs go to `.cache/distribution`. Packaging alone is not evidence that the development gate passed. The distributed
binaries target Linux x86_64, built on Ubuntu 24.04, and macOS arm64. A source build can target the developer's
compatible Linux or macOS environment.
