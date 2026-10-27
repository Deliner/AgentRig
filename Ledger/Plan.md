# Plan

**Plan contains only work explicitly selected for execution now in the current session or its resumed task. Future ideas, improvements and hypotheses belong in [Backlog](Backlog.md), without delivery priority.**

Use [edit-plan](../.agents/skills/edit-plan/SKILL.md) and [execute-plan-feature](../.agents/skills/execute-plan-feature/SKILL.md). A request to save an idea for later does not select it for execution. An interruption or context reset does not discard the current authorized task.

Each new product row references an agreed Backlog outcome and acceptance, leaving implementation and VAC boundaries to the executing agent. Follow edit-backlog before promotion: resolve behavior-changing questions and agree execution order. Preserve the source text and criterion IDs rather than creating a second specification. Keep its agreed Git revision with the source reference for delivery and review. A selected set is executed autonomously without asking again before each card. Row order sequences only the selected current work. Stable PNNN IDs are never reused. Dependencies may refer to current Plan or completed Archive rows; they must exist and be acyclic. Active/complete entries require completed prerequisites. Archived dependencies do not restore old rows to current Plan.

Statuses remain pending, active, paused and complete within current delivery; at most one entry is active. Details contain Feature, User capability and Acceptance, with Delivery required for paused/complete entries. A temporarily blocked current task retains its blocker, branch and resumption condition. If execution is deferred, preserve that context in Backlog.

Automatically rotate completed rows into [Archive/Plan.md](Archive/Plan.md) and their cards into Archive/Plan/ during completion, preserving IDs, acceptance and delivery evidence. The agent performs this through edit-plan without another confirmation, updating references and architecture inventories. Plan is not a completed-work archive; an empty Plan is valid.

P001-P009 are retained in Archive; P010-P016 moved to B001-B007 under the user's current-execution rule. Those P IDs remain retired. Original records are available at Git revision f6cd86efbae8ec501759f0c250b9bd4b76e300d7.

| ID | Status | Depends on | Feature | User capability |
| --- | --- | --- | --- | --- |
| [P019](Plan/019.md) | active | - | [B003](Backlog/003.md): Структура проекта, отражающая смысл его возможностей | Находить нужное поведение и сохранять изменения локальными. |
| [P021](Plan/021.md) | pending | P018 | [B002](Backlog/002.md): Согласованная работа хуков и понятные инструкции агенту | Понимать результат действия, требования хуков и управление их поведением. |
| [P025](Plan/025.md) | pending | P021 | [B008](Backlog/008.md): Запрет комментариев кроме ссылок на решения и инварианты | Объяснения и гарантии хранятся в предназначенных для них записях. |
| [P026](Plan/026.md) | pending | - | [B009](Backlog/009.md): Имена условий отражают смысл ветвления | Условия объясняют поведение, а не только удовлетворяют форме линтера. |
| [P027](Plan/027.md) | pending | P021 | [B014](Backlog/014.md): Структурированные инструменты с единым реестром и управляемым удалением | Переиспользовать инструменты, удалять ненужные реализации и сохранять сведения для восстановления без перегрузки контекста. |

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
Выбранный 2026-10-27 [B018](Backlog/018.md) выполнен как [P024](Archive/Plan/024.md);
B001 выполнен как [P018](Archive/Plan/018.md).
Порядок сохраняется; ссылки на другие карточки не выбирают их целиком в работу
и не разрешают менять исходную приёмку. Общая техническая часть, необходимая
для выбранного результата, выполняется внутри него без отдельного параллельного механизма.
