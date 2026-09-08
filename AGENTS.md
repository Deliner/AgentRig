# Repository instructions

## Resume and scope

Read Ledger/State.md, Ledger/Plan.md and any relevant feature detail, then inspect Git status and recent commits. Reconcile recorded progress with live evidence; preserve unrelated work. Follow the current decision index and explicit successors rather than superseded historical details. Apply complexity-discipline for non-trivial implementation and norm-or-choice for durable policy changes.

Use the execute-plan-feature skill for VAC delivery, plan evolution, blockers and branch integration. Explicit maintenance instructions independently authorize tooling work; they do not authorize starting unrelated product features. This repository develops the portable worker itself. Worker capabilities, including lint and review, belong under tooling/worker; Project is reserved for consumer examples, and repository memory is Ledger. D025 supersedes D008 for ownership in this repository. Runtime paths, Git base/prefix, commands, checks, hook routes and oracle bindings are owned by agentrig.yaml.

## Commands and verification

### IMPORTANT: ORDINARY FILE EDITING DOES NOT REQUIRE `just write`

**Per the user's explicit instruction, agents may edit files directly with editing tools such as `apply_patch`, `Edit` and `Write`, without a `just write` operation. Do not require a Just wrapper for ordinary file editing.**

Invoke shell operations through the justfile recipes (`just --list`). `just list` lists the configured command catalog. Use `just read -- ...` for sandboxed shell inspection, `just write -- ...` for authorized shell mutations, and `just run NAME -- ...` for catalog commands. Recipes are thin adapters and require an immediately preceding What/Why comment.

Choose a cohesive VAC and a falsifiable focused check, edit and correct failures, then inspect and stage only that change. Commit through .githooks/pre-commit, which runs the configured exported-index gate with affected test groups plus smoke tests. Do not substitute the full test suite for unmapped commit paths; maintain the impact map instead. Never bypass the hook. Avoid repeating the gate immediately before the same commit unless needed for diagnosis. Finish on a clean feature branch with `just feature-merge`, which runs the full test suite; retain the branch after integration. Branch policy comes from agentrig.yaml; never commit directly to its configured base.

## Memory and skills

Before editing Plan, Archive/Plan, Backlog, Decisions, Invariants or State (including detail files), read the matching edit-* skill. Their canonical bodies live in tooling/worker/assets/skills and are exposed through .agents/skills. Do not create another instruction copy. The configured pre-edit routes offer guidance; opaque shell writes require applying the relevant skill yourself.

When work reveals a concrete improvement outside the current task, proactively use edit-backlog to save it under the configured memory directory, then resume the task. Recording the idea needs no additional approval; it does not authorize planning or implementation. Plan contains only work explicitly selected for execution now in the current session or resumed task. Backlog stores future ideas, improvements and hypotheses without execution priority. A request to save or plan something for later belongs in Backlog. Move it to Plan only when selected to do now; automatically rotate completed Plan rows and cards into Archive/Plan.md and Archive/Plan/ through edit-plan. Never defer work required by current acceptance into Backlog.

State is a factual recovery snapshot. Refresh it at meaningful task/VAC boundaries, blockers and handoffs, with actual verification and a concrete next action; include it in the relevant VAC. Plan owns the selected current outcomes and acceptance. Archive retains completed outcome cards; Git owns VAC history.

D020 governs consolidation: committed decision IDs, statements and detail contents are immutable; application links identify current owners and may migrate with refactoring. Each linked Rust/Python/shell owner needs a real DECISION: DNNN comment. Invariants bind a marked Rust/Python test to its exact configured executable oracle. Use the scaffold guide for schema details; do not duplicate those contracts here.

## Runtime changes

Run `just bootstrap` once after cloning to install the verified revision in tooling/distribution/stable.txt. This build/install recipe is the explicit bootstrap exception to runtime dispatch. Use tooling/worker/run for the installed development runtime; it never builds candidate sources. All command, hook, memory, gate and Git logic belongs to the Rust runtime; adapters only forward calls. Python dependencies support tests and measurements.

Use `just candidate ...` to build and invoke the product under development. The build check and native test fixture select this candidate, while Git/agent hooks and gate orchestration use the installed revision. Change the development pin only after candidate acceptance, then explicitly bootstrap the new pin. Project policy and canonical skill sources remain reviewable repository files. See tooling/distribution/README.md for clean-clone setup and release instructions.

Use `just config-check` for project configuration, `just lint-config-check` for lint settings, and `just lint-rules` for actual rule/language capabilities. Apply the reported repair skill. Preserve policy intent; do not weaken thresholds or selectors merely to pass. New rules and language handlers require implemented measurements, capability declarations, behavioral verification and repair guidance. Consult tooling/worker/README.md for counting semantics and parser limits. Rust changes participate in rustfmt, Clippy and native behavioral tests through the configured gate.
