# Directory architecture lint

`directory-architecture` is an opt-in rule in both `agentrig lint` and
`agentrig-lint`. It checks measured source dependencies, not dependencies inferred
from permission declarations. Existing installations do not enable it automatically.

## Configuration

Add a rule to the existing lint YAML; skill paths use the normal `skill_root`:

```yaml
- id: architecture
  kind: directory-architecture
  target: directory
  include: [src, 'src/**']
  exclude: ['src/generated', 'src/generated/**']
  extensions: [.rs, .py, .js, .ts, .tsx]
  level: error
  architecture:
    python_root: src
    rust_roots: [src/lib.rs]
    external:
      rust: [std, core, alloc]
      python: [json]
      javascript: ['node:fs', react]
  warning_skill: .agents/skills/refactor-large-directory/SKILL.md
  error_skill: .agents/skills/refactor-large-directory/SKILL.md
```

The target is a directory; `extensions` selects its immediate source files.
Include both a parent and its descendants when both are in scope. Every selected
directory requires a contract, including documentation and resource directories.
Inventory coverage is independent of source extensions. Use `.` to select the project
root. Exclusions use the existing glob semantics; exclude a subtree with both its
directory and descendant pattern. Selected-source exclusions do not remove files
from dependency resolution. Resolution uses Git tracked/unignored files, or regular
non-symlink files in a non-Git project. Git-ignored targets cannot be resolved.

Without a VCS, filesystem discovery also retains empty directories, so an empty
selected directory still needs its contract. With a VCS, the selected backend's
tracked/unignored file inventory defines the directory tree; Git and Mercurial
do not version empty directories. Adding architecture.yaml makes such a directory
part of that inventory. Symlink entries are not followed by filesystem discovery.

`architecture` and explicit source extensions are required. `python_root` defaults
to the project root. `rust_roots` lists the actual crate roots and is required when
Rust sources are selected; use `[]` for other languages. No Cargo or tsconfig
discovery is implied. External names are explicit module/package prefixes, not
permission globs. Rust defaults to `std`, `core`, `alloc`; the other lists default
to empty. Locally resolved Python and anchored Rust paths cannot be hidden by an
external declaration. JavaScript bare package names require an external declaration;
local package aliases are not yet resolved.

`level: warning` reports findings without a failing exit status; `error` exits 1.
Thresholds and overrides are not applicable. Configuration errors exit 2.
`lint-rules`, `lint-config-check` and `lint-explain DIRECTORY` use the shared rule
registry and selection. Diagnostics retain the originating path/line, repair skill
and rerun command. Contract failures point to the contract path.

## Directory contracts

Each required `architecture.yaml` is strict YAML:

```yaml
purpose: Orders application service
files:
  architecture.yaml: Directory responsibilities and boundaries
  api.py: Public entry for the orders scenario
directories:
  tests: Behavior checks owned by orders
allow: ['src/catalog/api.py', 'src/storage/**']
deny: ['src/storage/private/**']
public: [api.py]
```

`purpose` must be nonempty and occupy one logical line. `files` registers immediate
filenames with nonempty responsibility descriptions, including architecture.yaml.
`directories` separately describes immediate child directories, each with its own
contract. Every in-scope entry must be registered. Names are literal, not globs;
stale entries and wrong file/directory kinds are errors. Existing P006 contracts
must acquire these maps; absent maps are empty and missing entries are reported.
Descriptions guide placement; lint cannot prove their semantic agreement with code.
Unknown fields are errors. `allow` and `deny` match project-relative target file paths for outbound
dependencies. Deny takes precedence. `public` matches paths relative to this
directory for inbound dependencies. Dependencies inside the same boundary need no
permission. Every crossed enclosing contract applies; a child cannot open its
parent's private boundary. Patterns cannot escape the project. Contracts must be
regular files inside the project.

Cycles use actual resolved edges and report source evidence. Checks include sibling
subsystem boundaries even when opposite edges connect different nested directories.
An allowed edge still participates in cycle detection.

## Current source analysis

Tree-sitter extracts references without executing project code. Malformed syntax,
unresolved references and recognized unsupported forms produce incomplete-analysis
findings at the configured severity. This is not a compiler or a full static-analysis
proof; remaining coverage is part of P006 delivery.

- **Rust 2018+**: explicit crate roots, declared `name.rs`/`name/mod.rs` and inline
  modules, `crate`/`self`/`super`, module-level imports and aliases, qualified item
  paths and public item facades. Missing/ambiguous module files and alias cycles
  fail. Known standard expression macros and imported `anyhow::{anyhow,bail,ensure}`
  and `serde_json::json` retain explicit paths inside their token arguments,
  including nested calls; strings and comments remain opaque. Macro imports are
  resolved before accepting the namespace. Unknown macros, definitions, source
  `include!`, ambiguous wildcard origins, block-local macro imports and declarations
  inside macro arguments report incomplete analysis. General expansion is not
  implemented. Static unescaped `include_str!` and `include_bytes!` literals
  produce resource-file edges, including imported aliases; missing, ignored,
  escaping or dynamically constructed targets fail analysis.
  Standard derives and `serde::{Serialize,Deserialize}` are supported. Supported
  serde callback attributes retain their function-path dependencies; naming
  metadata remains opaque. Unknown serde forms fail explicitly. `#[cfg(test)]`
  includes test code in the measured graph alongside ordinary code.
  `#[path]`, block-local modules and unresolved lexical/wildcard bindings require
  further analysis. Other conditional and unknown attributes report incomplete
  expansion, including inner attributes.
  Known nonexpanding metadata (`allow`, `warn`, `deny`, `forbid`, `doc`, `inline`,
  `cold`, `must_use`, `deprecated`, `repr`, `non_exhaustive`, `test`, `ignore`,
  `should_panic`, `track_caller`) remains supported. Compiler expansion and build
  condition evaluation are not implemented.
- **Python**: one import root, absolute/relative imports, regular package initializer
  chains and concrete namespace submodules, literal runtime imports. Package member
  ambiguity, package wildcard exports and namespace-only imports fail explicitly.
  Recognized loader import aliases and loader functions used as values report
  incomplete binding/data-flow analysis. Initializer export analysis and runtime
  search-path modification are not implemented. `.pyi` can be parsed but is not a
  runtime target fallback.
- **JavaScript** (`.js`, `.jsx`, `.mjs`, `.cjs`): static import/export and literal
  `import()`, `require()` and `require.resolve()`. Node imports require exact files;
  require searches Node extensions and relative package main/index. Dynamic strings,
  encoded paths and unsupported package resolution fail. Loader values and the
  `module`/`node:module` factory API report incomplete analysis; their alias and
  factory bindings are not inferred.
- **TypeScript** (`.ts`, `.tsx`, `.mts`, `.cts`): the same extraction plus type imports,
  exports and import-equals. Resolution uses the implemented bundler substitutions
  and relative package types/typings/main. NodeNext, tsconfig path aliases, module
  suffixes and package typesVersions are not implemented; typesVersions and ambiguous
  package fallback report errors. Do not treat this mode as validation of a different
  TypeScript resolution configuration.

Use the linked `refactor-large-directory` skill to repair responsibility, dependency
direction or public access. Moving files into arbitrary buckets or widening
permissions to silence findings does not establish the intended architecture.

## Executable consumer verification

`tooling/tests/native/lint/test_architecture_behavior.py` compiles or executes
independent Rust, Python, JavaScript and TypeScript consumers before and after
replacing private access with an existing public API. The result stays `7`, the
directory contracts remain byte-for-byte unchanged, and both lint binaries agree
on the violation and repair without changing consumer files. These tests use
Rust 1.98.1, Python 3.12.3 and Node 22.22.3. TypeScript execution uses Node's
`--experimental-strip-types` with explicit `.ts` imports and type annotations;
it verifies runtime behavior, not TypeScript compiler type checking.
