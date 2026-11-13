# Repository instructions

## Quality standard

AgentRig sets the code standard (D151): code kept 20 years, AI-first and agent-native, clear at once, well structured,
its architecture minimizing corner cases, undefined states and error-prone places; lint rules derive from it. Code not
conforming to the rig is repaired. Keep every check that raises quality, even if the rig lacks the construct;
refactoring cost never rejects one, and decisions may be revised. The rig dogfoods its checks and fully configures any
project as a controlled, deterministic environment where an agent cannot write code that its built-in linter rejects.

## Resume and scope

Read Ledger/State.md, Ledger/Plan.md and any relevant feature detail, then inspect Git status and recent commits.
Reconcile recorded progress with live evidence; preserve unrelated work. Follow the current decision index and explicit
successors, not superseded details. Apply complexity-discipline for non-trivial implementation, type-driven-discipline
when a value carries an invariant, a mode or a lifecycle, and norm-or-choice for durable policy changes.

Use the execute-plan-feature skill for VAC delivery, plan evolution, blockers and branch integration. Explicit
maintenance instructions independently authorize tooling work; they do not authorize starting unrelated product
features. This repository develops the portable worker itself. Worker capabilities, including lint, are first-level
features placed directly in Project under two-word names, with leaf support in Project/shared; tooling holds the
development launchers, distribution and Python verification configuration, and repository memory is Ledger (D042).
Runtime paths, Git base/prefix, commands, checks, tools, hook routes, oracles and Rust flags live in .agentrig/ modules.

## Commands and verification

### IMPORTANT: ORDINARY FILE EDITING DOES NOT REQUIRE `just write`

**Per the user's explicit instruction, agents may edit files directly with editing tools such as `apply_patch`, `Edit`
and `Write`, without a `just write` operation. Do not require a Just wrapper for ordinary file editing.**

Invoke shell operations through the Just recipes grouped under just/ (`just --list`; `just list` shows the catalog).
Use `just read -- PROGRAM ...` for sandboxed inspection and `just write -- PROGRAM ...` for shell mutations with the
programs configured in .agentrig/commands.yaml, `just run NAME -- ...` for catalog commands and `just scratch -- TEXT`
for temporary code in the branch task area. Each recipe is a thin adapter directly preceded by a What/Why comment.

Choose a cohesive VAC and a falsifiable focused check, edit and correct failures, then inspect and stage only that
change. When a VAC adds or changes tests, apply testing-discipline: place each test by the behavior it exercises and
verify through the public surface, not internal state. Commit through .agentrig/hooks/pre-commit, which runs the
configured exported-index gate with affected test groups plus smoke tests. Do not substitute the full test suite for
unmapped commit paths; maintain the impact map instead. Never bypass the hook; avoid rerunning the gate just before
the same commit except for diagnosis. Finish on a clean feature branch with `just feature-merge`, which runs the tests
affected by the whole branch; retain the branch. Never commit directly to the base configured in .agentrig/git.yaml.

## Memory and skills

Before editing Plan, Archive/Plan, Backlog, Decisions, Invariants or State (including detail files), read the matching
edit-* skill. Their canonical bodies live in Project/shared/skills and are exposed through .agents/skills. Do not
create another instruction copy. Hook routes give memory-skill guidance, stop the first code edit in each directory
until refactor-large-directory and its contracts are applied, and stop the first VAC commit in a context for edit-state
(D047). Opaque shell writes require applying the relevant skill yourself.

Use edit-backlog to draft new ideas and requirement changes in chat; save the shown version only after explicit user
agreement. All new product outcomes pass through Backlog before Plan. Plan contains only agreed work selected for
current or resumed execution, referencing the same acceptance contract. A selected set runs autonomously without
repeated questions about implementation or permission between cards. Backlog has no execution priority. Follow the
canonical skills for unresolved behavior, criterion-based acceptance and automatic rotation of completed Plan rows/cards
and their Backlog cards into Archive/. Required work remains in the current task.

State is a factual recovery snapshot. Refresh it at meaningful task/VAC boundaries, blockers and handoffs, with actual
verification and a concrete next action; include it in the relevant VAC. Plan owns current selection and delivery,
referencing the agreed Backlog outcome and acceptance. Archive retains completed outcome cards; Git owns VAC history.

D020 governs consolidation: committed decision IDs, statements and detail contents are immutable; application links
identify current owners and may migrate with refactoring. Each linked Rust/Python/shell owner needs a real DECISION:
DNNN comment. Invariants bind a marked Rust/Python test to its exact configured executable oracle.

## Runtime changes

Run `just bootstrap` once after cloning to install the verified revision in tooling/distribution/stable.txt. Use
tooling/worker/run for the installed development runtime; it never builds candidate sources. All command, hook, memory,
gate and Git logic belongs to the Rust runtime; adapters only forward calls.

Use `just candidate ...` to build and invoke the product under development. The build check and native test fixture
select this candidate, while Git/agent hooks and gate orchestration use the installed revision. Without asking, finish
every branch that changes Project by writing its gate-accepted head to stable.txt and bootstrapping it in one command,
then commit the pin before `just feature-merge` (D051). See tooling/distribution/README.md for clean-clone setup.

Use `just config-check`, `just lint-config-check` and `just lint-rules` for configuration, lint settings and actual rule
capabilities. Apply the reported repair skill; preserve policy intent, never weakening thresholds or selectors to pass.
New rules and language handlers need implemented measurements, declared capabilities, tests and repair guidance.

## Rust 1.100

Write current Rust: use these stable capabilities where they fit, even when your training predates them.

- `!` is a stable type and `Infallible` its alias: write `Result<T, !>` and `let Ok(value) = result;`.
- `std::mem::DropGuard::new(value, f)` calls `f(value)` on drop; `DropGuard::dismiss` cancels it, returning the value.
- `trim_prefix` and `trim_suffix` on `str`, slices and `Path` replace `strip_prefix(p).unwrap_or(x)`.
- `bool::toggle`, `Vec::from_fn(len, f)`, `Box::map`, `Rc::map`, `Arc::map` and `Box::take`.
- The allocator API: `Allocator`, `Global`, `Box::new_in` and `Vec::new_in`.
- `[lints.cargo]` in `Cargo.toml` and `global-min-publish-age` in `.cargo/config.toml`.
