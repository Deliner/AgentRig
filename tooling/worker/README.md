# Rust worker runtime and structural linter

The worker binary owns session/pre-edit hooks, command validation, complexity reminder state and transcript scanning, Git commit/reference guards, and structural lint. The existing Just command runner, feature integration orchestration, and Ledger checker remain Python; Python hook and size implementations are retained as historical parity references, not registered production handlers.

## Build and execution

Use just lint, just lint-rules, or tooling/worker/run through just write. The launcher builds a release binary with Cargo.lock and Rust 1.98.1, then executes it. Rustup, Cargo, rustfmt, Clippy, a native linker, Bash, Git, and flock must be available. First use needs toolchain/crate downloads; subsequent locked builds use the local cache. Install the pinned toolchain with rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy.

The default target is .cache/worker; WORKER_TARGET_DIR can place it outside a temporary staged tree. The launcher fingerprints source contents, compile-time reminder text, toolchain and dependency files, and serializes builds with flock. A source change, including a branch change or staged/working-tree difference, invalidates the cached binary. Build output goes to stderr so hook stdout remains JSON. No Python fallback executes when native compilation fails.

The hook registration invokes run hook with an explicit repository root. Ledger reminders remain context-only; existing command denials, complexity thresholds, retry allowance, session recovery guidance, SHA-256 state keys, file locks, and atomic state replacement are preserved. Full-refresh wording is a compile-time text asset. An existing session using an old registered command may require restart to use the native registration.

## Configuration

lint.toml uses TOML version 1 of the worker schema. Unknown fields, unsupported rule kinds/targets, invalid globs, duplicate IDs, missing skills, and invalid effective thresholds are errors. A configuration failure exits 2 and points at configure-linter; a structural error exits 1; warnings alone exit 0.

Each rules entry requires:

- id: unique diagnostic identity.
- kind: an implemented rule from just lint-rules.
- target: file or directory, compatible with that kind.
- include: nonempty repository-relative glob list.
- exclude: optional rule-local glob list.
- extensions: optional literal suffixes, such as .rs or .ts; only file rules accept them.
- warning and error: nonnegative thresholds, warning strictly below error. Equality with a threshold passes that threshold; a greater value triggers it.
- warning_skill and error_skill: existing repository SKILL.md paths with name/description frontmatter.
- overrides: optional ordered selector/threshold overrides.

Global exclude removes paths from the inventory before directory counts. Rule-local exclude only suppresses diagnostics on matching targets. Globs are case-sensitive, use / separators, and treat * as one path component and ** as recursive. Root-directory diagnostics use the path ".".

An override inherits the rule's target and repair skills. It supplies include, optional extensions, and at least one of warning/error. All matching overrides apply in declaration order; later supplied thresholds replace earlier values. Effective warning must remain below error. Different rules are independent and can both report on a file.

Example, placed inside the file-size rule before the next rules entry:

~~~toml
[[rules.overrides]]
include = ["Project/Runtime/**"]
extensions = [".rs", ".ts"]
warning = 220
error = 400

[[rules.overrides]]
include = ["Project/Runtime/parser.rs"]
warning = 280
error = 450
~~~

Do not add exceptions merely to turn the gate green. Preserve a current requirement or record a justified policy change.

## Rules and languages

nonblank-lines supports UTF-8 text files of any language. It counts nonempty lines, including comments, and skips non-UTF-8 files. Default suffixes include Python, shell, Rust, JavaScript/TypeScript, Go, C/C++, C#, Java, and common documentation/configuration formats. Selectors can narrow or extend these suffixes without Rust changes; an empty extension list selects all files supported by the rule.

directory-entries counts immediate child names, including child directories, from the selected file inventory. It is independent of language and rejects extension selectors. Git inventories include tracked and non-ignored untracked files; exported staged trees use their physical files. Deleted files and symlinks are excluded. Empty directories are not represented in Git and are not counted.

Current defaults retain warnings above 300 nonblank lines and errors above 500; directory warnings above 10 and errors above 15. Ledger/Decisions and Ledger/Invariants remain excluded from directory-size checks. All structural diagnostics include rule ID, path, measurement, limit, severity, and repair skill. Use just lint -- --json for structured output.

The registry in src/lint/rules.rs defines supported kinds and target/language capabilities. To add an actual rule, implement its measurement in the evaluator, update its registry metadata/validation and behavioral tests, and supply a repair skill. Syntax-aware language rules need their own parser and tests; extension selection alone does not claim AST analysis. The current extension point is compiled Rust code, not dynamically loaded plugins.

## Commit and merge gate

The pre-commit hook checks the actual exported Git index, including its Rust sources, lint config, and skill files. Structural lint runs before the other checks. The same gate runs on the integration candidate before merge and after a required rebase; errors stop the operation, warnings do not.

gate_skills maps each external stage (repo-policy, command-policy, Ruff, mypy, pytest, typos, Vulture, rustfmt, Clippy) to an existing repair skill. Native gate execution preserves the original tool output and appends the configured skill on failure. Structural findings have their own per-rule skills. No automatic fixer weakens policy or modifies files.

Native integration tests under tooling/tests/native execute the built binary. They compare hook responses and state with the Python reference and exercise configuration, selectors, thresholds, diagnostics and staged inventories. Existing Git branch/VAC tests use the native guards.

Configuration parsing uses the [TOML serde library](https://docs.rs/toml/latest/toml/); selectors follow [globset semantics](https://docs.rs/globset/latest/globset/). Builds use Cargo's [locked dependency mode](https://doc.rust-lang.org/cargo/commands/cargo-build.html).

## Observed hook latency

On this workspace, 50 interleaved warm PreToolUse invocations for Ledger/State.md measured median wall times of 28.019 ms for the Python reference, 8.071 ms for the registered Rust launcher including source validation, and 0.786 ms for the binary alone. The registered path was about 3.5 times faster for this event. This is a local measurement of a simple edit hook, not a claim about every event, first compilation, or the whole quality gate.
