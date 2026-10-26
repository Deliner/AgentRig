# Directory architecture lint

`directory-architecture` is an opt-in rule of `agentrig lint` and `agentrig-lint`. It checks measured source
dependencies against per-directory contracts, not dependencies inferred from the contracts.

## Configuration

```yaml
- id: architecture
  kind: directory-architecture
  target: directory
  include: [src, 'src/**']
  exclude: ['src/generated', 'src/generated/**']
  extensions: [.rs, .py]
  level: error
  architecture:
    python_root: src
    rust_roots: [src/lib.rs]
    external:
      rust: [std, core, alloc]
      python: [json]
  error_skill: .agents/skills/refactor-large-directory/SKILL.md
```

Every selected directory needs a contract, including documentation, resource and empty directories. Select a parent
and its descendants (`.` is the root); exclude a subtree with both patterns. `extensions` selects the immediate files
parsed for dependencies; inventory coverage does not depend on it. Resolution uses Git's tracked and unignored files,
or regular non-symlink files outside Git; Git metadata and symlinked directories are skipped.

`architecture` and explicit extensions are required. `python_root` defaults to the project root. `rust_roots` lists
the actual crate roots (`[]` without Rust); `rust_crates` maps further local crate names, such as
`{catalog: crates/catalog/src/lib.rs}`, to roots also listed there. External names are module or package prefixes, not
globs; Rust defaults to `std`, `core` and `alloc`. An external name cannot hide a locally resolved path.

`level: warning` reports without failing; `error` exits 1; configuration errors exit 2. `lint-rules`,
`lint-config-check` and `lint-explain DIRECTORY` share the registry and selection. Findings carry the source path and
line, or the contract path, with the repair skill and rerun command.

## Directory contracts

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

`purpose` is one nonempty line. `files` and `directories` register every immediate entry by literal name with a
nonempty responsibility; stale entries, wrong kinds and unknown fields are errors. Lint cannot prove that descriptions
agree with the code. `allow` and `deny` match project-relative targets of outbound dependencies, and deny wins.
`public` matches paths relative to the directory for inbound dependencies. Dependencies within one boundary need no
permission; every crossed enclosing contract applies, so a child cannot open its parent's private boundary. Patterns
and contracts cannot leave the project.

Cycles use actual resolved edges, allowed ones included, and report source evidence across sibling subsystems. Rust
module declarations (`mod child;`) express composition: their targets pass allow, deny and public checks but form no
cycle, so a child may use a parent-owned type.

## Analysis limits

Tree-sitter extracts references without executing project code. Malformed syntax, unresolved references and
unsupported forms are incomplete-analysis findings at the configured level; this is not a compiler.

- Rust 2018+: crate roots, file and inline modules, literal `#[path]`, `crate`/`self`/`super`, imports, aliases,
  qualified paths, public facades and `Self`. Explicit paths inside known standard macros, `anyhow` and
  `serde_json::json`, static `include_str!`/`include_bytes!` targets, standard derives and serde callbacks are measured.
  `#[cfg]` over test and platform predicates keeps every branch; duplicate bindings under exclusive predicates fail.
  Unknown macros, attributes and expansion are incomplete.
- Python: one import root, absolute and relative imports, package initializers, namespace submodules and literal
  runtime imports. Ambiguous members, wildcard exports, namespace-only imports and loader aliases fail or are
  incomplete. `.pyi` is parsed but is not a runtime target.

Repair findings with the `refactor-large-directory` skill. Moving files into arbitrary buckets or widening permissions
to silence findings does not establish the intended architecture.
