# Plan

**Plan contains only work explicitly selected for execution now in the current session or its resumed task. Future ideas, improvements and hypotheses belong in [Backlog](Backlog.md), without delivery priority.**

Use [edit-plan](../.agents/skills/edit-plan/SKILL.md) and [execute-plan-feature](../.agents/skills/execute-plan-feature/SKILL.md). A request to save an idea for later does not select it for execution. An interruption or context reset does not discard the current authorized task.

Each new product row references an agreed Backlog outcome and acceptance, leaving implementation and VAC boundaries to the executing agent. Follow edit-backlog before promotion: resolve behavior-changing questions and agree execution order. Preserve the source text and criterion IDs rather than creating a second specification. Keep its agreed Git revision with the source reference for delivery and review. A selected set is executed autonomously without asking again before each card. Row order sequences only the selected current work. Stable PNNN IDs are never reused. Dependencies may refer to current Plan or completed Archive rows; they must exist and be acyclic. Active/complete entries require completed prerequisites. Archived dependencies do not restore old rows to current Plan.

Statuses remain pending, active, paused and complete within current delivery; at most one entry is active. Details contain Feature, User capability and Acceptance, with Delivery required for paused/complete entries. A temporarily blocked current task retains its blocker, branch and resumption condition. If execution is deferred, preserve that context in Backlog.

Automatically rotate completed rows into [Archive/Plan.md](Archive/Plan.md) and their cards into Archive/Plan/ during completion, preserving IDs, acceptance and delivery evidence; their source Backlog rows and cards move to [Archive/Backlog.md](Archive/Backlog.md) and Archive/Backlog/ at the same time. The agent performs this through edit-plan without another confirmation, updating references. Plan is not a completed-work archive; an empty Plan is valid.

P001-P009 are retained in Archive; P010-P016 moved to B001-B007 under the user's current-execution rule. Those P IDs remain retired. Original records are available at Git revision f6cd86efbae8ec501759f0c250b9bd4b76e300d7.

| ID | Status | Depends on | Feature | User capability |
| --- | --- | --- | --- | --- |
| [P028](Plan/028.md) | pending | - | [B019](Backlog/019.md): Структурная дисциплина из коробки | Новый проект сразу получает контракты структуры, запрет циклов и дисциплину «прочитай структуру до правки». |
| [P029](Plan/029.md) | pending | - | [B020](Backlog/020.md): Markdown-дисциплина из коробки | Markdown нового проекта держится в тех же рамках, что в этом репозитории. |
| [P030](Plan/030.md) | pending | P028, P029 | [B021](Backlog/021.md): Статический страж докфудинга | Universal-возможность нельзя незаметно потерять с одной стороны. |

Выбрано пользователем 2026-09-11 в порядке B001 → B003 → B006 → B002 → B012 → B013.
2026-10-27 пользователь удалил из Backlog ненужные карточки, включая B006, B012
и B013, и поручил выполнить оставшиеся: B001, B002, B003, B008, B009 и B014.
P020, P022 и P023 сняты вместе с источниками; их ID повторно не используются.
Порядок по зависимостям: **B001 → B003 → B002 → B008 → B009 → B014**.
Ревизии контрактов указаны в карточках Plan.
Удаление ревью ([D033](Decisions/033.md)) заменяет ссылки карточек на существующий
runner ревью перед merge: смысловую оценку по тем же критериям выполняет
исполнитель при приёмке; новый runner не создаётся. Ссылки на удалённые карточки
сохранены текстом без перехода.
Открытые вопросы готовности указаны в Delivery и не означают приостановку работы.
Выбранный 2026-10-27 [B018](Archive/Backlog/018.md) выполнен как [P024](Archive/Plan/024.md);
B001 выполнен как [P018](Archive/Plan/018.md), B003 — как [P019](Archive/Plan/019.md),
B002 — как [P021](Archive/Plan/021.md), B008 — как [P025](Archive/Plan/025.md),
B009 — как [P026](Archive/Plan/026.md), B014 — как [P027](Archive/Plan/027.md).
Порядок сохраняется; ссылки на другие карточки не выбирают их целиком в работу
и не разрешают менять исходную приёмку. Общая техническая часть, необходимая
для выбранного результата, выполняется внутри него без отдельного параллельного механизма.

2026-10-28 по итогам аудита докфудинга пользователь согласовал B019, B020 и B021
и поручил их реализовать. Порядок по зависимостям: **B019 → B020 → B021**
(P028 → P029 → P030); P030 зависит от P028 и P029. Ревизии контрактов — в карточках
Plan. Доставка идёт на ветке feature/dogfood-init.
