# Developing and distributing AgentRig

The repository uses the verified revision in [stable.txt](stable.txt) for development commands, Git hooks and agent hooks. `just bootstrap` builds that exact Git tree and installs it through the existing `init` command under `.cache/development/<revision>`. It never builds the current checkout as a fallback. Generated installations are local and excluded from publication.

After cloning the full Git history, install the prerequisites listed in the scaffold guide, Rust 1.98.1 and uv. Run `just bootstrap`, then `just resume`. Bootstrap is the explicit shell entrypoint available before the worker exists. It registers the repository's .githooks and rejects a different existing hooksPath. Repeating it retains the same installation.

`just candidate --version` builds the current product through tooling/worker/build. Its cache is `.cache/worker`; native tests always select the candidate. `just run test -- PATH` performs focused tests. Commits run the complete exported-index gate, and `just feature-merge` verifies integration. `tooling/worker/run` performs no compilation. Repository configuration, memory and canonical skill sources remain versioned project policy.

To promote a development runtime, first accept and commit the candidate through the full gate. In a subsequent feature change, put its full commit hash in stable.txt and run `just bootstrap`. Retained installation directories allow an explicit pin rollback. This development pin is separate from a consumer's versioned configuration and `upgrade plan/apply/rollback` workflow.

The CI workflow installs Rust 1.98.1, Just 1.58.0, uv 0.10.10, cargo-about 0.9.2 and the official native Codex 0.153.4 package with its code-mode host. The latter is required by setup/doctor acceptance tests; CI does not supply model credentials. It prepares bubblewrap and the systemd user manager, bootstraps the pinned runtime and runs `just check`. It then runs `just release` to package the candidate executables, revision, MIT and dependency attribution. A release tag must match the crate version. Tag builds publish the verified archive and SHA-256 file; branch and pull-request builds retain them as CI artifacts.

For local packaging, install `cargo-about` 0.9.2 and run `just release` after acceptance from a clean committed checkout. Outputs go to `.cache/distribution`. Packaging alone is not evidence that the development gate passed. The distributed binaries target Linux x86_64; releases are built on Ubuntu 24.04. A source build can target the developer's compatible Linux environment.
