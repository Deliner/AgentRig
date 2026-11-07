# Plan

**Plan contains only work explicitly selected for execution now in the current session or its resumed task. Future ideas, improvements and hypotheses belong in [Backlog](Backlog.md), without delivery priority.**

Use [edit-plan](../.agents/skills/edit-plan/SKILL.md) and [execute-plan-feature](../.agents/skills/execute-plan-feature/SKILL.md). A request to save an idea for later does not select it for execution. An interruption or context reset does not discard the current authorized task.

Each new product row references an agreed Backlog outcome and acceptance, leaving implementation and VAC boundaries to the executing agent. Follow edit-backlog before promotion: resolve behavior-changing questions and agree execution order. Preserve the source text and criterion IDs rather than creating a second specification. Keep its agreed Git revision with the source reference for delivery and review. A selected set is executed autonomously without asking again before each card. Row order sequences only the selected current work. Stable PNNN IDs are never reused. Dependencies may refer to current Plan or completed Archive rows; they must exist and be acyclic. Active/complete entries require completed prerequisites. Archived dependencies do not restore old rows to current Plan.

Statuses remain pending, active, paused and complete within current delivery; at most one entry is active. Details contain Feature, User capability and Acceptance, with Delivery required for paused/complete entries. A temporarily blocked current task retains its blocker, branch and resumption condition. If execution is deferred, preserve that context in Backlog.

Automatically rotate completed rows into [Archive/Plan.md](Archive/Plan.md) and their cards into Archive/Plan/ during completion, preserving IDs, acceptance and delivery evidence; their source Backlog rows and cards move to [Archive/Backlog.md](Archive/Backlog.md) and Archive/Backlog/ at the same time. The agent performs this through edit-plan without another confirmation, updating references. Plan is not a completed-work archive; an empty Plan is valid.

P001-P009 are retained in Archive; P010-P016 moved to B001-B007 under the user's current-execution rule. Those P IDs remain retired. Original records are available at Git revision f6cd86efbae8ec501759f0c250b9bd4b76e300d7.

| ID | Status | Depends on | Feature | User capability |
| --- | --- | --- | --- | --- |
| [P077](Plan/077.md) | active | - | [B068](Backlog/068.md): ord-eq-mismatch — ручной порядок и равенство читают разные поля | Нарушение контракта порядка и равенства ловится на любом Rust-коде. |
| [P078](Plan/078.md) | pending | - | [B071](Backlog/071.md): boolean-expression — длинное логическое выражение | Каждое условие читается с одного взгляда. |
| [P079](Plan/079.md) | pending | - | [B072](Backlog/072.md): file-fan-out — исходящие связи файла | Файл-свалка видна по списку `use`. |
| [P080](Plan/080.md) | pending | - | [B073](Backlog/073.md): embedded-text — большой текст внутри кода | Данные лежат отдельным файлом, код читается как код. |
| [P081](Plan/081.md) | pending | - | [B069](Backlog/069.md): grammar-fork — форк tree-sitter-rust в составе AgentRig | Правильный Rust трёх частых форм разбирается и попадает под правила. |
| [P082](Plan/082.md) | pending | - | [B064](Backlog/064.md): ratchet-baseline — внедрение строгих правил в проект с историей | Строгие правила включаются без выключения гейта. |

История выбора и поставки P017–P070 хранится в Delivery архивных карточек [Archive/Plan](Archive/Plan.md) и в тексте этого файла на Git-ревизии c1935e47a3fe0c1b556692aed4f9c81d1a979fbb.

2026-11-07 пользователь поручил автономно, без вопросов между карточками, выполнить все открытые карточки Backlog
(B047, B048, B050, B051, B064, B068–B074): выбрано **P071 → P082**, порядок за исполнителем — сначала малые
ремонты и расширения действующих правил, затем новые виды правил, затем форк грамматики и ratchet. P076 зависит от
P075: async-контекст обеих карточек определяется одним механизмом. Открытые вопросы карточек о пороге, условии
срабатывания и формате исполнитель решает замером или проектированием при доставке и записывает в Delivery и
решениях; приёмка карточек не меняется. По одной карточке за ветку; новый `kind:` — двухфазно с мид-бранч пином
([D051](Decisions/051.md)).
