# Plan

**Plan contains only work explicitly selected for execution now in the current session or its resumed task. Future ideas, improvements and hypotheses belong in [Backlog](Backlog.md), without delivery priority.**

Use [edit-plan](../.agents/skills/edit-plan/SKILL.md) and [execute-plan-feature](../.agents/skills/execute-plan-feature/SKILL.md). A request to save an idea for later does not select it for execution. An interruption or context reset does not discard the current authorized task.

Each new product row references an agreed Backlog outcome and acceptance, leaving implementation and VAC boundaries to the executing agent. Follow edit-backlog before promotion: resolve behavior-changing questions and agree execution order. Preserve the source text and criterion IDs rather than creating a second specification. Keep its agreed Git revision with the source reference for delivery and review. A selected set is executed autonomously without asking again before each card. Row order sequences only the selected current work. Stable PNNN IDs are never reused. Dependencies may refer to current Plan or completed Archive rows; they must exist and be acyclic. Active/complete entries require completed prerequisites. Archived dependencies do not restore old rows to current Plan.

Statuses remain pending, active, paused and complete within current delivery; at most one entry is active. Details contain Feature, User capability and Acceptance, with Delivery required for paused/complete entries. A temporarily blocked current task retains its blocker, branch and resumption condition. If execution is deferred, preserve that context in Backlog.

Automatically rotate completed rows into [Archive/Plan.md](Archive/Plan.md) and their cards into Archive/Plan/ during completion, preserving IDs, acceptance and delivery evidence; their source Backlog rows and cards move to [Archive/Backlog.md](Archive/Backlog.md) and Archive/Backlog/ at the same time. The agent performs this through edit-plan without another confirmation, updating references. Plan is not a completed-work archive; an empty Plan is valid.

P001-P009 are retained in Archive; P010-P016 moved to B001-B007 under the user's current-execution rule. Those P IDs remain retired. Original records are available at Git revision f6cd86efbae8ec501759f0c250b9bd4b76e300d7.

| ID | Status | Depends on | Feature | User capability |
| --- | --- | --- | --- | --- |
| [P037](Plan/037.md) | active | - | [B026](Backlog/026.md): линты безопасности (обоснование `unsafe`, ruff `S`, скан секретов), догфуд к зелёному | Из коробки нет необоснованного `unsafe` и типовых дыр безопасности |

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
Plan. Доставка выполнена на ветке feature/dogfood-init: B019–B021 реализованы как
P028–P030 и перенесены в [Archive/Plan](Archive/Plan.md).
2026-10-28 пользователь выбрал [B022](Archive/Backlog/022.md) на исполнение одной
фичей P031 (тестовая дисциплина) на ветке feature/test-discipline. При реализации
оба статических кандидата (TC1/TC2) отклонены как неосуществимые на текущем движке
([D055](Decisions/055.md)/[D054](Decisions/054.md)); дисциплина доставлена
навыком-нормой. P031 выполнен и перенесён в [Archive/Plan](Archive/Plan.md).
2026-10-28 пользователь согласовал и выбрал на исполнение набор улучшений: B3 (D056),
C1 (индекс норм) и карточки [B023](Archive/Backlog/023.md)/[B024](Archive/Backlog/024.md).
B023 — как [P032](Archive/Plan/032.md) на ветке feature/test-visibility-norm: по
исследованию движка оба статических кандидата TC2/TC4 отклонены как неосуществимые без
ложных срабатываний ([D057](Decisions/057.md)); дисциплина остаётся нормой навыка,
отданной потребителю (P031). P032 выполнен и перенесён в
[Archive/Plan](Archive/Plan.md), B023 — в [Archive/Backlog](Archive/Backlog.md).
Ревизия контракта — в карточке P032.
B024 — как [P033](Archive/Plan/033.md) на ветке feature/test-placement-norm: спайк
показал отсутствие инструментов покрытия в обоих тулчейнах, а содержательно покрытие
переотмечает законный случай [D055](Decisions/055.md) (предмет vs попутно исполненный
общий код) и добавляет медленную динамическую подсистему поверх быстрого статического
гейта. Кандидат на правило отклонён ([D058](Decisions/058.md)); TC1 остаётся нормой
навыка, отданной потребителю (P031). P033 выполнен и перенесён в
[Archive/Plan](Archive/Plan.md), B024 — в [Archive/Backlog](Archive/Backlog.md).
Ревизия контракта — в карточке P033. Набор B3/C1/B023/B024 завершён.
2026-10-28 пользователь оформил и сразу выбрал на исполнение [B025](Backlog/025.md)
(«сохраняй и бери в работу сразу») — семантический анализ символов, снимающий барьер
для статических TC2/TC4 (D057) и усиливающий сигнал для TC1 (D058). Промоутнуто в
[P034](Archive/Plan/034.md) на ветке feature/semantic-symbols; объём большой, начинается
спайком по приёмке B025. Ревизия контракта — в карточке P034.
Спайк показал: чистых статических TC2/TC4 на синтаксическом движке (tree-sitter) нет —
вызовы метода по получателю и ссылки-значения требуют вывода типов и лексического
резолвинга, которые движок намеренно опускает, поэтому отсутствие продуктовых ссылок на
символ недоказуемо. Единственный рычаг — тип-резолвящий фронтенд, чья цена конфликтует с
быстрым гейтом (D035) и портативной установкой. По выбору пользователя (вариант A)
принято решение [D059](Decisions/059.md): TC2/TC4 остаются нормой (D057), движок не
меняется, цена фронтенда записана. P034 выполнен и перенесён в
[Archive/Plan](Archive/Plan.md), B025 — в [Archive/Backlog](Archive/Backlog.md).
2026-10-29 пользователь согласовал набор улучшений качества кода (кластеры 1–8,
[B026](Backlog/026.md)–[B033](Backlog/033.md)) и поручил реализовать его через догфудинг,
по одной карточке. Первой выбран [B027](Backlog/027.md) — промоутнут в [P035](Plan/035.md)
на ветке feature/external-profiles. Порядок остальных — за исполнителем; ревизии
контрактов — в карточках. Выполнение автономное, без повторного запроса между карточками.
P035 (B027) выполнен: профиль clippy — паник-риск-deny без print-линтов
([D060](Decisions/060.md)), ruff `PERF`; код приведён к зелёному (VAC-A..D). P035 перенесён
в [Archive/Plan](Archive/Plan.md), B027 — в [Archive/Backlog](Archive/Backlog.md); пин
stable.txt=`ace5511` ([D051](Decisions/051.md)). Остальные карточки набора (B026, B028–B033)
— следующими, по одной за ветку.
2026-10-30 P036 (B029) выполнен: три под-правила сложности доставлены — 4.1 nesting-depth
([D061](Decisions/061.md)), 4.2 boolean-parameter ([D062](Decisions/062.md)), 4.3
branch-budget ([D063](Decisions/063.md)); наш код приведён к зелёному. P036 перенесён в
[Archive/Plan](Archive/Plan.md), B029 — в [Archive/Backlog](Archive/Backlog.md); ссылки card
в D061/D062/D063 переведены на `Archive/Backlog/029.md`; пин stable.txt на голову ветки
([D051](Decisions/051.md)). Остаются карточки набора B026, B028, B030–B033.
