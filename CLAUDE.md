# Repository instructions

## Quality standard

AgentRig sets the code standard (D151): code kept 20 years, AI-first and agent-native, clear at once, well structured,
its architecture minimizing corner cases, undefined states and error-prone places; lint rules derive from it. Code not
conforming to the rig is repaired. Keep every check that raises quality, even if the rig lacks the construct;
refactoring cost never rejects one, and decisions may be revised. The rig dogfoods its checks. No backward
compatibility (D185): a change may break projects, configurations and runtimes from an older rig, which then update by
hand. No migration, upgrade path or compatibility shim is required until the user explicitly asks.

## Resume and scope

Read Ledger/State.md, Ledger/Plan.md and any relevant feature detail, then inspect Git status and recent commits.
Reconcile recorded progress with live evidence; preserve unrelated work. Follow the current decision index and explicit
successors, not superseded details. Apply complexity-discipline for non-trivial implementation, type-driven-discipline
when a value carries an invariant, a mode or a lifecycle, and norm-or-choice for durable policy changes.

Use the execute-plan-feature skill for VAC delivery, plan evolution, blockers and branch integration. Explicit
maintenance instructions independently authorize tooling work; they do not authorize starting unrelated product
features. Worker capabilities, including lint, are first-level features placed directly in Project under two-word
names, with leaf support in Project/shared; tooling holds the development launchers, distribution and Python
verification configuration, and repository memory is Ledger (D042); project settings live in .agentrig/ modules.

## Commands and verification

### IMPORTANT: ORDINARY FILE EDITING DOES NOT REQUIRE `just write`

**Per the user's explicit instruction, agents may edit files directly with editing tools such as `apply_patch`, `Edit`
and `Write`, without a `just write` operation. Do not require a Just wrapper for ordinary file editing.**

Invoke shell operations through the Just recipes grouped under .agentrig/just/. Use `just read -- PROGRAM ...` for
sandboxed inspection, `just write -- PROGRAM ...` for shell mutations with configured programs and
`just run NAME -- ...` for catalog commands. Each recipe is a thin adapter directly preceded by a What/Why comment.

Choose a cohesive VAC and a falsifiable focused check, edit and correct failures, then inspect and stage only that
change. When a VAC adds or changes tests, apply testing-discipline. Commit through .agentrig/hooks/pre-commit, which
runs the configured exported-index gate with affected test groups plus smoke tests. Do not substitute the full test
suite for unmapped commit paths; maintain the impact map instead. Never bypass the hook; avoid rerunning the gate just
before the same commit except for diagnosis. Finish on a clean feature branch with `just feature-merge`, which runs the
tests affected by the whole branch; retain the branch. Never commit directly to the base set in .agentrig/git.yaml.

## Memory and skills

Before editing Plan, Archive/Plan, Backlog, Decisions, Invariants or State (including detail files), read the matching
edit-* skill. Skills keep one copy in Project/shared/skills, which Codex reads via .agents/skills and Claude Code via
.claude/skills. Hook routes in .agentrig/hooks.yaml give skill guidance, and a blocking route stops the first
matching edit, check or commit until its skill is read (D186, D047); for opaque shell writes, apply the skill yourself.

Use edit-backlog to draft new ideas and requirement changes in chat; save the shown version only after explicit user
agreement. New product outcomes pass through Backlog, which has no priority, to Plan, which holds only selected work.
A selected set runs autonomously without repeated questions between cards; completed work rotates into Archive/.

State describes only the current VAC, with its verification and next action, and each feature commit carries it; the
commit guard refuses the empty template that post-commit restores, and feature-merge commits it. Git owns history.

D020 governs consolidation: committed decision IDs, statements and detail contents are immutable; application links
identify current owners and may migrate with refactoring. Each linked Rust/Python/shell owner needs a real DECISION:
DNNN comment. Invariants bind a marked Rust/Python test to its exact configured executable oracle.

## Runtime changes

Run `just bootstrap` once after cloning to install the verified revision in tooling/distribution/stable.txt. All
command, hook, memory, gate and Git logic belongs to the Rust runtime; adapters only forward calls.

Use `just candidate ...` to build and invoke the product under development. The build check and native test fixture
select this candidate, while Git/agent hooks and gate orchestration use the installed revision. Finish every branch
that changes Project with `just propose-pin HEAD` at its gate-accepted head. It pins that revision itself while the
user's `! just consent` holds, else ask the user to type `! just approve`, which installs it and then pins it; commit
the pin before `just feature-merge` (D051, D218, D226). See tooling/distribution/README.md for clean-clone setup.

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
