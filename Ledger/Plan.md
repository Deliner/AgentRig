# Plan

**Plan contains only work explicitly selected for execution now in the current session or its resumed task. Future ideas, improvements and hypotheses belong in [Backlog](Backlog.md), without delivery priority.**

Use [edit-plan](../.agents/skills/edit-plan/SKILL.md) and [execute-plan-feature](../.agents/skills/execute-plan-feature/SKILL.md). A request to save an idea for later does not select it for execution. An interruption or context reset does not discard the current authorized task.

Each new product row references an agreed Backlog outcome and acceptance, leaving implementation and VAC boundaries to the executing agent. Follow edit-backlog before promotion: resolve behavior-changing questions and agree execution order. Preserve the source text and criterion IDs rather than creating a second specification. Keep its agreed Git revision with the source reference for delivery and review. A selected set is executed autonomously without asking again before each card. Row order sequences only the selected current work. Stable PNNN IDs are never reused. Dependencies may refer to current Plan or completed Archive rows; they must exist and be acyclic. Active/complete entries require completed prerequisites. Archived dependencies do not restore old rows to current Plan.

Statuses remain pending, active, paused and complete within current delivery; at most one entry is active. Details contain Feature, User capability and Acceptance, with Delivery required for paused/complete entries. A temporarily blocked current task retains its blocker, branch and resumption condition. If execution is deferred, preserve that context in Backlog.

Automatically rotate completed rows into [Archive/Plan.md](Archive/Plan.md) and their cards into Archive/Plan/ during completion, preserving IDs, acceptance and delivery evidence; their source Backlog rows and cards move to [Archive/Backlog.md](Archive/Backlog.md) and Archive/Backlog/ at the same time. The agent performs this through edit-plan without another confirmation, updating references. Plan is not a completed-work archive; an empty Plan is valid.

P001-P009 are retained in Archive; P010-P016 moved to B001-B007 under the user's current-execution rule. Those P IDs remain retired. Original records are available at Git revision f6cd86efbae8ec501759f0c250b9bd4b76e300d7.

| ID | Status | Depends on | Feature | User capability |
| --- | --- | --- | --- | --- |
| [P049](Plan/049.md) | active | - | [B040](Backlog/040.md): правило `wildcard-import` — glob-импорт (Rust `use …::*` вне тестов, Python `from … import *`) без ссылки-обоснования — находка | Защита поверхности импортов от молчаливой эволюции API (C10) |

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
[B026](Archive/Backlog/026.md)–[B033](Backlog/033.md)) и поручил реализовать его через догфудинг,
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
2026-10-30 P037 (B026) выполнен: три линта безопасности доставлены — 1.1 unsafe-justification
([D065](Decisions/065.md)/[D066](Decisions/066.md)), 1.2 ruff bandit `S`
([D064](Decisions/064.md)), 1.3 hardcoded-secret ([D067](Decisions/067.md)); наш код приведён к
зелёному. P037 перенесён в [Archive/Plan](Archive/Plan.md), B026 — в
[Archive/Backlog](Archive/Backlog.md); ссылки card в D064/D065/D067 переведены на
`Archive/Backlog/026.md`; пин stable.txt на голову ветки ([D051](Decisions/051.md)). Остаются
карточки набора B028, B030–B033.
2026-10-30 P038 (B028) выполнен: дисциплина ошибок и ресурсов. 3.1 — Python-половина уехала в
поставку `init` структурным правилом `silent-error` ([D069](Decisions/069.md)): `except`-тело
из одного `pass` без ссылки-обоснования — находка; Rust-половина (`let _ = <Result>`) и 3.2 —
на dev-гейте через clippy `let_underscore_must_use` ([D068](Decisions/068.md)); 3.3 (ресурсы)
уже обеспечено группой ruff `SIM`. Наш код приведён к зелёному. P038 перенесён в
[Archive/Plan](Archive/Plan.md), B028 — в [Archive/Backlog](Archive/Backlog.md); ссылки card в
D068/D069 переведены на `Archive/Backlog/028.md`; пин stable.txt на голову ветки
([D051](Decisions/051.md)). Остаются карточки набора B030–B033.
2026-10-30 следующей выбран [B030](Archive/Backlog/030.md) — промоутнут в [P039](Archive/Plan/039.md) на ветке
feature/structure-map. По анализу: 5.2 (качество `purpose`) — чисто синтаксически, расширением
правила `directory-architecture`; 5.1 (семьи вместе) и 5.3 (одна возможность = один файл) —
оцениваются спайком, вероятно норма (символьный/типовой резолвинг либо декларативность, D059).
Порядок остальных (B031–B033) — за исполнителем; ревизии контрактов — в карточках.
2026-10-30 P039 (B030) выполнен: 5.2 — качество `purpose` доставлено в `init` ужесточением
правила `directory-architecture` ([D070](Decisions/070.md)): болванка/дженерик-`purpose` —
находка; 5.1 (семьи вместе) и 5.3 (одна возможность = один файл) — норма
([D071](Decisions/071.md): символьный резолвинг с дырами и объём D059; декларативность). Наш код
приведён к зелёному. P039 перенесён в [Archive/Plan](Archive/Plan.md), B030 — в
[Archive/Backlog](Archive/Backlog.md); ссылки card в D070/D071 переведены на
`Archive/Backlog/030.md`; пин stable.txt на голову ветки ([D051](Decisions/051.md)). Остаются
карточки набора B031–B033.
2026-10-30 следующей выбран [B031](Backlog/031.md) — промоутнут в [P040](Plan/040.md) на ветке
feature/naming-lints. По анализу: 6.1 (имена-помойки) — чисто синтаксически, новым правилом
`dump-name` (каталог с именем `utils`/`helpers`/`misc`/`common`; `shared` легитимен), ремонт —
существующий `refactor-large-directory` (навык не добавляется, I020 по навыкам не меняется);
6.2 (осмысленные идентификаторы) и 6.3 (магические числа) — вероятно норма (требуют анализа
области видимости / по природе шумят, D035). Порядок остальных (B032–B033) — за исполнителем;
ревизии контрактов — в карточках.
2026-10-30 P040 (B031) выполнен: 6.1 — имена-помойки доставлены в `init` новым правилом
`dump-name` ([D072](Decisions/072.md)): каталог `utils`/`helpers`/`misc`/`common` — находка,
`shared` легитимен; навык ремонта переиспользован (`refactor-large-directory`, I020 по skills не
меняется). 6.2 (осмысленные идентификаторы) и 6.3 (магические числа) — норма
([D073](Decisions/073.md): scope-анализ / шум магических чисел против near-zero-FP бара D035).
Наш код приведён к зелёному. P040 перенесён в [Archive/Plan](Archive/Plan.md), B031 — в
[Archive/Backlog](Archive/Backlog.md); ссылки card в D072/D073 переведены на
`Archive/Backlog/031.md`; пин stable.txt на голову ветки ([D051](Decisions/051.md)). Остаются
карточки набора B032–B033.
2026-10-30 следующей выбран [B032](Backlog/032.md) — промоутнут в [P041](Plan/041.md) на ветке
feature/test-lints. По анализу: 7.1 (тихо выключенные тесты) — чисто синтаксически, новым правилом
`disabled-test` (Rust `#[ignore]`, Python `@pytest.mark.skip`/`skipif` без ссылки-обоснования на
строке прямо над ними), переиспользуя механизм `justified` (D065/D069); новый навык ремонта в
группе `references`. 7.2 (недетерминизм в тестах) и 7.3 (имена/размещение) — вероятно норма
(детекция «тестового файла» и запретных вызовов/сети высоколожна; размещение уже норма TC1, D055).
Порядок остальных (B033) — за исполнителем; ревизии контрактов — в карточках.
2026-10-30 P041 (B032) выполнен: 7.1 — тихо выключенные тесты доставлены в `init` новым правилом
`disabled-test` ([D074](Decisions/074.md)): Rust `#[ignore]` и Python `@pytest.mark.skip`/`skipif`
без ссылки-обоснования на строке прямо над маркером — находка, переиспользует механизм `justified`
(D065/D069); новый навык ремонта `justify-skip` в группе `references` (I020: и виды правил, и навыки
растут на один — наборы сбалансированы). 7.2 (детерминизм) и 7.3 (имена/размещение) — норма
([D075](Decisions/075.md)): 7.2 добавлен пунктом детерминизма в навык `testing-discipline`, 7.3 уже
покрыт его нормами размещения (D055) и именования. Наш код приведён к зелёному (в репозитории нет
выключенных тестов). P041 перенесён в [Archive/Plan](Archive/Plan.md), B032 — в
[Archive/Backlog](Archive/Backlog.md); ссылки card в D074/D075 переведены на `Archive/Backlog/032.md`;
пин stable.txt на голову ветки ([D051](Decisions/051.md)). Остаётся карточка набора B033.
2026-10-30 последней выбран [B033](Backlog/033.md) — промоутнут в [P042](Plan/042.md) на ветке
feature/dep-hygiene. По анализу: 8.1 (объявленные, но неиспользуемые external-зависимости) — чисто
синтаксически, расширением правила `directory-architecture` (агрегировать использованные external из
резолвинга и вычесть из объявленных минус seed std/core/alloc), как в D070; новый вид/навык не нужен.
8.2 (локфайлы + `--locked`) уже обеспечено конфигом команд и нормой (`init` не поставляет команды
сборки, D060), поэтому — не правило `init`. 8.3 (бюджет числа зависимостей на фичу) — мягкий порог и
требует нового пофичевого подсчёта импортов → вероятно норма (norm-or-choice, бар D035). Ревизия
контракта — в карточке. Это последняя карточка набора кластеров 1–8.
2026-10-30 P042 (B033) выполнен: 8.1 — объявленные, но неиспользуемые external-зависимости доставлены
в `init` расширением правила `directory-architecture` ([D076](Decisions/076.md)): резолвинг агрегирует
первый сегмент использованных external, а объявленная external вне used и seed (std/core/alloc,
`__future__`) — находка «declared external … is never imported» на владельце; новый вид/навык не добавлен
(I020 без изменений). 8.2 (локфайлы + `--locked`) и 8.3 (бюджет зависимостей) — норма
([D077](Decisions/077.md)): 8.2 уже обеспечено конфигом команд и тем, что `init` не поставляет команды
сборки; 8.3 — мягкий порог, требует пофичевого подсчёта, которого движок не делает. Наш код приведён к
зелёному (из `.agentrig/lint.yaml` убраны 7 реально неиспользуемых python-external). P042 перенесён в
[Archive/Plan](Archive/Plan.md), B033 — в [Archive/Backlog](Archive/Backlog.md); ссылки card в D076/D077
переведены на `Archive/Backlog/033.md`; пин stable.txt на голову ветки ([D051](Decisions/051.md)). Набор
кластеров 1–8 (B026–B033) завершён.
2026-10-31 после экспертного разбора пользователь согласовал **Набор II** — переносимые на любое
приложение, agent-native улучшения качества (10 кластеров C1–C10) — на автономное исполнение по моей
рекомендации, по одной карточке за ветку (завожу карточку → реализую → догфуд → merge+пин → следующая).
Порядок по ROI и зависимостям: **C1 зонирование → C2 границы окружения → C4 локальность/радиус →
C9 подавление линтов/`dbg!` → C6 эффекты/ошибки → C3 детерминизм → C5 контракты поведения →
C10 эволюция API/данных → C7 анти-дублирование → C8 анти-переусложнение**. Фундамент — C1: без класса
зоны правила бланкет-вредят или непереносимы. Честная оговорка: где свойство семантическое или требует
отвергнутого тип-фронтенда (D059), доставка — навык-норма + узкое синтаксическое правило (norm-or-choice,
бар [D035](Decisions/035.md)); притворных инвариантов нет. Первой выбран [B034](Backlog/034.md) —
промоутнут в [P043](Plan/043.md) на ветке feature/zoning. Остальные кластеры получают карточку в момент
исполнения (каждый проходит через Backlog перед Plan); ревизии контрактов — в карточках.
2026-10-31 по complexity-discipline C1 (зонирование) и C2 (границы окружения) слиты в одну фичу (решение
пользователя): механизм зоны без потребителя — конфигурационное измерение без поведения, поэтому поле
зоны доставляется вместе с первым правилом, которое его читает. Остаётся 9 фактических фич, содержание
сохранено. Доставляемый минимум по complexity-discipline — **один** класс зоны `pure` (модель
«assertion-check»: помеченный `pure` модуль проверяется, непомеченный не затрагивается — аддитивно, без
ослабления порогов D035), а не весь примерный набор классов; прочие классы приходят со своими
потребителями в следующих кластерах. Правило синтаксическое (префиксы `std::env`/`fs`/`net`/`process`/`time`,
near-zero-FP, D035/D059) — настоящее `[правило]`, не норма. Расширение схемы `architecture.yaml` новым полем
`zone` фиксируется решением [D078] (расширяет D043) в том же VAC. B034/P043 переформулированы под слитую фичу.
2026-10-31 P043 (B034, слитая C1+C2) выполнен: контракт `architecture.yaml` получил опциональное поле
`zone` с единственным классом `pure` ([D078](Decisions/078.md), расширяет [D044](Decisions/044.md)),
а правило `directory-architecture` сообщает о ссылках на `std::env`/`std::fs`/`std::process` в
собственных файлах каталога `zone: pure` (три сплошь-эффектогенных namespace, near-zero-FP; `std::net`/
`std::time`/`std::io` исключены как содержащие чистые типы-значения). Escape — не объявлять `pure` (opt-in,
аддитивно); нового вида правила/навыка нет → I020 без изменений. Догфуд: `structural_lint/architecture/
resolution` помечен `zone: pure`, наш код зелёный (коммит-гейт VAC-B `5102816` весь PASS). P043 перенесён в
[Archive/Plan](Archive/Plan.md), B034 — в [Archive/Backlog](Archive/Backlog.md); ссылка card в D078 переведена
на `Archive/Backlog/034.md`; пин stable.txt на голову ветки ([D051](Decisions/051.md)). Следующий кластер
Набора II — C4 (локальность/радиус).
2026-10-31 следующей выбран [B035](Backlog/035.md) — промоутнут в [P044](Plan/044.md) на ветке
feature/locality. По анализу: единственное новое статическое ядро C4 — входящая связность (fan-in)
каталога по уже существующему графу зависимостей, расширением правила `directory-architecture` (как
D070/D076/D078); бюджет — опциональное поле `dependents` в `architecture.yaml`, дефолт-константа,
легитимный хаб объявляет радиус явно. Прочие под-пункты C4 уже обеспечены (циклы — граф; файл/функция
— `file-size`/`function-lines`) либо норма (смена публичного контракта — тесты/тип-фронтенд D059;
action-at-a-distance — норма/узкое будущее правило). Нового вида правила/навыка нет → I020 без
изменений. Ревизия контракта — в карточке. Порядок остальных кластеров — за исполнителем.
2026-10-31 P044 (B035, C4) выполнен: контракт `architecture.yaml` получил опциональное поле
`dependents` — бюджет числа различных других каталогов, которым дозволено зависеть от этого
([D079](Decisions/079.md), расширяет [D078](Decisions/078.md)); правило `directory-architecture`
считает входящую связность по уже строящемуся графу (как для циклов) и сообщает о каталоге сверх
бюджета (дефолт 6) на уровне error с числом и бюджетом. Догфуд: шесть спроектированных общих хабов
объявили реальный радиус (`shared/paths`/`shared/vcs` — 12, `project_config` — 11, `shared/testing`/
`shared/diagnostics` — 10, `shared/yaml` — 9), наш код зелёный. Прочие под-пункты C4 уже обеспечены
(циклы — граф; размер файла/функции — `file-size`/`function-lines`) либо норма (смена публичного
контракта, action-at-a-distance — [D059](Decisions/059.md)). Нового вида правила/навыка нет → I020 без
изменений. P044 перенесён в [Archive/Plan](Archive/Plan.md), B035 — в [Archive/Backlog](Archive/Backlog.md);
ссылка card в D079 переведена на `Archive/Backlog/035.md`; пин stable.txt на голову ветки
([D051](Decisions/051.md)). Следующий кластер Набора II — C9 (подавление линтов/`dbg!`).
2026-10-31 следующей выбран [B036](Backlog/036.md) — промоутнут в [P045](Plan/045.md) на ветке
feature/hygiene. Кластер C9 Набора II «долговременная гигиена». По анализу: единственное новое
статическое ядро C9 — два Rust-артефакта глушения гейта, не ловимые существующими правилами:
широкий `#[allow(...)]`/`#![allow(...)]` (атрибут, не комментарий) без ссылки-обоснования — новым
правилом `allow-justification`, переиспользуя механизм justified ([D065](Decisions/065.md)/
[D074](Decisions/074.md)); и вызов `dbg!` — новым правилом `debug-artifact` (безусловно). Прочие
под-пункты C9 уже обеспечены: Python `# noqa`/`# type: ignore`, закомментированный код и
`TODO`/`FIXME` — бан комментариев ([D037](Decisions/037.md)); свежесть самоописания каталога —
[D070](Decisions/070.md); `println!`/`eprintln!` — сознательный выбор [D060](Decisions/060.md), не
возвращается; единый локальный стиль ошибок/логов/имён — норма (norm-or-choice, бар
[D035](Decisions/035.md)). Каждое новое правило добавляет свой навык ремонта → I020 (виды правил и
навыки растут согласованно). Притворных инвариантов нет. Порядок остальных кластеров — за
исполнителем; ревизии контрактов — в карточках.
2026-11-01 P045 (B036, C9) выполнен: доставлены два правила долговременной гигиены Rust —
`allow-justification` ([D080](Decisions/080.md)): широкий `#[allow(...)]`/`#![allow(...)]` без
ссылки-обоснования на строке прямо выше — находка error, с ссылкой — без находки (механизм
`justified`, как D065/D074), навык `justify-allow`; и `debug-artifact` ([D081](Decisions/081.md)):
вызов `dbg!` — находка error безусловно, навык `remove-debug`. Оба вида в `catalog::ALL`, оба навыка
в `assets.rs` → I020 (виды правил и навыки выросли на два, delivered == live). VAC-B1 `a464830`,
VAC-B2 `c851755`, VAC-C1 `843a29e`, VAC-C2 `dbcd342`. Наш код зелёный (единственный
`#[allow(deprecated)]` в `macos.rs` обоснован `// DECISION: D066`, `dbg!` в репозитории нет). P045
перенесён в [Archive/Plan](Archive/Plan.md), B036 — в [Archive/Backlog](Archive/Backlog.md); ссылки
card в D080/D081 переведены на `Archive/Backlog/036.md`; пин stable.txt на голову ветки
([D051](Decisions/051.md)). Набор II: C1+C2, C4 и C9 доставлены; следующий кластер — C6 (эффекты/ошибки).
2026-11-01 следующей выбран [B037](Backlog/037.md) — промоутнут в [P046](Plan/046.md) на ветке
feature/effects. Кластер C6 Набора II «эффекты и ошибки». По анализу: единственное новое переносимое
статическое ядро C6 — panic-путь в ПОСТАВЛЯЕМОМ линтере (`.unwrap()`/`.expect()` и макросы
`panic!`/`todo!`/`unimplemented!`/`unreachable!` без ссылки-обоснования прямо выше, тестовый код
исключён), новым правилом, переиспользующим механизм justified ([D065](Decisions/065.md)/
[D080](Decisions/080.md)); новый навык ремонта в группе `references`. На собственном dev-гейте это уже
закрыто профилем clippy ([D060](Decisions/060.md)), но поставляемый `init` этого не даёт — реформулировка
Набора II требует переносимости, поэтому ядро едет в `init` (D060 остаётся строгим dev-гейтом, не
отменяется). Прочие под-пункты C6 уже обеспечены: Python-обработка ошибок — `silent-error`
([D069](Decisions/069.md)); Rust `let _ = Result` — dev-гейт ([D068](Decisions/068.md)); явность
эффектов/чистота — зона `pure` ([D078](Decisions/078.md)); либо норма: «проглоченный `Result`» по
значению требует вывода типов (отвергнут, [D059](Decisions/059.md)). Правило добавляет свой навык → I020
(виды правил и навыки растут согласованно). Путь — тот же двухфазный (source→delivery) с мид-веткой
бутстрапом, что и C9. Притворных инвариантов нет. Порядок остальных кластеров — за исполнителем.
2026-11-01 P046 (B037, C6) выполнен: доставлено правило `panic-path` ([D082](Decisions/082.md)) —
вызов `.unwrap()`/`.expect(...)` и макросы `panic!`/`todo!`/`unimplemented!`/`unreachable!` без
ссылки-обоснования прямо выше — находка error, с ссылкой — без находки (механизм `justified`, как D065/
D080); детектор в `rust.rs` диспетчеризует `call_expression` и расширяет `macro_invocation` (рядом с
D081), обход предков `in_test_context` исключает тела `#[cfg(test)]`/`#[test]`, каталоги `tests/` и
файл-сосед `tests.rs` — паттернами exclude живого правила. Вид в `catalog::ALL`, навык `handle-panic` в
`assets.rs` → I020 (виды правил и навыки выросли на один, delivered == live). VAC-B1 `2a61311`, VAC-B2
`011e446`. Наш код зелёный (lib/bin без паники, inline `#[cfg(test)]` сняты обходом предков). P046
перенесён в [Archive/Plan](Archive/Plan.md), B037 — в [Archive/Backlog](Archive/Backlog.md); ссылка card
в D082 переведена на `Archive/Backlog/037.md`; пин stable.txt на голову ветки ([D051](Decisions/051.md)).
Набор II: C1+C2, C4, C9 и C6 доставлены; следующий кластер — C3 (детерминизм).
2026-11-01 следующей выбран [B038](Backlog/038.md) — промоутнут в [P047](Plan/047.md) на ветке
feature/determinism. Кластер C3 Набора II «детерминизм». По анализу (заземлён на реализации зоны
`pure`, D078): единственное новое переносимое статическое ядро C3 — чтение стенных часов
(`Instant::now`/`SystemTime::now`) в собственных файлах `zone: pure`-каталога, расширением проверки
зоны в правиле `directory-architecture` (как [D079](Decisions/079.md)/[D076](Decisions/076.md)):
ссылки уже резолвятся в полные пути, поэтому `::now` отличается от namespace `std::time` (чистые
типы-значения `Duration` не трогаются), near-zero-FP. Нового вида правила/навыка нет → I020 без
изменений; ремонт — существующий `refactor-large-directory`. Прочее C3 — норма: энтропия/RNG,
порядок итерации `HashMap`/`HashSet`, тайминг потоков (тип-вывод/поток данных отвергнут,
[D059](Decisions/059.md)); детерминизм тестов — уже testing-discipline 7.2 ([D075](Decisions/075.md)).
Притворных инвариантов нет. Порядок остальных кластеров — за исполнителем.
2026-11-01 P047 (B038, C3) выполнен: расширена проверка зоны `pure` правила `directory-architecture`
([D083](Decisions/083.md), расширяет [D078](Decisions/078.md)) — чтение стенных часов (`Instant::now`/
`SystemTime::now`) в собственном файле `zone: pure`-каталога становится находкой error, как и доступ к
окружению. `architecture/mod.rs` — `CLOCK` + предикат `clock()` (точное равенство полному external-пути);
`runner/analysis.rs` классифицирует ссылку через `environment()||clock()`; `policy.rs::check_zones`
называет категорию (свободная fn `zone_message()`: «must not read the wall clock» для часов, байт-
идентичная D078 строка для окружения). Резолвер уже дописывает `::now`, поэтому `Duration`/`UNIX_EPOCH`
остаются легальными (near-zero-FP, [D035](Decisions/035.md)). Нового вида правила/навыка нет → I020 без
изменений; ремонт — существующий `refactor-large-directory`. VAC-A `ef62152`, VAC-B `ad3235b`. Наш код
зелёный (единственный `pure`-каталог `structural_lint/architecture/resolution` часов не читает). P047
перенесён в [Archive/Plan](Archive/Plan.md), B038 — в [Archive/Backlog](Archive/Backlog.md); ссылка card
в D083 переведена на `Archive/Backlog/038.md`; пин stable.txt на голову ветки ([D051](Decisions/051.md)).
Набор II: C1+C2, C4, C9, C6 и C3 доставлены; следующий кластер — C5 (контракты поведения).
2026-11-01 следующей выбран [B039](Backlog/039.md) — промоутнут в [P048](Plan/048.md) на ветке
feature/contracts. Кластер C5 Набора II «контракты поведения». По анализу (заземлён на движке правил):
единственное новое переносимое near-zero-FP статическое ядро C5 — вызов семейства `debug_assert!`
(`debug_assert!`/`debug_assert_eq!`/`debug_assert_ne!`) в не-тестовом Rust без ссылки-обоснования прямо
выше, новым правилом `debug-assertion`, переиспользующим механизм justified
([D065](Decisions/065.md)/[D082](Decisions/082.md)); семейство вырезается под `cfg(debug_assertions)`,
поэтому закреплённый им контракт молча не выполняется в release. Новый навык ремонта `firm-up-assertion`
(поднять до `assert!`, убрать или обосновать). `assert!` не трогается — реальный контракт во всех
сборках. Прочее C5 — норма/покрыто: контракт прозой — бан комментариев ([D037](Decisions/037.md));
`#[must_use]`/проглоченный результат — вывод типов (dev-гейт [D068](Decisions/068.md)/отвергнут
[D059](Decisions/059.md)); механизм `INVARIANT` ([D020](Decisions/020.md)) — контрактная машинерия
репозитория, не переносимое правило `init`; Python `assert` под `-O` того же класса, но выше FP → норма.
Правило добавляет свой навык → I020 (виды правил и навыки растут согласованно). Путь — двухфазный
(source→delivery) с мид-веткой бутстрапом, как у C9/C6 (новый вид правила читается stable-бинарём из
`lint.yaml`). Притворных инвариантов нет. Порядок остальных кластеров — за исполнителем.
2026-11-01 P048 (B039, C5) выполнен: доставлено правило `debug-assertion` ([D084](Decisions/084.md)) —
вызов `debug_assert!`/`debug_assert_eq!`/`debug_assert_ne!` в не-тестовом Rust без ссылки-обоснования
прямо выше — находка error, с ссылкой — без находки (механизм `justified`, как D065/D082); детектор в
`rust.rs` (`assertion_macro`/`report_assertion`) расширяет `macro_invocation` (рядом с D081/D082), обход
предков `in_test_context` исключает `#[cfg(test)]`/`#[test]`, каталоги `tests/` и файл `tests.rs` —
паттернами exclude; `assert!`/`assert_eq!`/`assert_ne!` не трогаются (реальный контракт во всех сборках).
Вид в `catalog::ALL`, навык `firm-up-assertion` в `assets.rs` → I020 (виды правил и навыки выросли на
один, delivered == live). Путь двухфазный: VAC-B1 `5061b18` (source), мид-ветка пин stable=`5061b18`
(`787b27f`), VAC-B2 `1cb93b8` (delivery). Наш код зелёный (в репозитории нет вызовов `debug_assert!`,
`just candidate lint` → 0 находок). P048 перенесён в [Archive/Plan](Archive/Plan.md), B039 — в
[Archive/Backlog](Archive/Backlog.md); ссылка card в D084 переведена на `Archive/Backlog/039.md`; пин
stable.txt на голову ветки ([D051](Decisions/051.md)). Набор II: C1+C2, C4, C9, C6, C3 и C5 доставлены;
следующий кластер — C10 (эволюция API/данных).
2026-11-01 следующей выбран [B040](Backlog/040.md) — промоутнут в [P049](Plan/049.md) на ветке
feature/evolution. Кластер C10 Набора II «эволюция API/данных». По анализу (заземлён на движке правил и
уже существующем анализаторе зависимостей): единственное новое переносимое near-zero-FP статическое ядро
C10 — glob/wildcard-импорт, тянущий всю растущую поверхность чужого модуля: Rust `use …::*` (узел
`use_wildcard`) вне тестового кода и Python `from … import *` (узел `wildcard_import`) без ссылки-обоснования
прямо выше, новым правилом `wildcard-import`, переиспользующим механизм `justified`
([D065](Decisions/065.md)/[D082](Decisions/082.md)) и обход предков `in_test_context` (D082); новый навык
ремонта `narrow-import` в группе `references`. Прочее C10 — норма: исчерпываемость `match`/wildcard-рука
`_` высоколожна (скаляр-`match` и `#[non_exhaustive]`-enum чужого крейта законно требуют `_`, нужен
отвергнутый вывод типов [D059](Decisions/059.md)); `#[non_exhaustive]` нерелевантно бинарному крейту;
версионирование serde/схем конфиг-специфично; исчерпываемость enum уже стережёт компилятор. Правило
добавляет свой навык → I020 (виды правил и навыки растут согласованно). Путь — двухфазный
(source→delivery) с мид-веткой бутстрапом, как у C9/C6/C5 (новый вид правила читается stable-бинарём из
`lint.yaml`). Притворных инвариантов нет. Порядок остальных кластеров — за исполнителем.
