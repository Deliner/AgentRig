# Independent scaffold examples

These are consumer projects. Copy either `python/` or `rust/` outside the worker repository. Neither contains worker source or depends on its development gate. Supply an absolute path to a built `discipline-worker` binary as `WORKER_BINARY`; initialization copies that binary and embeds its configuration, memory and skills into the consumer.

For Python, install Python 3 and pytest. For Rust, install Cargo and a Rust compiler supporting edition 2024. Both examples use Linux, Git, Just and bubblewrap. `doctor` diagnoses executables, sandbox availability and hook registration; it does not install tools.

## Python

From the copied Python directory:

```sh
git init -b trunk
git config user.name "Example Developer"
git config user.email "example@example.invalid"
git add .
git commit -m "Initial example"
git switch -c topic/bootstrap
"$WORKER_BINARY" init --language python --source application --memory notes --skills guides --base trunk --prefix topic/
```

## Rust

From the copied Rust directory:

```sh
git init -b release
git config user.name "Example Developer"
git config user.email "example@example.invalid"
git add .
git commit -m "Initial example"
git switch -c change/bootstrap
"$WORKER_BINARY" init --language rust --source crates/engine --memory knowledge --skills policies --base release --prefix change/
```

Generated defaults enforce named conditions, function length and parameter limits as errors for both Rust and Python. Project-specific selectors and thresholds remain configurable in `.worker/lint.toml`.

## Exercise either installation

```sh
.worker/bin/discipline-worker --version
just setup
just config-check
just resume
just run test
just check
git add .
just check --staged
printf '%s' '{"hook_event_name":"SessionStart","session_id":"example"}' | .worker/bin/discipline-worker hook
git commit -m "Attach portable scaffold"
just feature-merge
just feature-start next
```

`git commit` invokes the staged gate. `feature-merge` checks the integration candidate and retains the bootstrap branch. The initial commit is made before hook installation so bootstrap does not require bypassing an installed guard. Later development uses the configured branch prefix.

The source marker can be linked from the generated invariant index. For Python, add `[I001](Invariants/001.md)`, a predicate, and `[test_doubles](../application/test_sample.py)` as one index row; create `notes/Invariants/001.md` with nonempty `Predicate` and `Oracle` sections. Add to `worker.toml`:

```toml
[oracles.I001]
check = "tests"
runner = "pytest"
target = "application/test_sample.py::test_doubles"
```

For Rust use `knowledge/Invariants/001.md`, `[doubles](../crates/engine/src/lib.rs)` and:

```toml
[oracles.I001]
check = "tests"
runner = "cargo"
target = "tests::doubles"
```

Run `memory-check` to validate the link and discover the exact test target. Run `check` to execute the test. Discovery alone does not verify the predicate.

The native scaffold integration tests exercise both installations with their own source/memory/skills paths, branch names, severity settings and oracles. They also reject an unsupported shell-language selector, repair it through configuration, exercise real Git hooks, and show that an unrelated source marker or missing test target cannot validate an invariant.

See [observed latency](LATENCY.md) for first-process and repeated-process measurements and the reproducible measurement command.


## Enable isolated review

Add the review capability to the consumer's worker.toml:

```toml
[capabilities.review]
config = ".worker/review/config/review.toml"
```

Run `just setup`. It installs the stock review resources and review-project skill,
registers MCP, and checks dependencies. Adapt the installed project visibility
patterns to `application/**` or `crates/engine/**` for these examples. Edit the
machine contract, reviewer selection, models and prompts for the desired review;
then run `just review config-check`. Commit the intended candidate and call the
configured MCP tool with the consumer root and the base/candidate Git boundary.

Codex authentication stays outside the project. Supply CODEX_HOME and, when
needed, REVIEW_CODEX_BIN in the environment from which the MCP client starts.
Restart the MCP connection after changing its tool configuration. Repeated setup
preserves settings and reports conflicting adapters instead of overwriting them.

The separate `discipline-lint --root OTHER_PROJECT --config POLICY --json` command
needs no worker installation in OTHER_PROJECT. See the
[lint guide](../README.md) for external repair-skill resources.
See [portable delivery verification](PORTABILITY.md) for the observed setup,
real MCP review, update and rollback scenario.
