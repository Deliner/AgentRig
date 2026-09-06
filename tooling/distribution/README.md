# Developing and distributing AgentRig

The repository uses the verified revision in [stable.txt](stable.txt) for development commands, Git hooks and agent hooks. `just bootstrap` builds that exact Git tree and installs it through the existing `init` command under `.cache/development/<revision>`. It never builds the current checkout as a fallback. Generated installations are local and excluded from publication.

After cloning the full Git history, install the prerequisites listed in the scaffold guide, Rust 1.98.1 and uv. Run `just bootstrap`, then `just resume`. Bootstrap is the explicit shell entrypoint available before the worker exists. Repeating it retains the same installation.

`just candidate --version` builds the current product through tooling/worker/build. Its cache is `.cache/worker`; native tests always select the candidate. `just run test -- PATH` performs focused tests. Commits run the complete exported-index gate, and `just feature-merge` verifies integration. `tooling/worker/run` performs no compilation. Repository configuration, memory and canonical skill sources remain versioned project policy.

To promote a development runtime, first accept and commit the candidate through the full gate. In a subsequent feature change, put its full commit hash in stable.txt and run `just bootstrap`. Retained installation directories allow an explicit pin rollback. This development pin is separate from a consumer's versioned configuration and `upgrade plan/apply/rollback` workflow.
