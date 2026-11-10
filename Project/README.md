# AgentRig worker

The Rust runtime owns agent and Git hooks, command execution and jobs, feature integration, memory validation and
structural lint. Just recipes and the Git, Codex and Claude Code registrations only forward calls to it. Python serves
behavioral tests and the latency benchmark. Project settings come from the `.agentrig/` modules: see the
[setup guide](project_setup/README.md) for their schema, [selective tests](change_delivery/gate/README.md) for the
affected-test map and the [architecture rule](structural_lint/architecture/README.md) for directory contracts and
analysis limits.

## Build

`tooling/worker/build` compiles a locked release binary with Rust 1.98.1 into `.cache/worker`, or into
`WORKER_TARGET_DIR`. It rebuilds when sources, assets, toolchain or dependency files change, serializes builds with
flock and writes build output to stderr so hook stdout stays JSON. `just candidate ...` runs it;
`tooling/worker/run` runs the pinned installation and never compiles. Rustfmt, Clippy, a native linker, Bash, Git and
flock must be available.

## Lint

Lint reads strict YAML version 1: the file `paths.lint` selects, `--config`, or `lint.yaml` when standalone. Unknown
fields, duplicate keys, anchors, aliases, tags and merge keys are rejected. A configuration error exits 2 and names the
repair skill; an error finding exits 1; warnings exit 0 and are printed only with `--warnings`.

- `just lint-rules` lists each rule kind with its target, languages, measurement and parameters;
  `just lint-rule KIND --example` prints a complete rule entry.
- `just lint-config-check` validates configuration, skills and selectors against the current inventory without
  reading sources.
- `just lint-explain PATH` shows why each rule selects or skips a path, with effective thresholds and overrides.
- `just lint --json` prints each finding's rule, path, line, symbol, actual, limit, level, message, skill and rerun
  command.

A rule has an id, a kind, a file or directory target, include globs, and optional exclude globs and extensions.
Numeric rules take a warning threshold, an error threshold or both, with warning below error; a larger value reports.
Policy rules take a level. Ordered overrides change thresholds for matching paths; later matches win. Global exclude
removes paths from the inventory; a rule's exclude only hides its findings. Globs are case-sensitive, `*` matches one
path component and `**` any depth; the root directory is `.`. The inventory is Git's tracked and non-ignored files.
`src/**` matches every path under `src` but not `src` itself, so a directory rule that also covers `src` lists both:
`include: [src, 'src/**']`.

Ratchet mode holds while `lint-baseline.yaml` lies beside `lint.yaml`, named after the configuration file. It maps
rule IDs to paths and their accepted error counts. Errors within a path's count become warnings; a rule's errors in
a path above it all stay errors, and counts below it are reported for lowering. `just lint-baseline` records the
current counts, later only lowers them and never adds new pairs; delete the file to accept new errors. A count lowered
in the working file can be restored up to the baseline committed at `HEAD`: a rerun returns it when the errors come
back. When Git detects a renamed file between `HEAD` and the working tree, which needs the new path in the index as
after `git mv`, recording moves its counts to the new path; a finding added by the move still exceeds them. `init`
offers it when existing files already have errors. The gate reads the baseline from the tree it checks, staged under
`--staged`.

Rust (`.rs`) and Python (`.py`, `.pyi`) files are parsed once per run with Tree-sitter; a file that does not parse
fails with its location. Rust uses the fork of tree-sitter-rust 0.24.2 in `shared/syntax/tree-sitter-rust`, which also
parses `safe` items of `unsafe extern` blocks, macros named after primitive types and lifetimes first after `dyn`;
other valid Rust the grammar does not know still fails. Syntax rules reject extensions without a handler. Analysis is
syntactic: no type checking, macro expansion, decorator alias resolution or generated declarations. Repository policy
and installed defaults block every language rule. Do not add exceptions to turn the gate green; change policy through a
recorded decision.

`structural_lint/rules` owns each kind's identity, target, metric, parameters, skill and defaults; the catalog,
validation and init consume it. `structural_lint/languages/registry.rs` binds grammars and inspection functions to
extensions and kinds. A new rule or language needs its measurement, registration, repair skill, and tests with accepted
and rejected examples.

The release build also produces `agentrig-lint`, which assesses another project without installing the worker:

```sh
agentrig-lint --root /projects/consumer --config /policies/lint.yaml --json
```

Selectors apply to the assessed root. An external policy can set `skill_root`, relative to its file, to use a separate
skill bundle. The assessment writes nothing, and its findings match the embedded linter.

## Jobs

Catalog commands run as jobs recorded under the runtime jobs directory. `just jobs`, `just job-status ID` and
`just job-logs ID` show the command, its state, the last 64 KiB of each stream and process observations. A process is
identified by PID, start time and boot identity, never by a bare PID. `just job-start COMMAND -- ARGS` runs a command
in the background as the leader of its own process group, and a delayed launch response never starts a duplicate.
`just job-stop ID` sends SIGTERM, then SIGKILL, to that group; descendants that start their own session or group
escape it. Owners come from WORKER_OWNER, else CODEX_THREAD_ID or CODEX_SESSION_ID. `just job-cleanup` and a
successful feature-merge stop the owner's task runs, skip the run they execute inside and keep `lifetime: shared`
services. Read-only commands run under bubblewrap on Linux and sandbox-exec on macOS, with a private writable temporary
directory and no network.

[measure.py](shared/testing/examples/measure.py) measures first and repeated standalone process latency of a
built binary.
