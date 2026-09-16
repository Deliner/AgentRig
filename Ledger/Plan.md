# Plan

**Plan contains only work explicitly selected for execution now in the current session or its resumed task. Future ideas, improvements and hypotheses belong in [Backlog](Backlog.md), without delivery priority.**

Use [edit-plan](../.agents/skills/edit-plan/SKILL.md) and [execute-plan-feature](../.agents/skills/execute-plan-feature/SKILL.md). A request to save an idea for later does not select it for execution. An interruption or context reset does not discard the current authorized task.

Each new product row references an agreed Backlog outcome and acceptance, leaving implementation and VAC boundaries to the executing agent. Follow edit-backlog before promotion: resolve behavior-changing questions and agree execution order. Preserve the source text and criterion IDs rather than creating a second specification. Keep its agreed Git revision with the source reference for delivery and review. A selected set is executed autonomously without asking again before each card. Row order sequences only the selected current work. Stable PNNN IDs are never reused. Dependencies may refer to current Plan or completed Archive rows; they must exist and be acyclic. Active/complete entries require completed prerequisites. Archived dependencies do not restore old rows to current Plan.

Statuses remain pending, active, paused and complete within current delivery; at most one entry is active. Details contain Feature, User capability and Acceptance, with Delivery required for paused/complete entries. A temporarily blocked current task retains its blocker, branch and resumption condition. If execution is deferred, preserve that context in Backlog.

Automatically rotate completed rows into [Archive/Plan.md](Archive/Plan.md) and their cards into Archive/Plan/ during completion, preserving IDs, acceptance and delivery evidence. The agent performs this through edit-plan without another confirmation, updating references and architecture inventories. Plan is not a completed-work archive; an empty Plan is valid.

P001-P009 are retained in Archive; P010-P016 moved to B001-B007 under the user's current-execution rule. Those P IDs remain retired. Original records are available at Git revision f6cd86efbae8ec501759f0c250b9bd4b76e300d7.

| ID | Status | Depends on | Feature | User capability |
| --- | --- | --- | --- | --- |
| [P018](Plan/018.md) | pending | - | [B001](Backlog/001.md): Исправление конфигурации без блокировки агента | Исправлять конфигурацию в текущей сессии без потери изменений. |
| [P019](Plan/019.md) | pending | - | [B003](Backlog/003.md): Структура проекта, отражающая смысл его возможностей | Находить нужное поведение и сохранять изменения локальными. |
| [P020](Plan/020.md) | pending | P018 | [B006](Backlog/006.md): Единый каталог компонентов и конструктор окружений | Собирать окружения из общих версионируемых компонентов с модульной конфигурацией в одной директории. |
| [P021](Plan/021.md) | pending | P018 | [B002](Backlog/002.md): Согласованная работа хуков и понятные инструкции агенту | Понимать результат действия, требования хуков и управление их поведением. |
| [P022](Plan/022.md) | pending | P021 | [B012](Backlog/012.md): Хук на 50% контекста: соразмерность проверок | Сокращать время проверки гипотез с сохранением достаточности проверки. |
| [P023](Plan/023.md) | pending | P021, P022 | [B013](Backlog/013.md): Хук на 80% контекста: фиксация и исправление проблем рабочего процесса | Исправлять замеченные проблемы после суммаризации и удалять обработанную одноразовую запись. |

Выбрано пользователем 2026-09-11 строго в порядке **B001 → B003 → B006 → B002 → B012 → B013**.
Исходная ревизия контрактов: `30bf0e08faa6053d77b7a4370168ec008c9e3bfb`.
Для B003, B006 и B002 применяются также согласованные 2026-09-11 уточнения
в текущих [B003](Backlog/003.md), [B006](Backlog/006.md) и [B002](Backlog/002.md);
происхождение указано в [P019](Plan/019.md), [P020](Plan/020.md) и [P021](Plan/021.md).
Все шесть карточек имеют статус pending; ни одна ещё не начата.
Открытые вопросы готовности указаны в Delivery и не означают приостановку работы.
Порядок сохраняется; ссылки на другие карточки не выбирают их целиком в работу
и не разрешают менять исходную приёмку. Общая техническая часть, необходимая
для выбранного результата, выполняется внутри него без отдельного параллельного механизма.
