# Developing and distributing AgentRig

The repository uses the verified revision in [stable.txt](stable.txt) for development commands, Git hooks and agent
hooks. `just bootstrap` builds that exact Git tree and installs it through the existing `init` command under
`.cache/development/<revision>`. It never builds the current checkout as a fallback. Generated installations are local
and excluded from publication.

After cloning the full Git history, install the prerequisites listed in the scaffold guide, rustup and uv; on macOS
also install `flock` (for example `brew install flock`). Run `just bootstrap`, then `just resume`. Bootstrap is the
explicit shell entrypoint available before the worker exists. It registers the repository's .agentrig/hooks and rejects
a different existing hooksPath. Repeating it retains the same installation.

`just candidate --version` builds the current product through tooling/worker/build. Its cache is `.cache/worker`; native
tests always select the candidate. `just run test -- FILTER` performs focused tests. Commits run the configured
exported-index gate with [affected test groups](../../Project/change_delivery/gate/README.md) plus smoke tests. The
rig's `**` group gives every path the smoke tests, so no path is unmatched and the gate's full-suite fallback for one
never applies here; paths outside feature groups select only the smoke tests. `just feature-merge` runs
the tests affected by the whole branch; `just check` runs the full suite. `tooling/worker/run` performs no compilation.
Repository configuration, memory and canonical skill sources remain versioned project policy.

Repository lint runs twice. The native `lint` check lints with the installed runtime, so a change cannot weaken the
rule that judges it, and the `candidate-lint` command check lints with the candidate, because its analysis develops
independently of the pinned runtime. The installed runtime retains gate orchestration and snapshot isolation.

Every branch that changes Project promotes its runtime before `just feature-merge`: once its head passes the commit
gate, run `just propose-pin HEAD`. It prepares the pin as a proposal and lists the source files changed since the
pinned revision. The user then types `! just approve` in Claude Code or runs it without `!` in a terminal: it runs
the bootstrap installer with that revision and writes its full hash to stable.txt only after the installation
succeeded, so agent hooks, which run the pinned launcher, never meet an uninstalled pin. Commit the pin. Given a
revision, bootstrap only installs it, and the agent hook refuses that call; without a revision it installs the pinned
one. The pin only moves forward: the proposal and the approval refuse a revision that does not descend from the
committed pin, and the commit gate refuses an unapproved or backward pin, so a rollback is a new commit that reverts
the change. This development pin is separate from a consumer's versioned configuration.

If stable.txt is edited by hand to an uninstalled revision, agent hooks stop running. In that case run `just bootstrap`
from an ordinary terminal; the agent's PreToolUse hook can otherwise reject bootstrap itself before the recipe starts.

The CI workflow runs on Ubuntu 24.04 and macOS. It installs the Rust of `rust-toolchain.toml`, Just 1.58.0, uv 0.10.10
and cargo-about 0.9.2, prepares bubblewrap on Linux and flock on macOS, bootstraps the pinned runtime and runs
`just check`. It then runs `just release` to package the candidate executables, revision, MIT and dependency
attribution. A release tag must match the crate version. Tag builds publish the verified archives and SHA-256 files;
branch and pull-request builds retain them as CI artifacts.

For local packaging, run `cargo install --locked cargo-about --version 0.9.2 --features cli`, then
`just release` after acceptance from a clean committed checkout. The CLI feature is required to install the executable.
Outputs go to `.cache/distribution`. Packaging alone is not evidence that the development gate passed. The distributed
binaries target Linux x86_64, built on Ubuntu 24.04, and macOS arm64. A source build can target the developer's
compatible Linux or macOS environment.
