# Directory architecture lint

`directory-architecture` (opt-in in `agentrig lint` and `agentrig-lint`) checks contracts, feature types and cycles.

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
    feature_root: src
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
the actual crate roots (`[]` without Rust); `rust_crates` maps further local crate names to roots also listed there.
External names are literal module or package prefixes; Rust defaults to `std`, `core` and `alloc`, and none hides a
locally resolved path. `disallowed` entries (`path`, `reason`, `replacement`, `allow` globs, optional `methods`) flag,
outside `allow`, references to the path or a `::` subpath and bare calls of listed methods whose result goes straight
to `?` or `.map_err(Into::into)`, by name. `warning` level only reports; `error` exits 1, configuration errors 2.

## Directory contracts

```yaml
purpose: Orders application service
files:
  api.py: Public entry for the orders scenario
directories:
  tests: Behavior checks owned by orders
```

A contract has `purpose`, `files`, `directories` and, on a feature without nested features, `type`. `purpose` is one
nonempty line. `files` and `directories` register every immediate entry except the contract itself by literal name
with a nonempty responsibility; stale entries, wrong kinds and unknown fields are errors. Contracts stay in the project.

## Feature types

Subdirectories of the optional `feature_root` are features. A feature without `type` is composite: its subdirectories
except `tests` are features too. A feature with no other subdirectories must declare `type`; `type` anywhere else is
an error. A typed feature has exactly its type's parts plus an optional `tests`:

| Type | Parts, outer to inner |
| --- | --- |
| flat | none |
| layered | presentation, application, domain, infrastructure |
| hexagonal | adapters, ports, domain |
| onion | infrastructure, application, domain |
| clean | frameworks, adapters, use_cases, entities |
| custom | any subdirectories, unchecked |

A file in a part may use its own part and inner parts. Files in the feature root may use any part; parts never use
them, and nothing outside `tests` uses `tests`. Module declarations and dependencies on other features are not
direction findings, and flat and custom features add no direction rule.

Cycles use actual resolved edges and report source evidence across sibling subsystems. Rust module declarations
(`mod child;`) express composition and form no cycle, so a child may use a parent-owned type. A source file may use
source files in at most `fan_out` (default 10) other project directories; crate roots and `__main__.py` are exempt.

## Analysis limits

Tree-sitter extracts references without executing project code. Malformed syntax, unresolved references and
unsupported forms are incomplete-analysis findings at the configured level; this is not a compiler.

- Rust 2018+: modules, literal `#[path]`, imports, aliases, globs and `Self`; paths in macro arguments and
  `macro_rules!` bodies, literal `include!` and resource includes, standard derives, serde callbacks and external
  attribute macros. `OUT_DIR` includes and item-position macro calls generate their module's unresolved names.
  `#[cfg]` over test and platform predicates keeps every branch; duplicate bindings under exclusive predicates fail.
  Metavariable paths are invisible; unresolved macros, local attribute macros and computed includes are incomplete.
- Python: one import root, absolute and relative imports, package initializers, namespace submodules and literal
  runtime imports. Ambiguous members, wildcard exports, namespace-only imports and loader aliases fail or are
  incomplete. `.pyi` is parsed but is not a runtime target.
