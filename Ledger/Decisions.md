# Decisions

Current precedence: [D054](Decisions/054.md) ships testing discipline as the `testing-discipline` norm skill to consumers and makes its statically checkable parts lint rules with invariant oracles, leaving unprovable parts as review norms. [D053](Decisions/053.md) divides delivered capabilities into universal — required in both the `init` delivery and this repository — and worker-local, present only in the worker-building repository, enforced by invariant I020. [D052](Decisions/052.md) requires directory architecture contracts only in the Project source tree, narrowing D028's repository-wide application. [D051](Decisions/051.md) has the agent pin and bootstrap the accepted head of every branch that changes Project before merging it, without asking, deciding when D027 promotes the pin. [D050](Decisions/050.md) moves the Backlog rows and cards of completed outcomes to Archive/Backlog, extending D031's rotation of completed Plan cards. [D049](Decisions/049.md) prints lint warnings only with `--warnings`, superseding D017's visible-by-default warnings. [D047](Decisions/047.md) merges hook requirements into one ordered response, delivers each skill and area once per context window and lets configured routes block until delivered, superseding D014's context-only guidance without delivery state. [D046](Decisions/046.md) lets configuration recovery read its sources and repair skill through `cat`, superseding D045's single permitted shell command. [D044](Decisions/044.md) adds an architecture type to contracts of features without nested features, superseding D043's three-field limit. [D043](Decisions/043.md) limits directory contracts to purpose, files and directories, superseding D028's permission and public-boundary checks. [D042](Decisions/042.md) places first-level features with two-word names directly in Project and adds Python verification configuration to tooling, superseding D041's tooling scope. [D041](Decisions/041.md) develops the worker under Project, superseding D025's placement under tooling/worker. [D035](Decisions/035.md) makes feature-merge select tests affected by the whole branch, superseding D022's complete integration gate for test commands. [D034](Decisions/034.md) replaces systemd scopes with process groups and adds a macOS read-only sandbox. [D033](Decisions/033.md) retires review, delegation, upgrade and its receipt, declarative setup, Mercurial and external VCS, and JavaScript/TypeScript analysis; D023 and D024 remain history and D025 keeps only lint ownership. [D032](Decisions/032.md) requires chat agreement before saving new or revised feature requirements, Backlog-origin Plan contracts and autonomous execution of selected sets; it supersedes immediate idea capture under D031 and new-outcome prerequisite bypasses under D012. [D031](Decisions/031.md) limits Plan to current execution and sends future ideas to unordered Backlog, superseding D012's future queue semantics. [D022](Decisions/022.md) adds actionable verification evidence and recovery facts without new checkpoints. [D021](Decisions/021.md) supersedes D017's warning-only language rollout. [D020](Decisions/020.md) consolidates the runtime and current guidance. D011 retires D003 edit checkpoints; D012 supersedes D009 delivery sequencing. Rows and details D001–D019 record historical choices, not additional active runtimes. Their statements and details are preserved; Applies in points to current owners.

| ID | Decision | Applies in |
| --- | --- | --- |
| [D001](Decisions/001.md) | Separate contextual decisions from executable invariants. | [instructions](../AGENTS.md), [decisions](Decisions.md), [invariants](Invariants.md), [classification skill](../.agents/skills/norm-or-choice/SKILL.md) |
| [D002](Decisions/002.md) | Enforce durable-knowledge links and size policy with one standard-library checker. | [checker](../Project/project_memory/mod.rs), [parser](../Project/project_memory/source.rs), [tests](../Project/project_memory/tests/test_memory.py), [pre-commit](../.agentrig/hooks/pre-commit) |
| [D003](Decisions/003.md) | Block a second agent edit until prior edited contents are committed or reverted. | [current hook dispatch](../Project/agent_hooks/mod.rs), [hooks](../.codex/hooks.json), [tests](../Project/change_delivery/git/tests/test_git.py), [pre-commit](../.agentrig/hooks/pre-commit) |
| [D004](Decisions/004.md) | Use Ruff, mypy, pytest, typos, and Vulture as the Python quality gate. | [configuration](../tooling/pyproject.toml), [gate](../Project/change_delivery/gate/mod.rs), [pre-commit](../.agentrig/hooks/pre-commit) |
| [D005](Decisions/005.md) | Route agent shell operations through catalogued and logged Just recipes. | [instructions](../AGENTS.md), [recipes](../just/commands.just), [catalog](../.agentrig/commands.yaml), [runner](../Project/command_runner/mod.rs), [catalog checker](../Project/project_config/settings/mod.rs), [shell guard](../Project/agent_hooks/guard.rs), [hooks](../.codex/hooks.json), [gate](../Project/change_delivery/gate/mod.rs), [tests](../Project/agent_hooks/tests/test_hooks.py), [pre-commit](../.agentrig/hooks/pre-commit) |
| [D006](Decisions/006.md) | Inject complexity discipline at session start and configured attention intervals. | [instructions](../AGENTS.md), [skill](../.agents/skills/complexity-discipline/SKILL.md), [schedule](../.agents/skills/complexity-discipline/context-reminder.json), [hook](../Project/agent_hooks/reminder.rs), [hooks](../.codex/hooks.json), [tests](../Project/agent_hooks/tests/test_reminders.py) |
| [D007](Decisions/007.md) | Warn before size limits fail and route structural findings to focused refactoring skills. | [instructions](../AGENTS.md), [policy](../Project/structural_lint/mod.rs), [tests](../Project/project_memory/tests/test_memory.py), [file skill](../.agents/skills/refactor-large-file/SKILL.md), [directory skill](../.agents/skills/refactor-large-directory/SKILL.md) |
| [D008](Decisions/008.md) | Keep product contents under Project and durable worker knowledge in its sibling Ledger. | [instructions](../AGENTS.md), [overview](../README.md), [project](../Project/README.md), [plan](Plan.md), [decisions](Decisions.md), [invariants](Invariants.md), [checker](../Project/project_memory/mod.rs), [tests](../Project/project_memory/tests/test_memory.py) |
| [D009](Decisions/009.md) | Drive development through user-facing Plan features while leaving implementation choices to the executing agent. | [instructions](../AGENTS.md), [plan](Plan.md), [execution skill](../.agents/skills/execute-plan-feature/SKILL.md), [checker](../Project/project_memory/mod.rs), [tests](../Project/project_memory/tests/test_memory.py) |
| [D010](Decisions/010.md) | Develop on retained feature branches, rebase divergent work onto current master, and integrate it with an explicit merge. | [instructions](../AGENTS.md), [execution skill](../.agents/skills/execute-plan-feature/SKILL.md), [workflow](../Project/change_delivery/git/mod.rs), [Git integration](../Project/change_delivery/git/integration.rs), [recipes](../just/delivery.just), [catalog](../.agentrig/commands.yaml), [pre-commit](../.agentrig/hooks/pre-commit), [reference hook](../.agentrig/hooks/reference-transaction), [tests](../Project/change_delivery/git/tests/test_git.py) |
| [D011](Decisions/011.md) | Deliver cohesive VACs through the staged commit gate and retire per-edit checkpoint blocking. | [instructions](../AGENTS.md), [overview](../README.md), [execution skill](../.agents/skills/execute-plan-feature/SKILL.md), [current hook dispatch](../Project/agent_hooks/mod.rs), [hooks](../.codex/hooks.json), [pre-commit](../.agentrig/hooks/pre-commit), [tests](../Project/change_delivery/git/tests/test_git.py) |
| [D012](Decisions/012.md) | Adapt outcome-based delivery through validated prerequisites, paused features, justified follow-ups, and retained branch handoffs. | [instructions](../AGENTS.md), [overview](../README.md), [plan](Plan.md), [execution skill](../.agents/skills/execute-plan-feature/SKILL.md), [classification skill](../.agents/skills/norm-or-choice/SKILL.md), [checker](../Project/project_memory/mod.rs), [plan tests](../Project/project_memory/tests/test_plan.py), [repository tests](../Project/project_memory/tests/test_memory.py), [branch tests](../Project/change_delivery/git/tests/test_git.py) |
| [D013](Decisions/013.md) | Preserve a compact current-work snapshot and reconcile it with Git and contracts on resume. | [state](State.md), [instructions](../AGENTS.md), [overview](../README.md), [execution skill](../.agents/skills/execute-plan-feature/SKILL.md), [state skill](../.agents/skills/edit-state/SKILL.md), [checker](../Project/project_memory/mod.rs), [resume](../Project/project_memory/resume.rs), [dispatcher](../Project/agent_hooks/mod.rs), [state tests](../Project/project_memory/tests/test_state.py), [hook tests](../Project/agent_hooks/tests/test_hooks.py) |
| [D014](Decisions/014.md) | Route Ledger edit guidance and existing checks through one shared hook entrypoint. | [instructions](../AGENTS.md), [overview](../README.md), [hooks](../.codex/hooks.json), [dispatcher](../Project/agent_hooks/mod.rs), [plan skill](../.agents/skills/edit-plan/SKILL.md), [decision skill](../.agents/skills/edit-decisions/SKILL.md), [invariant skill](../.agents/skills/edit-invariants/SKILL.md), [state skill](../.agents/skills/edit-state/SKILL.md), [hook tests](../Project/agent_hooks/tests/test_hooks.py), [VAC tests](../Project/change_delivery/git/tests/test_git.py) |
| [D015](Decisions/015.md) | Execute hook services and Git guards in a compatible native Rust runtime. | [runtime](../Project/command_line/main.rs), [hooks](../Project/agent_hooks/mod.rs), [launcher](../tooling/worker/run), [registration](../.codex/hooks.json), [pre-commit](../.agentrig/hooks/pre-commit), [reference hook](../.agentrig/hooks/reference-transaction), [gate](../Project/change_delivery/gate/mod.rs), [native tests](../Project/agent_hooks/tests/test_hooks.py), [instructions](../AGENTS.md) |
| [D016](Decisions/016.md) | Configure native structural rules and gate repair skills through versioned TOML. | [config](../.agentrig/lint.yaml), [engine](../Project/structural_lint/mod.rs), [registry](../Project/structural_lint/catalog/mod.rs), [gate](../Project/change_delivery/gate/mod.rs), [Ledger checker](../Project/project_memory/mod.rs), [tests](../Project/structural_lint/tests/test_lint.py), [configuration skill](../.agents/skills/repair/SKILL.md), [guide](../Project/README.md) |
| [D017](Decisions/017.md) | Analyze Rust and Python syntax for named conditions, function size and input counts with configurable enforcement. | [handlers](../Project/structural_lint/languages/mod.rs), [registry](../Project/structural_lint/catalog/mod.rs), [engine](../Project/structural_lint/mod.rs), [schema](../Project/structural_lint/config/mod.rs), [configuration](../.agentrig/lint.yaml), [tests](../Project/structural_lint/languages/tests/test_language.py), [guide](../Project/README.md), [instructions](../AGENTS.md), [configuration skill](../.agents/skills/repair/SKILL.md), [condition skill](../.agents/skills/name-if-condition/SKILL.md), [function skill](../.agents/skills/refactor-long-function/SKILL.md), [parameter skill](../.agents/skills/reduce-parameters/SKILL.md) |
| [D018](Decisions/018.md) | Share config and target validation between lint and a standalone command, rejecting unsupported language selection. | [selection](../Project/structural_lint/selection/mod.rs), [registry](../Project/structural_lint/catalog/mod.rs), [engine](../Project/structural_lint/mod.rs), [schema](../Project/structural_lint/config/mod.rs), [CLI](../Project/command_line/main.rs), [recipes](../just/lint.just), [catalog](../.agentrig/commands.yaml), [tests](../Project/structural_lint/config/tests/test_config_check.py), [guide](../Project/README.md), [instructions](../AGENTS.md), [skill](../.agents/skills/repair/SKILL.md) |
| [D019](Decisions/019.md) | Keep only native hook and structural lint implementations, preserving behavior through direct native tests and explicit historical link migration. | [hooks](../Project/agent_hooks/mod.rs), [Git guards](../Project/change_delivery/git/mod.rs), [checker](../Project/project_memory/mod.rs), [test adapter](../Project/conftest.py), [tests](../Project/agent_hooks/tests/test_reminders.py), [guide](../Project/README.md), [instructions](../AGENTS.md) |
| [D020](Decisions/020.md) | Use one configured native runtime and canonical guidance, preserving decision history while keeping application links current. | [configuration](../.agentrig/project.yaml), [runtime](../Project/command_line/main.rs), [history](../Project/project_memory/history.rs), [hooks](../Project/agent_hooks/mod.rs), [instructions](../AGENTS.md), [skills](../.agents/skills/norm-or-choice/SKILL.md), [tests](../Project/project_memory/tests/test_memory.py) |
| [D021](Decisions/021.md) | Block language-rule violations at the existing limits and repair all selected source. | [policy](../.agentrig/lint.yaml), [installed defaults](../Project/project_setup/lint.rs), [verification](../Project/project_setup/tests/test_lint_defaults.py), [guide](../Project/README.md) |
| [D022](Decisions/022.md) | Connect VAC outcomes to actionable checks and content-bound recovery evidence without another task registry. | [diagnostics](../Project/shared/diagnostics/mod.rs), [evidence](../Project/project_memory/evidence/mod.rs), [recovery](../Project/project_memory/resume.rs), [execution skill](../.agents/skills/execute-plan-feature/SKILL.md), [repair skill](../.agents/skills/repair/SKILL.md), [guide](../Project/project_setup/README.md) |
| [D023](Decisions/023.md) | Record installation ownership and shipped checksums so upgrades can distinguish stock files from local changes. | [successor](Decisions/033.md) |
| [D024](Decisions/024.md) | Apply one explicit release transition with reviewed conflicts and a recoverable technical journal. | [successor](Decisions/033.md) |
| [D025](Decisions/025.md) | Develop lint and review as portable worker capabilities while consumer projects own their configuration and source. | [instructions](../AGENTS.md), [overview](../README.md), [worker capabilities](../Project/README.md) |
| [D026](Decisions/026.md) | Use strict YAML for AgentRig configuration without runtime format fallback; migrate legacy settings explicitly. | [codec](../Project/shared/yaml/mod.rs), [configuration guide](../Project/project_setup/README.md) |
| [D027](Decisions/027.md) | Develop AgentRig using an explicitly pinned installed runtime while testing candidate source independently. | [launcher](../tooling/worker/run), [bootstrap](../tooling/distribution/bootstrap.sh), [instructions](../AGENTS.md) |
| [D028](Decisions/028.md) | Constrain measured source dependencies with directory-owned YAML contracts and enforce each crossed architectural boundary. | [contracts](../Project/structural_lint/architecture/policy.rs), [architecture guide](../Project/README.md) |
| [D029](Decisions/029.md) | Apply the existing flat-memory registry size exemption to Plan while retaining complete memory and architecture validation. | [lint policy](../.agentrig/lint.yaml) |
| [D030](Decisions/030.md) | Apply flat-memory registry size policy to Backlog while retaining idea-card and architecture validation. | [lint policy](../.agentrig/lint.yaml) |
| [D031](Decisions/031.md) | Keep Plan limited to explicitly selected current execution and retain future ideas in unordered Backlog. | [instructions](../AGENTS.md), [Plan](Plan.md), [Backlog](Backlog.md), [Archive](Archive/Plan.md), [planning skill](../.agents/skills/edit-plan/SKILL.md), [idea skill](../.agents/skills/edit-backlog/SKILL.md), [execution skill](../.agents/skills/execute-plan-feature/SKILL.md), [archive validation](../Project/project_memory/mod.rs), [archive scenarios](../Project/project_memory/tests/test_plan.py), [consumer instructions](../Project/project_setup/assets.rs), [guide](../Project/project_setup/README.md) |
| [D032](Decisions/032.md) | Agree business behavior and observable acceptance in chat before Backlog and autonomous Plan delivery. | [instructions](../AGENTS.md), [Backlog](Backlog.md), [Plan](Plan.md), [card skill](../.agents/skills/edit-backlog/SKILL.md), [planning skill](../.agents/skills/edit-plan/SKILL.md), [execution skill](../.agents/skills/execute-plan-feature/SKILL.md), [consumer instructions](../Project/project_setup/assets.rs), [guide](../Project/project_setup/README.md) |
| [D033](Decisions/033.md) | Retire capabilities outside the lint-first target state and keep Git-only delivery. | [requests](Requests.md), [instructions](../AGENTS.md), [Git](../Project/shared/vcs/mod.rs), [initialization](../Project/project_setup/initialization/mod.rs), [languages](../Project/structural_lint/languages/registry.rs), [configuration](../.agentrig/project.yaml) |
| [D034](Decisions/034.md) | Contain jobs with process groups and sandbox read-only commands per platform. | [requests](Requests.md), [process groups](../Project/command_runner/jobs/group.rs), [identity](../Project/command_runner/jobs/identity/mod.rs), [sandbox](../Project/command_runner/sandbox.rs), [jobs guide](../Project/README.md), [CI](../.github/workflows/ci.yml) |
| [D035](Decisions/035.md) | Select affected tests for the whole branch at feature merge. | [requests](Requests.md), [instructions](../AGENTS.md), [selection](../Project/change_delivery/gate/selection.rs), [merge](../Project/change_delivery/git/mod.rs), [checks guide](../Project/change_delivery/gate/README.md), [execution skill](../.agents/skills/execute-plan-feature/SKILL.md) |
| [D036](Decisions/036.md) | Limit README and AGENTS.md size and width, and keep other Markdown out of Project. | [requests](Requests.md), [repository rules](../.agentrig/lint.yaml), [rule kinds](../Project/structural_lint/rules/descriptor.rs), [measurements](../Project/structural_lint/mod.rs), [consumer defaults](../Project/project_setup/lint.rs), [skill](../.agents/skills/condense-markdown/SKILL.md), [tests](../Project/structural_lint/tests/mod.rs) |
| [D037](Decisions/037.md) | Allow only decision and invariant references as source comments. | [requests](Requests.md), [backlog idea](Archive/Backlog/008.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [measurement](../Project/structural_lint/languages/mod.rs), [Rust](../Project/structural_lint/languages/rust.rs), [Python](../Project/structural_lint/languages/python.rs), [skill](../.agents/skills/replace-comment/SKILL.md), [reference format](../Project/shared/syntax/mod.rs), [record existence](../Project/project_memory/mod.rs), [tests](../Project/structural_lint/languages/tests/mod.rs) |
| [D038](Decisions/038.md) | Keep project configuration in single-responsibility modules under .agentrig instead of agentrig.yaml. | [requests](Requests.md), [settings](../Project/project_config/settings/mod.rs), [configuration guide](../Project/project_setup/README.md), [repository modules](../.agentrig/project.yaml), [tests](../Project/project_config/tests/test_configuration.py) |
| [D039](Decisions/039.md) | Keep Just recipes in grouped files under just/ imported by the root justfile. | [requests](Requests.md), [entry](../justfile), [repository groups](../just/commands.just), [template](../Project/project_setup/template.rs), [configuration guide](../Project/project_setup/README.md), [tests](../Project/project_setup/initialization/tests/test_layout.py) |
| [D040](Decisions/040.md) | Initialize memory in Ledger and sources in Project by default. | [requests](Requests.md), [template](../Project/project_setup/template.rs), [configuration guide](../Project/project_setup/README.md), [tests](../Project/project_setup/initialization/tests/test_layout.py) |
| [D041](Decisions/041.md) | Develop the worker under Project and keep only development launchers and distribution under tooling. | [requests](Requests.md), [instructions](../AGENTS.md), [overview](../README.md), [product](../Project/architecture.yaml), [tooling](../tooling/distribution/README.md), [launcher](../tooling/worker/build), [bootstrap](../tooling/distribution/bootstrap.sh) |
| [D042](Decisions/042.md) | Place first-level features with two-word names directly in Project, shared support in Project/shared and Python verification configuration in tooling. | [requests](Requests.md), [instructions](../AGENTS.md), [overview](../README.md), [product](../Project/architecture.yaml), [crate](../Project/lib.rs), [shared support](../Project/shared/architecture.yaml), [tooling](../tooling/pyproject.toml) |
| [D043](Decisions/043.md) | Limit directory contracts to purpose, files and directories, with directory cycles as the only dependency finding. | [requests](Requests.md), [contracts](../Project/structural_lint/architecture/policy.rs), [architecture guide](../Project/structural_lint/architecture/README.md), [repair skill](../.agents/skills/refactor-large-directory/SKILL.md) |
| [D044](Decisions/044.md) | Make features without nested features declare an architecture type with required parts and inward dependencies. | [card](Archive/Backlog/018.md), [features](../Project/structural_lint/architecture/features.rs), [contracts](../Project/structural_lint/architecture/policy.rs), [settings](../Project/structural_lint/rules/architecture.rs), [architecture guide](../Project/structural_lint/architecture/README.md), [repair skill](../.agents/skills/refactor-large-directory/SKILL.md) |
| [D045](Decisions/045.md) | Keep configuration repair and its check available while the configuration is invalid. | [card](Archive/Backlog/001.md), [hooks](../Project/agent_hooks/recovery.rs), [sources](../Project/project_config/settings/mod.rs), [validation](../Project/project_config/settings/validation.rs), [repair skill](../.agents/skills/repair/SKILL.md) |
| [D046](Decisions/046.md) | Let configuration recovery read its sources and repair skill through `cat` and accept Codex patch payloads. | [card](Archive/Backlog/001.md), [hooks](../Project/agent_hooks/recovery.rs), [patch payloads](../Project/agent_hooks/mod.rs), [repair skill](../.agents/skills/repair/SKILL.md) |
| [D047](Decisions/047.md) | Merge hook requirements into one ordered response, deliver each skill and area once per context window and let configured routes block until delivered. | [card](Archive/Backlog/002.md), [dispatcher](../Project/agent_hooks/mod.rs), [coordinator](../Project/agent_hooks/requirements.rs), [session state](../Project/agent_hooks/reminder.rs), [check entry](../Project/agent_hooks/guard.rs), [route settings](../Project/project_config/settings/mod.rs), [order check](../Project/project_config/settings/validation.rs), [tests](../Project/agent_hooks/tests/requirements.rs), [routes](../.agentrig/hooks.yaml), [instructions](../AGENTS.md) |
| [D048](Decisions/048.md) | Apply the flat-memory registry size exemption to the Plan archive while retaining complete memory and architecture validation. | [lint policy](../.agentrig/lint.yaml) |
| [D049](Decisions/049.md) | Print and serialize lint warnings only when `--warnings` is requested. | [engine](../Project/structural_lint/mod.rs), [arguments](../Project/structural_lint/cli.rs), [tests](../Project/structural_lint/tests/test_lint.py), [guide](../Project/README.md), [setup guide](../Project/project_setup/README.md) |
| [D050](Decisions/050.md) | Move the Backlog row and card of each completed outcome to Archive/Backlog together with its Plan card. | [memory checker](../Project/project_memory/mod.rs), [tests](../Project/project_memory/tests/test_backlog.py), [plan skill](../.agents/skills/edit-plan/SKILL.md), [backlog skill](../.agents/skills/edit-backlog/SKILL.md), [instructions](../AGENTS.md), [shipped instructions](../Project/project_setup/assets.rs), [archive](Archive/Backlog.md) |
| [D051](Decisions/051.md) | Pin and bootstrap the gate-accepted head of every branch that changes Project before merging it, without asking. | [instructions](../AGENTS.md), [distribution guide](../tooling/distribution/README.md) |
| [D052](Decisions/052.md) | Require directory architecture contracts only in the Project source tree. | [lint policy](../.agentrig/lint.yaml), [architecture guide](../Project/structural_lint/architecture/README.md), [plan skill](../.agents/skills/edit-plan/SKILL.md), [tools skill](../.agents/skills/develop-tools/SKILL.md) |
| [D053](Decisions/053.md) | Split delivered capabilities into universal, required in both the init delivery and this repository, and worker-local. | [card](Archive/Backlog/021.md), [manifest and guard](../Project/shared/testing/test_delivery.py), [norm](Invariants.md), [oracle](../.agentrig/oracles.yaml) |
| [D054](Decisions/054.md) | Ship testing discipline as a norm skill, enforcing its statically checkable parts. | [skill](../Project/shared/skills/testing-discipline/SKILL.md), [delivery](../Project/project_setup/assets.rs), [repository instructions](../AGENTS.md) |
| [D055](Decisions/055.md) | Keep test placement (TC1) a review norm; no static rule. | [skill](../Project/shared/skills/testing-discipline/SKILL.md), [superseded consequence](Decisions/054.md) |
| [D056](Decisions/056.md) | Apply the flat-memory registry size exemption to the Backlog archive while retaining complete memory validation. | [lint policy](../.agentrig/lint.yaml) |
| [D057](Decisions/057.md) | Keep test public-surface discipline (TC2/TC4) a review norm; no static rule. | [skill](../Project/shared/skills/testing-discipline/SKILL.md), [superseded consequence](Decisions/054.md) |
| [D058](Decisions/058.md) | Reject coverage-based test-placement enforcement; TC1 stays a review norm. | [skill](../Project/shared/skills/testing-discipline/SKILL.md), [kept norm](Decisions/055.md) |
| [D059](Decisions/059.md) | Static TC2/TC4 require a type-resolving frontend; keep the review norm. | [skill](../Project/shared/skills/testing-discipline/SKILL.md), [kept norm](Decisions/057.md) |
| [D060](Decisions/060.md) | Run Clippy as a dev-gate with a panic-risk deny set, excluding print lints because the CLI's output is the product. | [card](Archive/Backlog/027.md), [clippy profile](../.agentrig/commands.yaml), [ruff profile](../tooling/pyproject.toml) |
| [D061](Decisions/061.md) | Limit Rust and Python control-flow nesting depth per function (finding at depth 5+), with else-if and elif flat and nested functions measured on their own; delivered to consumers. | [card](Archive/Backlog/029.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [measurement](../Project/structural_lint/languages/mod.rs), [Rust](../Project/structural_lint/languages/rust.rs), [Python](../Project/structural_lint/languages/python.rs), [skill](../Project/shared/skills/flatten-nesting/SKILL.md), [tests](../Project/structural_lint/languages/tests/mod.rs) |
| [D062](Decisions/062.md) | Ban boolean parameters in Rust and Python at error level (primitive `bool`, or Python `bool`-annotated or `True`/`False`-defaulted; not fields, `self` or untyped), naming the two modes with an enum or a string label; delivered to consumers. | [card](Archive/Backlog/029.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [measurement](../Project/structural_lint/languages/mod.rs), [Rust](../Project/structural_lint/languages/rust.rs), [Python](../Project/structural_lint/languages/python.rs), [skill](../Project/shared/skills/split-boolean-parameter/SKILL.md), [tests](../Project/structural_lint/languages/tests/mod.rs) |
| [D063](Decisions/063.md) | Budget decision points per Rust and Python function at error level (finding above 5), counting each if, else-if and ternary and each match or case as one multi-way decision, with nested functions measured on their own; delivered to consumers. | [card](Archive/Backlog/029.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [measurement](../Project/structural_lint/languages/mod.rs), [Rust](../Project/structural_lint/languages/rust.rs), [Python](../Project/structural_lint/languages/python.rs), [skill](../Project/shared/skills/control-flow/reduce-branching/SKILL.md), [tests](../Project/structural_lint/languages/tests/mod.rs) |
| [D064](Decisions/064.md) | Enable the ruff bandit (`S`) group for Python in the development gate, scoping the assert and subprocess noise checks (S101/S603/S607) to test code while keeping the full group on production code; a dev-gate profile, not an init capability. | [card](Archive/Backlog/026.md), [ruff profile](../tooling/pyproject.toml), [related](Decisions/060.md) |
| [D065](Decisions/065.md) | Require a `DECISION:` or `INVARIANT:` reference on the line above every Rust `unsafe` block or `unsafe` function at error level, or remove the `unsafe`; Rust only, delivered to consumers. | [card](Archive/Backlog/026.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [skill](../Project/shared/skills/references/justify-unsafe/SKILL.md), [tests](../Project/structural_lint/languages/tests/mod.rs) |
| [D066](Decisions/066.md) | Accept the `command_runner/jobs` libc FFI `unsafe` calls as sound — arguments built to the libc contract, returns and `errno` checked, no aliasing or borrowed pointer outliving the call — under one shared justification. | [rule](Decisions/065.md), [macOS identity](../Project/command_runner/jobs/identity/macos.rs), [Linux identity](../Project/command_runner/jobs/identity/linux.rs), [process signals](../Project/command_runner/jobs/process.rs), [group control](../Project/command_runner/jobs/group.rs) |
| [D067](Decisions/067.md) | Flag string literals matching a fixed set of unambiguous credential formats (private-key block, AWS, GitHub, Google or Slack token) at error level in Rust and Python; delivered to consumers, additive to the ruff bandit group. | [card](Archive/Backlog/026.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [matcher](../Project/shared/syntax/mod.rs), [measurement](../Project/structural_lint/languages/mod.rs), [skill](../Project/shared/skills/remove-secret/SKILL.md), [tests](../Project/structural_lint/languages/tests/mod.rs) |
| [D068](Decisions/068.md) | Deny clippy `let_underscore_must_use` in the development gate so an ignored `Result`/`#[must_use]` must be consciously consumed, catching the Rust half of B028's ignore-errors rule on the dev gate rather than shipping it to init (no type inference); extends D060. | [card](Archive/Backlog/028.md), [clippy profile](../.agentrig/commands.yaml), [ruff profile](../tooling/pyproject.toml), [related](Decisions/060.md) |
| [D069](Decisions/069.md) | Flag a Python `except` clause whose body is only `pass` at error level unless a `DECISION:` or `INVARIANT:` reference sits on the line directly above the `pass`, shipping the clean-syntactic half of B028's silent-error rule to init; Python only, delivered to consumers; the type-dependent Rust half stays on the dev gate (D068). | [card](Archive/Backlog/028.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Python](../Project/structural_lint/languages/python.rs), [skill](../Project/shared/skills/references/handle-error/SKILL.md), [tests](../Project/structural_lint/languages/tests/mod.rs) |
| [D070](Decisions/070.md) | Reject an `architecture.yaml` purpose whose whole meaningful content is a placeholder/generic token (`TODO`, `Module`, `...`) by tightening the existing `directory-architecture` contract validation, shipping B030's purpose-quality check (5.2) to init; no new rule kind or skill, I020 unchanged; extends D028/D043 and the fixed-set approach of D067. | [card](Archive/Backlog/030.md), [owner](../Project/structural_lint/architecture/policy.rs), [tests](../Project/structural_lint/architecture/tests.rs), [related](Decisions/067.md) |
| [D071](Decisions/071.md) | Keep B030's families-together (5.1) and one-capability-one-file (5.3) as norms, not init rules: 5.1 needs trait/base name→file resolution with wildcard/re-export/prelude holes and a new extraction subsystem (D059), and 5.3 is not syntactically identifiable without framework semantics; only the decidable purpose-quality check (5.2, D070) ships. | [card](Archive/Backlog/030.md), [norm](../Project/shared/skills/refactor/refactor-large-directory/SKILL.md), [related](Decisions/059.md), [related](Decisions/070.md) |
| [D072](Decisions/072.md) | Flag a directory whose own name is a generic dumping-ground word (`utils`, `helpers`, `misc`, `common`) at error level via a new `dump-name` rule kind, mechanizing the `refactor-large-directory` norm against reuse buckets; `shared` is allowed; delivered to consumers through `catalog::ALL`; reuses the existing skill so I020 skills are unchanged and only the rule-kind set grows (B031 6.1). | [card](Archive/Backlog/031.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [measurement](../Project/structural_lint/mod.rs), [skill](../Project/shared/skills/refactor/refactor-large-directory/SKILL.md), [tests](../Project/structural_lint/tests/mod.rs) |
| [D073](Decisions/073.md) | Keep B031's meaningful-identifier (6.2) and magic-number (6.3) candidates as norms, not init rules: 6.2 needs scope analysis the syntactic engine lacks and `data`/`tmp`/single letters are often legitimate, and 6.3 cannot separate magic literals from legitimate ones (indices, sizes, status codes, powers of two) — both high-false-positive against the gate's bar (D035); only the decidable dump-name check (6.1, D072) ships. | [card](Archive/Backlog/031.md), [related](Decisions/059.md), [related](Decisions/072.md) |
| [D074](Decisions/074.md) | Flag a silently disabled test — a Rust `#[ignore]` attribute or a Python `@pytest.mark.skip`/`skipif` decorator without a `DECISION:` or `INVARIANT:` reference on the line directly above it — at error level via a new `disabled-test` rule kind reusing the justification mechanism (D065/D069); cross-language, delivered to consumers through `catalog::ALL` with a new `justify-skip` repair skill, so both the rule-kind and skill sets grow by one and stay balanced (I020) (B032 7.1). | [card](Archive/Backlog/032.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [Python](../Project/structural_lint/languages/python.rs), [skill](../Project/shared/skills/references/justify-skip/SKILL.md), [tests](../Project/structural_lint/languages/tests/markers.rs) |
| [D075](Decisions/075.md) | Keep B032's test-determinism (7.2) and naming/placement (7.3) candidates as norms, not init rules: 7.2 cannot tell a benign clock/RNG/network call from a nondeterminism bug and order dependence is not syntactic, and 7.3 is already the `testing-discipline` placement (D055) and behavior-naming norms; both are false-positive-prone or redundant against the gate bar (D035). 7.2 is added to the delivered skill as a determinism point; only the decidable disabled-test check (7.1, D074) ships as a rule. | [card](Archive/Backlog/032.md), [norm](../Project/shared/skills/testing-discipline/SKILL.md), [related](Decisions/055.md), [related](Decisions/074.md) |
| [D076](Decisions/076.md) | Flag a declared external dependency (rust/python module) that no scanned source imports, by extending the existing directory-architecture resolution to aggregate used externals and diff against the declared set minus the std/core/alloc and __future__ seeds; no new rule kind or skill, I020 unchanged; extends D070 and the fixed-set/seed approach of D067 (B033 8.1). | [card](Archive/Backlog/033.md), [settings](../Project/structural_lint/rules/architecture.rs), [resolver](../Project/structural_lint/architecture/runner/analysis.rs), [runner](../Project/structural_lint/architecture/runner/mod.rs), [tests](../Project/structural_lint/architecture/tests/test_architecture.py), [related](Decisions/070.md) |
| [D077](Decisions/077.md) | Keep B033's lockfile discipline (8.2) a configuration norm already satisfied — committed Cargo.lock/uv.lock and --locked on every cargo/uv command, with init shipping no build commands — and the dependency budget (8.3) a review norm needing a soft threshold and per-feature counting the engine lacks; no new rule, skill or invariant, I020 unchanged; only 8.1 (D076) ships. | [card](Archive/Backlog/033.md), [command config](../.agentrig/commands.yaml), [related](Decisions/060.md), [related](Decisions/076.md) |
| [D078](Decisions/078.md) | Add an optional architecture.yaml `zone` field with one class `pure`, whose directory must not reference `std::env`, `std::fs` or `std::process` in its own files, reported at error level by the existing directory-architecture rule; opt-in, additive and delivered to consumers with no new rule kind or skill (I020 unchanged); supersedes D044's four-field contract limit (B034 C1+C2). | [card](Archive/Backlog/034.md), [schema](../Project/structural_lint/architecture/policy.rs), [environment](../Project/structural_lint/architecture/mod.rs), [resolver](../Project/structural_lint/architecture/runner/analysis.rs), [runner](../Project/structural_lint/architecture/runner/mod.rs), [tests](../Project/structural_lint/architecture/tests.rs), [related](Decisions/044.md) |
| [D079](Decisions/079.md) | Add an optional architecture.yaml `dependents` budget capping how many distinct other directories may depend on a directory; the existing directory-architecture rule measures incoming coupling on the dependency graph it already builds for cycles and reports a directory over its budget (default 6) at error level, naming the count and budget; opt-in override, additive, no new rule kind or skill (I020 unchanged); extends D078's contract field set (B035, Набор II C4). | [card](Archive/Backlog/035.md), [schema](../Project/structural_lint/architecture/policy.rs), [runner](../Project/structural_lint/architecture/runner/mod.rs), [tests](../Project/structural_lint/architecture/tests.rs), [related](Decisions/078.md) |
| [D080](Decisions/080.md) | Flag a broad Rust `#[allow(...)]`/`#![allow(...)]` attribute without a `DECISION:` or `INVARIANT:` reference on the line directly above it, at error level via a new `allow-justification` rule kind that unifies the `#[ignore]` attribute dispatch (D074) and reuses the justification mechanism (D065/D069); Rust-only, delivered to consumers through `catalog::ALL` with a new `justify-allow` repair skill, so both the rule-kind and skill sets grow by one and stay balanced (I020) (B036, Набор II C9). | [card](Archive/Backlog/036.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [skill](../Project/shared/skills/references/justify-allow/SKILL.md), [tests](../Project/structural_lint/languages/tests/markers.rs) |
| [D081](Decisions/081.md) | Flag a Rust `dbg!` macro invocation — a leftover debugging artifact that prints and returns its argument — at error level via a new `debug-artifact` rule kind, unconditionally with no justifying-reference escape since a committed `dbg!` is never legitimate (unlike D065/D069/D074/D080); Rust-only, delivered to consumers through `catalog::ALL` with a new `remove-debug` repair skill, so both the rule-kind and skill sets grow by one and stay balanced (I020) (B036, Набор II C9). | [card](Archive/Backlog/036.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [skill](../Project/shared/skills/references/remove-debug/SKILL.md), [tests](../Project/structural_lint/languages/tests/markers.rs) |

| [D082](Decisions/082.md) | Flag a Rust panic path outside test code — a `.unwrap()`/`.expect(...)` call or a `panic!`/`todo!`/`unimplemented!`/`unreachable!` macro — without a `DECISION:` or `INVARIANT:` reference on the line directly above it, at error level via a new `panic-path` rule kind reusing the justification mechanism (D065/D080) and extending the `macro_invocation` dispatch (D081) with a `call_expression` dispatch; test code is excluded by an ancestor `#[cfg(test)]`/`#[test]` walk plus the delivered rule's `**/tests/**` exclude; Rust-only, delivered to consumers through `catalog::ALL` with a new `handle-panic` repair skill, so both the rule-kind and skill sets grow by one and stay balanced (I020); the strict Clippy profile (D060) stays the dev gate and this is the portable baseline (B037, Набор II C6). | [card](Archive/Backlog/037.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [skill](../Project/shared/skills/references/handle-panic/SKILL.md), [tests](../Project/structural_lint/languages/tests/markers.rs) |

| [D083](Decisions/083.md) | Extend the `zone: pure` check so a reference to `std::time::Instant::now` or `std::time::SystemTime::now` in a pure directory's own files is reported at error level like environment access, matched by exact full external-path equality (the resolver already appends the `::now` segment) so `std::time::Duration` and the `UNIX_EPOCH` constant stay legal; the pure-zone finding names the category (wall clock vs environment); opt-in, additive, no new rule kind or skill (I020 unchanged), repair via refactor-large-directory; extends D078's zone behavior (B038, Набор II C3). | [card](Archive/Backlog/038.md), [clock](../Project/structural_lint/architecture/mod.rs), [schema](../Project/structural_lint/architecture/policy.rs), [resolver](../Project/structural_lint/architecture/runner/analysis.rs), [tests](../Project/structural_lint/architecture/tests.rs), [resolver tests](../Project/structural_lint/architecture/resolution/rust/tests.rs), [related](Decisions/078.md) |

| [D084](Decisions/084.md) | Flag a Rust `debug_assert!`/`debug_assert_eq!`/`debug_assert_ne!` macro outside test code — a contract that `cfg(debug_assertions)` strips from release builds, leaving it unchecked where it matters — without a `DECISION:` or `INVARIANT:` reference on the line directly above it, at error level via a new `debug-assertion` rule kind that extends the `macro_invocation` dispatch (D081/D082) and reuses the justification mechanism and `in_test_context` walk (D082); `assert!`/`assert_eq!`/`assert_ne!` are not flagged since they hold in every build; Rust-only, delivered to consumers through `catalog::ALL` with a new `firm-up-assertion` repair skill, so both the rule-kind and skill sets grow by one and stay balanced (I020); the strict Clippy profile (D060) stays the dev gate and this is the portable baseline (B039, Набор II C5). | [card](Archive/Backlog/039.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [skill](../Project/shared/skills/references/firm-up-assertion/SKILL.md), [tests](../Project/structural_lint/languages/tests/markers.rs), [related](Decisions/082.md) |

| [D085](Decisions/085.md) | Flag a glob/wildcard import — a Rust `use path::*` outside test code or a Python `from module import *` — without a `DECISION:` or `INVARIANT:` reference on the line directly above it, at error level via a new `wildcard-import` rule kind reusing the justification mechanism (D065/D082) and, for Rust, the `in_test_context` ancestor walk (D082); a glob binds the file to another module's whole, evolving public surface, so a name added or renamed upstream silently changes what is in scope; delivered to consumers through `catalog::ALL` with a new `narrow-import` repair skill, so both the rule-kind and skill sets grow by one and stay balanced (I020) (B040, Набор II C10). | [card](Archive/Backlog/040.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [Python](../Project/structural_lint/languages/python.rs), [skill](../Project/shared/skills/references/narrow-import/SKILL.md), [tests](../Project/structural_lint/languages/tests/markers.rs), [related](Decisions/082.md) |

| [D086](Decisions/086.md) | Ship C7 anti-duplication as a norm, not a static clone rule: a portable principle in the delivered `complexity-discipline` skill — do not introduce a second representation of behavior that already has an owner; reuse or extend the existing definition instead of copying it — because broad and fuzzy (Type-2/3) and cross-file clone detection needs a tuned corpus/windowing subsystem that is false-positive-prone and conflicts with the fast gate and portable install (D059 family, against the near-zero-FP bar D035), and the only variant below the new-subsystem rung (within-file exact whole-body duplication) fires too rarely to earn a new rule kind, repair skill and per-file aggregation pass; no rule kind, skill-for-a-rule or invariant is added, so the I020 delivery sets stay balanced, and the existing size budgets, dump-name and directory-architecture checks (D070/D079) already pressure duplication's enabling conditions (B041, Набор II C7). | [card](Archive/Backlog/041.md), [skill](../Project/shared/skills/complexity-discipline/SKILL.md), [related](Decisions/059.md) |

| [D087](Decisions/087.md) | Ship C8 anti-over-complication as a coverage map, not a new rule: over-complication is the area AgentRig already covers most densely, so each decidable dimension keeps its existing shipped owner — `function-lines`, `nesting-depth` (D061), `branch-budget` (D063), `parameter-count`, `boolean-parameter` (D062), `named-if-condition`, `file-size`/`directory-entries`, `dump-name` (D072) and `directory-architecture` fan-in/type/cycle/purpose checks (D070/D079) — while design discipline rides the delivered `complexity-discipline` norm; a new C8 rule would duplicate that coverage and adding redundant machinery is itself over-complication, one-variant-enum detection is false-positive-prone (`#[non_exhaustive]`/FFI) against the near-zero-FP bar D035, and the residual smells (speculative generality, needless indirection, dead code) are semantic or need the rejected type/data-flow frontend (D059); no rule kind, skill-for-a-rule or invariant is added, so the I020 delivery sets stay balanced and no Project file changes (no D051 pin), completing Набор II C1–C10 (B042, Набор II C8). | [card](Archive/Backlog/042.md), [skill](../Project/shared/skills/complexity-discipline/SKILL.md), [related](Decisions/086.md) |

| [D088](Decisions/088.md) | Flag a Rust function parameter type or return type that is a tuple, outside test code, without a `DECISION:` or `INVARIANT:` reference on the line directly above the function, at error level via a new `tuple-as-record` rule kind that runs inside the existing `function_item`/`function_signature_item` dispatch and reuses the justification mechanism (D065/D082) and `in_test_context` walk (D082); a tuple in a signature is an unnamed record that evades the `parameter-count` and `boolean-parameter` (D062) budgets and hides what each position means, while a unit `()` and a tuple nested in a generic (`Option<(A, B)>`) are not flagged; Rust-only, delivered to consumers through `catalog::ALL` with a new `name-the-record` repair skill, so both the rule-kind and skill sets grow by one and stay balanced (I020) (B043, Tier A). | [card](Backlog/043.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [skill](../Project/shared/skills/references/name-the-record/SKILL.md), [tests](../Project/structural_lint/languages/tests/markers.rs), [related](Decisions/082.md) |

| [D089](Decisions/089.md) | Flag a Rust `.unwrap_or`, `.ok_or` or `.map_or` call whose eager first argument propagates with `?` (a `try_expression`), outside test code, without a `DECISION:` or `INVARIANT:` reference on the line directly above it, at error level via a new `eager-fallback` rule kind extending the `call_expression` dispatch (D082) and reusing the justification mechanism and `in_test_context` walk (D082); the fallback argument is evaluated unconditionally, so a `?` inside it propagates even when the value is present — the search ignores a `?` in a deferred `closure_expression` (the lazy `*_else` form); Rust-only, delivered through `catalog::ALL` with a new `eager-to-lazy` repair skill, so both the rule-kind and skill sets grow by one and stay balanced (I020) (B043, Tier A). | [card](Backlog/043.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [skill](../Project/shared/skills/references/eager-to-lazy/SKILL.md), [tests](../Project/structural_lint/languages/tests/markers.rs), [related](Decisions/082.md) |

| [D090](Decisions/090.md) | Flag a Rust bare `expr.ok();` statement that drops a `Result`, outside test code, without a `DECISION:` or `INVARIANT:` reference on the line directly above it, at error level via a new `discarded-result` rule kind on a new `expression_statement` dispatch, reusing the justification mechanism (D065/D069/D082) and `in_test_context` walk (D082); it closes the Rust gap in `silent-error` (D069, Python-only) and the strict Clippy dev gate (D060, which catches `let _ =` but not `.ok();`), while a kept value (`let x = expr.ok()`), an assignment and a propagated `expr.ok()?` are not flagged and the high-FP `Err(_) =>` arm form stays a norm; Rust-only, delivered through `catalog::ALL` with a new `handle-discarded` repair skill, so both the rule-kind and skill sets grow by one and stay balanced (I020) (B043, Tier A). | [card](Backlog/043.md), [repository rule](../.agentrig/lint.yaml), [rule kind](../Project/structural_lint/rules/descriptor.rs), [Rust](../Project/structural_lint/languages/rust.rs), [skill](../Project/shared/skills/references/handle-discarded/SKILL.md), [tests](../Project/structural_lint/languages/tests/markers.rs), [related](Decisions/082.md) |

| [D091](Decisions/091.md) | On the attach-failure error path of process teardown, deliberately discard the best-effort `child.kill()` and `child.wait()` `Result`s with a justified `.ok();` rather than propagate or log them — the child may already have exited, the attach error being returned is the failure that matters, the strict Clippy dev gate (D060) rejects `let _ =` and `drop(...)` risks `clippy::drop_non_drop`; a context-specific choice under the new `discarded-result` rule (D090), dogfooded during B043 Tier A delivery, not a universal norm. | [process](../Project/command_runner/jobs/process.rs), [related](Decisions/090.md) |

D081 ships the second of B036's two C9 cores: a leftover Rust `dbg!` macro.
`dbg!` prints its argument's file, line and value and returns it, for a
momentary local inspection that is never deliberate shipped behavior; whether a
call is `dbg!` is decidable from syntax alone — a `macro_invocation` whose macro
path is `dbg` — with no symbol resolution (D035). Unlike unsafe (D065), a
swallowed error (D069), a disabled test (D074) or a broad allow (D080), a
committed `dbg!` has no legitimate use, so the finding is unconditional and
carries no justifying-reference escape, keeping the detector simpler than the
justification family. The gate's Clippy already denies `clippy::dbg_macro` here,
but a consumer need not run that configuration, so the `debug-artifact` kind
ships through `catalog::ALL` with a new `remove-debug` skill in the `references`
group, growing the delivered and live rule-kind and skill sets by one and
keeping them equal (I020). Keying on the fixed macro name mirrors
hardcoded-secret (D067) and dump-name (D072); `println!`/`eprintln!` remain the
D060 choice, not debug artifacts. With D080 it completes B036's static C9 cores.

D080 ships the first of B036's two C9 cores (Набор II, local consistency and
self-description): a broad Rust `allow` attribute without a justifying reference.
A `#[allow(...)]` or `#![allow(...)]` switches a compiler or Clippy lint off for
its scope, so an agent can quiet a rule instead of fixing the code; whether the
attribute is an `allow` and whether a reference sits on the line directly above
it is decidable from syntax alone (D035), and an attribute is not a comment, so
the comment ban (D037) does not reach it. The Rust detector folds the `#[ignore]`
(D074) and `#[allow]` handling into one attribute dispatch keyed on the attribute
name — covering both the outer `attribute_item` and the inner
`inner_attribute_item` — and reuses the `justified` reference-escape helper
(D065/D069/D074). The new `allow-justification` kind ships through `catalog::ALL`
and a new `justify-allow` repair skill in the `references` group, so the
delivered and live rule-kind and skill sets each grow by one and stay equal
(I020). Python `# noqa`/`# type: ignore`, commented-out code and bare
`TODO`/`FIXME` stay under the comment ban (D037); `println!`/`eprintln!` remain
the D060 choice; directory self-description stays under D070; the `dbg!`
debug-artifact core is recorded separately. It supersedes nothing.

D079 ships B035's locality feature (Набор II, C4) and extends D078's contract
field set: the architecture.yaml contract now takes an optional `dependents`
budget — the number of distinct other directories allowed to depend on this
one. The directory-architecture engine already builds the cross-directory
dependency graph to detect cycles (D043); it now also counts each directory's
distinct dependents on that graph and reports one whose incoming coupling
exceeds its budget at error level, naming the count and budget. The default
budget is 6, the tightest value that keeps every ordinary module in this
repository green; the designed shared hubs (`shared/paths`, `shared/vcs`,
`shared/testing`, `shared/diagnostics`, `shared/yaml`, `project_config`) declare
their real radius, so the blast radius of a change is visible and bounded in the
contract. Same-directory and module-declaration edges are not coupling. It is a
stricter check inside the existing rule, not a new `Kind` or skill, so the
delivered rule-kind and skill sets are unchanged (I020), mirroring D070/D076/D078.
D044's type classification and D078's zone field remain in force.

D078 ships B034's merged zone-and-environment feature (Набор II, C1+C2) and
supersedes D044's four-field contract limit: the architecture.yaml contract now
takes an optional `zone` field whose one class, `pure`, forbids a directory's
own files from referencing `std::env`, `std::fs` or `std::process`. The
directory-architecture resolver already preserved each reference's external
origin (D076); the engine now collects those that reach the three wholesale
effectful namespaces and the contract rejects them in a `pure` directory at
error level. One class is the complexity-discipline minimum that proves the
mechanism with a real consumer; `std::net`, `std::time` and `std::io` stay out
because each holds pure value types that would be false positives. It is a
stricter check inside the existing rule, not a new `Kind` or skill, so the
delivered rule-kind and skill sets are unchanged (I020), mirroring D070/D076.
The repository tags `structural_lint/architecture/resolution` as the first pure
module and stays green. D044's type classification remains in force.

D077 records why two of B033's candidates stay norms and supersedes nothing: 8.2
(lockfile and `--locked` discipline) is already true — `Project/Cargo.lock` and
`tooling/uv.lock` are committed, every cargo and uv command in
`.agentrig/commands.yaml` and the build launcher passes `--locked`, and `init`
ships no build commands — so there is nothing to ship or newly enforce beyond the
command configuration that owns it (D005/D060); 8.3 (a dependency-count budget) is
a soft, project-specific threshold needing per-feature aggregation the syntactic
engine does not compute, so a blanket limit would be noise against the gate bar
(D035). A pretend invariant for either is forbidden by norm-or-choice. Only the
decidable never-imported check (8.1, D076) ships. With 8.1 and 8.2/8.3,
B033/P042 — the last card of the clusters-1–8 quality set — is complete.

D076 ships B033's dependency-hygiene check (8.1) and supersedes nothing: the
`directory-architecture` rule now flags any declared `external` module that no
scanned source imports, the mirror of the undeclared-external error it already
raised. Resolution already computed an external origin per reference but dropped
it on the Rust path; the engine now preserves and aggregates the first segment of
every resolved external into a used-set and reports each declared external that is
neither used nor a seed member on the feature owner. Two seeds are exempt:
`std`/`core`/`alloc` as the always-available Rust baseline, and `__future__`
because `from __future__ import …` parses as a `future_import_statement` the
inspector does not record (a tree-sitter blind spot that would otherwise be a
false positive). It is a stricter check inside the existing rule, not a new
`Kind`, so the delivered rule-kind and skill sets are unchanged (I020) and the
repair skill stays `refactor-large-directory`, mirroring D070. This repository
dropped seven genuinely unused Python externals (contextlib, copy, hashlib, io,
re, tarfile, tomllib) to go green.

D075 records why two of B032's candidates stay norms and supersedes nothing: 7.2
(a test is deterministic) cannot be flagged syntactically — a call to a clock, RNG
or the network is not by itself a bug, a fixture legitimately uses them, and order
dependence is not a syntactic property — so a blanket rule would be
false-positive-prone against the gate's bar (D035); 7.3 (name and place a test by
behavior) is already the `testing-discipline` placement norm (D055, point 1) and
its behavior-naming guidance (point 5). 7.2 is added to that delivered skill as a
determinism point (point 7) so the agreed outcome has visible guidance, and 7.3
needs nothing new. No rule kind, skill or invariant is added, so the I020 delivery
sets are unchanged. Only the decidable disabled-test check (7.1) became a rule
(D074).

D074 ships B032's disabled-test discipline (7.1) and supersedes nothing: the lint
now flags any Rust `#[ignore]` attribute or Python `@pytest.mark.skip`/`skipif`
decorator that disables a test, since such a marker drops the behavior from the
suite without a trace. The escape mirrors unsafe-justification (D065) and
except-pass (D069) — a `DECISION:` or `INVARIANT:` reference on the line directly
above records why the skip is sound, otherwise the author must re-enable or delete
the test (skill `justify-skip`). The Python half matches a small fixed set of
decorator paths (`pytest.mark.skip`, `pytest.mark.skipif`), keeping it decidable
and near-zero-false-positive (D035). It ships to consumers through `catalog::ALL`
and the delivered per-kind template, and the new `justify-skip` repair skill joins
the `references` group, so the delivered and live skill sets both grow by one and
stay equal while the rule-kind set grows too (I020). AgentRig has no disabled
tests, so dogfooding is clean. B032's 7.2 (nondeterminism) and 7.3
(name/placement) stay norms.

D073 records why two of B031's candidates stay norms and supersedes nothing:
6.2 (identifiers carry meaning) would need scope analysis the syntactic engine does
not do to tell a legitimate loop-local or short-lived name from a meaningless
module-level one, and `data`, `tmp` and single letters are common legitimate names;
6.3 (no magic numbers) cannot separate a genuinely magic literal from a legitimate
index, size, status code, time unit or power of two without context, which is why
clippy and ruff keep magic-value lints in opt-in noisy groups, and forcing a named
constant for every literal is itself noise. Both would be high-false-positive
against the gate's near-zero bar (D035), so only the decidable dump-name check
(6.1, D072) became a rule.

D072 mechanizes the `refactor-large-directory` norm against reuse buckets and
supersedes nothing: the lint now flags any directory whose own basename is one of
`utils`, `helpers`, `misc` or `common`, which the norm already rejected but nothing
enforced. `shared` is deliberately allowed as the home for genuine cross-feature
leaves. The check reads the directory name from the inventory, so it is decidable
and type-free (D035); a small fixed set keeps it near-zero-false-positive and this
repository has no such directory. It ships to consumers through `catalog::ALL` and
the delivered per-kind template, reuses the existing `refactor-large-directory`
skill so the delivered skill set is unchanged and only the rule-kind set grows
(I020), and mirrors the fixed-set approach of D067/D070 (B031 6.1).

D071 records why two of B030's candidates stay norms and supersedes nothing:
5.1 (a trait with its impls, or a base with its subclasses, co-located) would need
to resolve each impl's trait or each class's base by name to its defining file,
which the syntactic engine cannot do for wildcard, re-export or prelude names and
which would need a new extraction subsystem — the cost D059 already declined; 5.3
(one capability per file) cannot identify "a CLI command, rule or handler" without
framework-specific semantics, and the generic proxy would flag legitimate cohesion
the facade guidance allows. Both intents already live in the feature-type and
contract discipline (D043/D044) and `refactor-large-directory`. Only the decidable
purpose-quality check (5.2) became a rule (D070).

D070 ships the init half of B030's structure-as-map discipline (5.2) and
supersedes nothing: the `directory-architecture` contract loader now rejects a
purpose whose whole meaningful content is a placeholder or generic token (`TODO`,
`Module`, `...`), on top of the empty and multi-line checks it already made
(D028/D043). A small fixed set keeps the check decidable and
near-zero-false-positive — the repository's real purposes are multi-word phrases,
and a phrase merely containing a generic word still passes — mirroring the
hardcoded-secret fixed set (D067). It is a stricter check inside the existing rule,
not a new `Kind`, so the delivered rule-kind and skill sets are unchanged (I020)
and the repair skill stays `refactor-large-directory`. B030's families-together
(5.1) and one-capability-one-file (5.3) are not shipped as rules — they need
symbol resolution or are declarative — and remain norms.

D069 ships the init half of B028's error discipline and supersedes nothing: the
lint now flags any Python `except` clause whose body is only `pass`, since such a
handler swallows its error without a trace. The escape mirrors
unsafe-justification — a `DECISION:` or `INVARIANT:` reference on the line
directly above the `pass` records why the swallow is sound, otherwise the author
must handle or re-raise the error (skill `handle-error`). The rule is purely
syntactic and high-signal (D035), so it ships to consumers through `catalog::ALL`;
its Rust counterpart `let _ = <Result>` stays on the dev gate (D068) because it
needs type inference the engine omits (D059). AgentRig has no such handler, so
the repository is already clean.

D068 records the Rust, development-gate half of B028's error discipline and
supersedes nothing: the clippy profile (D060) now denies
`let_underscore_must_use`, so `let _ = <Result>` is a gate error alongside the
`unused_must_use` already caught by `-D warnings`. By the user's 2026-10-30
choice this stays on the dev gate rather than shipping to init, because proving
the bound expression is a `Result` needs type inference the syntactic engine
omits (D059). The resource discipline of 3.3 (ruff `SIM115`) is already provided
by the enabled `SIM` group, so it needs no change. AgentRig's one
`let _ = <Result>` (best-effort teardown in `command_runner/jobs/process.rs`)
now uses `.ok()`.

D067 adds B026's last rule and supersedes nothing: the gate now flags any string
literal matching a fixed set of unambiguous credential formats — a private-key
block, or an AWS, GitHub, Google or Slack token — in Rust and Python at error
level. It is additive to the ruff bandit group (D064), which catches Python
credentials by variable name, whereas this matches by value in both languages;
broader entropy scanning stays out as too false-positive-prone for a gate. The
worker has no such literals, so dogfooding is clean. The remove-secret repair
skill is added top-level, and the three refactor-* skills are grouped under
`refactor/` to keep the skills directory within its size limit. With 1.1
(D065/D066), 1.2 (D064) and 1.3 delivered, B026 (P037) is complete.

D066 records why B026's unsafe-justification rule leaves AgentRig's own `unsafe`
in place rather than removing it: the eleven FFI calls in `command_runner/jobs`
are libc process-identity sampling and process-group control that no safe std
API exposes. Each builds its arguments to the libc signature, checks the return
or `errno`, and reads only scalars or a struct the call fully initializes, with
no borrowed pointer outliving the call, so one shared soundness reason is
referenced at every site. It supersedes nothing.

D065 adds B026's first rule and supersedes nothing: the gate now requires a
`DECISION:` or `INVARIANT:` reference directly above each Rust `unsafe` block or
`unsafe` function, reusing the comment-reference marker parser (D037) so the
check stays syntactic. Rust only, since Python has no `unsafe`. The justify-unsafe
repair skill joins a new `references` skill group beside replace-comment — the
two rules built on the reference marker — keeping the skills directory within its
size limit. The worker's eleven libc FFI blocks are justified by D066.

D064 scopes B026's Python bandit group to this repository's development gate: all
914 initial `S` findings were the assert and CLI-spawning test mechanism
(S101/S603/S607) in test code, so those three are exempt there while the full
group guards production Python; like D060 it hardens AgentRig's own code rather
than `init`, which ships no ruff, and records the scoping as a choice, not a
weakened selector.

D063 adds the last of B029's complexity lints and supersedes nothing: a function
over five decision points is a finding, each if, else-if and ternary counts one
and each match or case counts one however many arms it has, and the threshold is
read from the worker's own distribution rather than set to spare it. With 4.1,
4.2 and 4.3 delivered, P036 is complete.

D062 adds B029's boolean-parameter lint and supersedes nothing: a `bool`
parameter is a finding in Rust and Python, the worker's own 33 sites become named
mode enums or string labels built at the call site, and `Capture` moves into
`command_runner::jobs` so the directory dependency graph stays acyclic.

D061 adds the first of B029's complexity lints and supersedes nothing: nesting depth is measured apart from function size and input count, else-if and elif stay flat, a nested function or closure is its own scope, and the depth-4 boundary reuses the parameter-count line rather than a weakened one.

D060 scopes B027 to this repository's development gate: the `init` template ships no external tools, so the Clippy panic-risk profile (`panic`/`todo`/`unimplemented`/`dbg_macro`/`unreachable`/`expect_used`/`unwrap_used`) and the ruff `PERF` group harden AgentRig's own code rather than `init`. It supersedes nothing and records the `print_stdout`/`print_stderr` exclusion as a choice (CLI output is the product), not a weakened threshold.

D059 records the B025 semantic-analyzer alternative to static TC2/TC4 as rejected on the current tree-sitter engine; it reinforces D057's norm and names a type-resolving frontend (rust-analyzer-class for Rust, type inference for Python) as the measured lever, without changing the engine.

D058 records the B024 coverage alternative to TC1 as rejected; TC1 remains the D055 review norm, with D057 (TC2/TC4) leaving no part of testing discipline a static rule on the current engine.

D057 supersedes D054's consequence that TC2/TC4 become lint rules with invariant oracles; D054's norm-skill delivery and enforcement-split principle remain. With D055 (TC1), no part of testing discipline is a static rule on the current engine.

D055 supersedes D054's consequence that TC1 becomes a lint rule with an invariant; D054's norm-skill delivery and enforcement-split principle remain.

D047 supersedes D014's context-only route guidance without delivery state; D014's single entrypoint and Ledger guidance without a denial remain.

D046 supersedes D045's single permitted shell command; D045's other recovery rules remain.

D025 supersedes D008 for this worker repository; the historical D008 statement and detail remain unchanged.

D041 supersedes D025's placement: the worker lives under Project and tooling keeps only launchers and distribution.

D042 supersedes D041's tooling scope: tooling also holds the Python verification configuration.

D043 supersedes D028's allow, deny and public checks; D028's directory-owned contracts and measured cycle graph remain.

D044 supersedes D043's three-field contract limit; D043's removal of allow, deny and public remains.

D026 supersedes D016's TOML format choice for AgentRig configuration; P004 delivers the strict YAML transition. External tool formats remain externally owned.

D027 supersedes automatic candidate rebuilding for active development commands and hooks. D015's native runtime ownership remains in force.
