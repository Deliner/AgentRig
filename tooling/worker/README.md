# Rust worker runtime and structural linter

The worker binary owns session/pre-edit hooks, command validation, complexity reminder state and transcript scanning, Git commit/reference guards, and structural lint. The existing Just command runner, feature integration orchestration, and Ledger checker remain Python; Python hook and size implementations are retained as historical parity references, not registered production handlers.

## Build and execution

Use just lint, just lint-rules, or tooling/worker/run through just write. The launcher builds a release binary with Cargo.lock and Rust 1.98.1, then executes it. Rustup, Cargo, rustfmt, Clippy, a native linker, Bash, Git, and flock must be available. First use needs toolchain/crate downloads; subsequent locked builds use the local cache. Install the pinned toolchain with rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy.

The default target is .cache/worker; WORKER_TARGET_DIR can place it outside a temporary staged tree. The launcher fingerprints source contents, compile-time reminder text, toolchain and dependency files, and serializes builds with flock. A source change, including a branch change or staged/working-tree difference, invalidates the cached binary. Build output goes to stderr so hook stdout remains JSON. No Python fallback executes when native compilation fails.

The hook registration invokes run hook with an explicit repository root. Ledger reminders remain context-only; existing command denials, complexity thresholds, retry allowance, session recovery guidance, SHA-256 state keys, file locks, and atomic state replacement are preserved. Full-refresh wording is a compile-time text asset. An existing session using an old registered command may require restart to use the native registration.

## Configuration

lint.toml uses TOML version 1 of the worker schema. Unknown fields, unsupported rule kinds/targets, invalid globs, duplicate IDs, missing skills, and invalid effective thresholds are errors. A configuration failure exits 2 and points at configure-linter; a structural error exits 1; warnings alone exit 0.

Validate independently with just lint-config-check. Use just lint-config-check -- --config path/to/lint.toml --json for another config and machine-readable diagnostics (an empty array means valid). Exit 0 means the configuration and current target selection are valid; exit 2 reports a configuration error and repair skill. This command checks TOML/schema, skills, selectors, supported targets/extensions and effective overrides against the current inventory, without reading or parsing source contents. It does not claim the source passes lint. Normal lint uses the same validation automatically.

Use just lint-rules to inspect each rule's target, languages, supported handler extensions and measurement. These are implementation capabilities, not user-editable claims. Language selection uses extensions; setting a suffix cannot create a handler.

Each rules entry requires:

- id: unique diagnostic identity.
- enabled: optional boolean, true by default. false skips target selection and execution; the entry still needs valid schema, capabilities and repair skills.
- kind: an implemented rule from just lint-rules.
- target: file or directory, compatible with that kind.
- include: nonempty repository-relative glob list.
- exclude: optional rule-local glob list.
- extensions: optional literal suffixes, such as .rs or .ts; only file rules accept them.
- Numeric rules require warning, error, or both: nonnegative thresholds; warning must be strictly below error when both exist. Equality passes; a greater value triggers that level. Omit error for warnings only, or warning for blocking only.
- named-if-condition instead requires level = "warning" or "error", with no numeric thresholds or threshold overrides.
- warning_skill and error_skill: existing repository SKILL.md paths with name/description frontmatter.
- overrides: optional ordered selector/threshold overrides.

Global exclude removes paths from the inventory before directory counts. Rule-local exclude only suppresses diagnostics on matching targets. Globs are case-sensitive, use / separators, and treat * as one path component and ** as recursive. Root-directory diagnostics use the path ".".

An override inherits the rule's target and repair skills. It supplies include, optional extensions, and at least one of warning/error. All matching overrides apply in declaration order; later supplied thresholds replace earlier values. Effective warning must remain below error when both exist. An omitted override threshold inherits its prior value; overrides cannot remove thresholds. Different rules are independent and can both report on a file.

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

Example syntax rule selection (inside its rules entry):

~~~toml
enabled = true
include = ["Project/**"]
exclude = ["Project/generated/**"]
extensions = [".rs", ".py"]
~~~

Setting extensions = [".sh"] on function-lines fails with an error naming supported Rust/Python handlers. The same applies to numeric override selectors. File-size remains language independent, so it can select shell scripts.

### Syntax-aware rules

The Rust engine parses each selected source once per run using Tree-sitter. Compiled handlers in src/lint/languages interpret Rust (.rs) and Python (.py, .pyi) syntax and produce measurements for the same rules. Empty extensions applies the include/exclude selection without an implicit language filter: any selected file lacking a handler is a configuration error. In mixed directories, explicitly set extensions or exclude unsupported files. An unsupported literal suffix in include (such as scripts/*.sh) or extensions is rejected even if no files exist yet. Other glob forms are checked against the current inventory; validation does not predict future files. Other file types remain available to language-independent rules. Files with parse errors block with a source location and repair skill, even when rule findings are warnings. This is syntax analysis, not type checking or name resolution.

- named-if-condition accepts one identifier or named field path, optionally parenthesized. Calls, comparisons, negations, boolean operators, indexing and literals must be assigned a meaningful name before branching. Rust else-if and Python elif, conditional expressions and comprehension filters are included. A simple Rust if let is a binding pattern and remains allowed; let chains are reported. The linter cannot prove that a Python name contains bool or that its name explains the branch.
- function-lines counts nonblank lines from the function signature through the end of its body, including comments, docstrings and nested definitions. Decorators and preceding attributes are excluded. Methods, async functions, constructors, nested functions and anonymous closures/lambdas are included; declarations without bodies have no size finding. Nested functions are also measured independently.
- parameter-count counts declared inputs, including optional and variadic parameters as one each. Generic type parameters, commas inside types/defaults, and separators do not count. Rust self receivers (including typed self) and the first bound Python method receiver are excluded. For literal @staticmethod decorators, inputs all count; bound method/classmethod receivers do not. Decorator aliases are not resolved. Python constructors are checked through explicit __init__/__new__ declarations; Rust associated constructor functions such as new are ordinary functions. Calls, class inheritance arguments, generated constructors (such as dataclass) and macro-expanded code are not inferred.

Defaults enable all three new rules across the existing Rust/Python harness as warnings: named conditions at every finding, functions above 40 nonblank lines, and signatures above 4 inputs. This initial rollout exposes existing violations without an unrelated repository-wide rewrite. To block named conditions, set level = "error". For numeric rules, add a justified hard threshold (for example, error = 60 for functions or error = 6 for parameters). Existing file/directory blocks remain active. Selectors and numeric overrides work identically for structural and syntax rules.

Syntax diagnostics add a 1-based line and symbol to the existing JSON fields; text output renders file:line (symbol). Each finding points to name-if-condition, refactor-long-function or reduce-parameters. No automatic code transformation is performed.

The registry in src/lint/rules.rs declares supported kinds, targets and actual handlers. To add a rule, implement its measurements, validation, behavioral tests and repair skill. To support another language, add its grammar and handler, update supported extensions and registry metadata together, and test its syntax against the existing rule semantics. The extension point is compiled Rust code; there is no dynamic plugin lifecycle. Rust macros are opaque token trees, and Python decorator aliases or generated declarations require semantic tooling beyond these handlers.

Parser APIs and grammars: [Tree-sitter](https://docs.rs/tree-sitter/0.26.13/tree_sitter/), [Rust grammar](https://docs.rs/tree-sitter-rust/0.24.2/tree_sitter_rust/), [Python grammar](https://docs.rs/tree-sitter-python/0.25.0/tree_sitter_python/).

## Commit and merge gate

The pre-commit hook checks the actual exported Git index, including its Rust sources, lint config, and skill files. Structural lint runs before the other checks. The same gate runs on the integration candidate before merge and after a required rebase; errors stop the operation, warnings do not.

gate_skills maps each external stage (repo-policy, command-policy, Ruff, mypy, pytest, typos, Vulture, rustfmt, Clippy) to an existing repair skill. Native gate execution preserves the original tool output and appends the configured skill on failure. Structural findings have their own per-rule skills. No automatic fixer weakens policy or modifies files.

Native integration tests under tooling/tests/native execute the built binary. They compare hook responses and state with the Python reference and exercise configuration, selectors, thresholds, diagnostics and staged inventories. Existing Git branch/VAC tests use the native guards.

Configuration parsing uses the [TOML serde library](https://docs.rs/toml/latest/toml/); selectors follow [globset semantics](https://docs.rs/globset/latest/globset/). Builds use Cargo's [locked dependency mode](https://doc.rust-lang.org/cargo/commands/cargo-build.html).

## Observed hook latency

On this workspace, 50 interleaved warm PreToolUse invocations for Ledger/State.md measured median wall times of 28.019 ms for the Python reference, 8.071 ms for the registered Rust launcher including source validation, and 0.786 ms for the binary alone. The registered path was about 3.5 times faster for this event. This is a local measurement of a simple edit hook, not a claim about every event, first compilation, or the whole quality gate.
