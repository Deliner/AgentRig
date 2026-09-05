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

In the Rust example, set the named-if rule's `level` in `.worker/lint.toml` to `"error"`. Leave Python at `"warning"`. The same binary enforces both policies.

## Exercise either installation

```sh
.worker/bin/discipline-worker --version
.worker/bin/discipline-worker doctor
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
