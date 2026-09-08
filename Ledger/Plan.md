# Plan

Each row defines a product outcome: a capability, a substantial MVP result, or an observable improvement such as optimization. Details state required functionality, user capability, and acceptance without prescribing architecture or a technical task list. The executing agent chooses implementation and VAC boundaries.

Use row order for intended delivery priority and stable IDs for identity; inserting a prerequisite does not renumber existing features. `Depends on` contains `-` or comma-separated IDs such as `P002, P003`. Dependencies identify required prior outcomes, must exist, and must not form cycles. An active or complete feature has only complete prerequisites.

Statuses: `pending` is planned, `active` is being delivered, `paused` is explicitly suspended, and `complete` has verified acceptance. At most one feature is active. Zero active features is valid when finished, waiting, or between deliveries; it does not authorize inventing or starting more work.

Each detail has `Feature`, `User capability`, and `Acceptance` sections in that order. An optional final `Delivery` section records the source of new work, plan changes, or acceptance evidence. It is required for paused and complete features: explain a blocker's evidence and retained branch/resumption condition when paused, or record the acceptance checks and results when complete. The checker validates structure and dependencies; the executing agent evaluates the meaning and evidence.

Add a prerequisite before a paused feature when a separate outcome is necessary. Add justified follow-ups after the current feature while continuing it; use a dependency on the current feature only when that outcome is actually required. User or authorized manager instructions can add future work without interrupting the current VAC. Do not postpone work required by current acceptance into a follow-up and still mark the current feature complete. See the [execution skill](../.agents/skills/execute-plan-feature/SKILL.md) for branch handoff and resumption.

| ID | Status | Depends on | Feature | User capability |
| --- | --- | --- | --- | --- |
| [P001](Plan/001.md) | complete | - | Isolated review MCP | Request reproducible multi-reviewer assessment of an exact project revision and receive one validated report. |
| [P002](Plan/002.md) | complete | P001 | Portable worker capabilities and setup | Configure and prepare a consuming project with shipped lint/review, safely repeat setup and update or roll back the environment. |
| [P003](Plan/003.md) | complete | P002 | Discoverable lint and managed delegation | Configure rules and specialized workers, inspect owned processes, and obtain isolated verified reports, artifacts and code changes. |
| [P004](Plan/004.md) | complete | P003 | Composable AgentRig environments | Assemble, install and migrate YAML-configured project and delegate environments from reusable packages, with custom skills and hooks. |
| [P005](Plan/005.md) | complete | P004 | Stable development and public distribution | Develop AgentRig with an independently installed verified runtime and obtain the portable worker from GitHub under MIT. |
| [P006](Plan/006.md) | complete | P005 | Directory architecture contracts and lint | Describe directory responsibilities and boundaries, detect forbidden dependencies and cycles across languages, and receive architectural repair guidance. |
| [P007](Plan/007.md) | complete | P005 | Extensible version control support | Use AgentRig with Git and another VCS, and connect a private implementation without publishing its code. |
| [P008](Plan/008.md) | complete | P005 | Selectable agent harnesses | Prepare project and delegate workflows for Codex and Claude Code from shared environment declarations. |
| [P009](Plan/009.md) | complete | P006 | Complete architecture maps and feature ownership | Maintain checked file/directory responsibilities and dependency boundaries, and apply them to AgentRig itself. |
| [P010](Plan/010.md) | pending | P005 | Recoverable configuration errors | Diagnose and repair invalid worker configuration without hook deadlock or discarding unfinished project edits. |
| [P011](Plan/011.md) | pending | P008, P009, P010 | Coordinated hooks and timely architecture guidance | Receive relevant architectural skills before edits and coherent guidance when multiple hook requirements overlap or conflict. |
| [P012](Plan/012.md) | pending | P011 | Cohesive AgentRig ownership and reliable development flows | Maintain feature-owned code with explicit shared boundaries and correct the cross-capability omissions identified in the repository audit. |
| [P013](Plan/013.md) | pending | P012 | Complete lint coverage and executable language evaluations | Apply all existing AgentRig rules to Python, TypeScript, Rust, JavaScript, Go, Java and Kotlin and verify actual detection through a dedicated evaluation environment. |
| [P014](Plan/014.md) | pending | P011, P012 | Explainable and verifiable consumer workflows | Explain effective hooks, skills and checks, verify installed workflows in independent projects, and recover predictably after interrupted operations. |
| [P015](Plan/015.md) | pending | P004, P012 | Git-backed component catalog and shared environment constructor | Register and version reusable components and assemble developer rigs, reviewers, advisers and delegates from common YAML recipes. |
