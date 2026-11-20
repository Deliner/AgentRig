# Independent scaffold examples

These are consumer projects. Copy either `python/` or `rust/` outside the worker repository and set `WORKER_BINARY` to
the absolute path of a built `agentrig` binary; initialization copies that binary and embeds its configuration, memory
and skills into the consumer.

For Python, install Python 3 and pytest. For Rust, install Cargo and a Rust compiler supporting edition 2024. Both
examples use Linux or macOS, Git and Just; on Linux they also need bubblewrap. After initialization,
`.agentrig/bin/agentrig doctor` in a terminal diagnoses executables, sandbox availability and hook registration; it does
not install tools.

## Python

From the copied Python directory:

```sh
git init -b trunk
git config user.name "Example Developer"
git config user.email "example@example.invalid"
git add .
git commit -m "Initial example"
"$WORKER_BINARY" init --language python --skills guides --base trunk --prefix topic/
just feature-start bootstrap
```

## Rust

From the copied Rust directory:

```sh
git init -b release
git config user.name "Example Developer"
git config user.email "example@example.invalid"
git add .
git commit -m "Initial example"
"$WORKER_BINARY" init --language rust --skills policies --base release --prefix change/
just feature-start bootstrap
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
printf '%s' '{"hook_event_name":"SessionStart","session_id":"example"}' | .agentrig/bin/agentrig hook --agent codex
```

Describe the change in `Ledger/State.md` with the edit-state skill; the commit refuses the empty template:

```sh
just write -- git add .
just write -- git commit -m "Attach portable scaffold"
just feature-merge
just feature-start next
```

The commit runs the staged gate; `feature-merge` checks the integration candidate and retains the bootstrap branch.
The initial commit precedes hook installation, so bootstrap never bypasses an installed guard.

The source marker can be linked from the generated invariant index. For Python, add `[I001](Invariants/001.md)`, a
predicate, and `[test_doubles](../Project/tests/test_sample.py)` as one index row; create `Ledger/Invariants/001.md`
with nonempty `Predicate` and `Oracle` sections. Write `.agentrig/oracles.yaml`:

```yaml
I001:
  check: tests
  runner: pytest
  target: Project/tests/test_sample.py::test_doubles
```

For Rust use `Ledger/Invariants/001.md`, `[doubles](../Project/tests/mod.rs)` and:

```yaml
I001:
  check: rust-test
  runner: cargo
  target: tests::doubles
```

Run `just check --only memory` to validate the link and discover the exact test target. Run `just check` to execute
the test. Discovery alone does not verify the predicate.

`cargo run --example measure -- --worker ABSOLUTE_AGENTRIG > REPORT.md` measures first and repeated process latency
of hooks and lint on fresh installations of both examples.
