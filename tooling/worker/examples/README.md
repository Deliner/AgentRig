# Independent scaffold examples

These are consumer projects. Copy either `python/` or `rust/` outside the worker repository and set `WORKER_BINARY` to
the absolute path of a built `agentrig` binary; initialization copies that binary and embeds its configuration, memory
and skills into the consumer.

For Python, install Python 3 and pytest. For Rust, install Cargo and a Rust compiler supporting edition 2024. Both
examples use Linux or macOS, Git and Just; on Linux they also need bubblewrap. `doctor` diagnoses executables, sandbox
availability and hook registration; it does not install tools.

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
"$WORKER_BINARY" init --language rust --source crates/engine --memory knowledge --skills policies \
  --base release --prefix change/
```

Generated defaults enforce named conditions, function length, parameter limits and decision/invariant-only comments as
errors for both Rust and Python. Project-specific selectors and thresholds remain configurable in `.agentrig/lint.yaml`.

## Exercise either installation

```sh
.agentrig/bin/agentrig --version
just config-check
just resume
just run test
just check
git add .
just check --staged
printf '%s' '{"hook_event_name":"SessionStart","session_id":"example"}' | .agentrig/bin/agentrig hook
git commit -m "Attach portable scaffold"
just feature-merge
just feature-start next
```

`git commit` runs the staged gate; `feature-merge` checks the integration candidate and retains the bootstrap branch.
The initial commit precedes hook installation, so bootstrap never bypasses an installed guard.

The source marker can be linked from the generated invariant index. For Python, add `[I001](Invariants/001.md)`, a
predicate, and `[test_doubles](../application/test_sample.py)` as one index row; create `notes/Invariants/001.md` with
nonempty `Predicate` and `Oracle` sections. Add to `agentrig.yaml`:

```yaml
oracles:
  I001:
    check: tests
    runner: pytest
    target: application/test_sample.py::test_doubles
```

For Rust use `knowledge/Invariants/001.md`, `[doubles](../crates/engine/src/lib.rs)` and:

```yaml
oracles:
  I001:
    check: tests
    runner: cargo
    target: tests::doubles
```

Run `memory-check` to validate the link and discover the exact test target. Run `check` to execute the test. Discovery
alone does not verify the predicate.

[Observed latency](LATENCY.md) lists first-process and repeated-process measurements and their reproducible command.
