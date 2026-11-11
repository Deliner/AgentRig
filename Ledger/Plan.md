# Plan

**Plan contains only work explicitly selected for execution now in the current session or its resumed task. Future ideas, improvements and hypotheses belong in [Backlog](Backlog.md), without delivery priority.**

Use [edit-plan](../.agents/skills/edit-plan/SKILL.md) and [execute-plan-feature](../.agents/skills/execute-plan-feature/SKILL.md). A request to save an idea for later does not select it for execution. An interruption or context reset does not discard the current authorized task.

Each new product row references an agreed Backlog outcome and acceptance, leaving implementation and VAC boundaries to the executing agent. Follow edit-backlog before promotion: resolve behavior-changing questions and agree execution order. Preserve the source text and criterion IDs rather than creating a second specification. Keep its agreed Git revision with the source reference for delivery and review. A selected set is executed autonomously without asking again before each card. Row order sequences only the selected current work. Stable PNNN IDs are never reused. Dependencies may refer to current Plan or completed Archive rows; they must exist and be acyclic. Active/complete entries require completed prerequisites. Archived dependencies do not restore old rows to current Plan.

Statuses remain pending, active, paused and complete within current delivery; at most one entry is active. Details contain Feature, User capability and Acceptance, with Delivery required for paused/complete entries. A temporarily blocked current task retains its blocker, branch and resumption condition. If execution is deferred, preserve that context in Backlog.

Automatically rotate completed rows into [Archive/Plan.md](Archive/Plan.md) and their cards into Archive/Plan/ during completion, preserving IDs, acceptance and delivery evidence; their source Backlog rows and cards move to [Archive/Backlog.md](Archive/Backlog.md) and Archive/Backlog/ at the same time. The agent performs this through edit-plan without another confirmation, updating references. Plan is not a completed-work archive; an empty Plan is valid.

P001-P009 are retained in Archive; P010-P016 moved to B001-B007 under the user's current-execution rule. Those P IDs remain retired. Original records are available at Git revision f6cd86efbae8ec501759f0c250b9bd4b76e300d7.

| ID | Status | Depends on | Feature | User capability |
| --- | --- | --- | --- | --- |
| [P122](Plan/122.md) | active | - | [B117](Backlog/117.md): shared-state — общее состояние без гонок | Инкременты не теряются, ложное пробуждение не проскакивает мимо условия, а соседние потоки и тесты не видят чужого окружения. |
| [P123](Plan/123.md) | pending | - | [B118](Backlog/118.md): visible-evaluation — вычисление там, где его видно | Аргумент вычисляется один раз и только когда нужен, выход из функции виден на месте вызова, а тип значения записан там, где значение появилось. |
| [P124](Plan/124.md) | pending | - | [B119](Backlog/119.md): false-pass — тесты не проходят ложно | Тест не проходит от посторонней ошибки или паники и не зависит от скорости машины, а проверяется тот же код, что работает в продукте. |
| [P125](Plan/125.md) | pending | - | [B120](Backlog/120.md): outer-boundary — граница с внешним миром | Данные не меняют команду, запрос или формат, а между проверкой и действием нет гонки. |

История выбора и поставки P017–P070 хранится в Delivery архивных карточек [Archive/Plan](Archive/Plan.md) и в тексте этого файла на Git-ревизии c1935e47a3fe0c1b556692aed4f9c81d1a979fbb.

2026-11-07 пользователь поручил автономно, без вопросов между карточками, выполнить все открытые карточки Backlog
(B047, B048, B050, B051, B064, B068–B074): выбрано **P071 → P082**, порядок за исполнителем — сначала малые
ремонты и расширения действующих правил, затем новые виды правил, затем форк грамматики и ratchet. P076 зависит от
P075: async-контекст обеих карточек определяется одним механизмом. Открытые вопросы карточек о пороге, условии
срабатывания и формате исполнитель решает замером или проектированием при доставке и записывает в Delivery и
решениях; приёмка карточек не меняется. По одной карточке за ветку; новый `kind:` — двухфазно с мид-бранч пином
([D051](Decisions/051.md)).

2026-11-07 пользователь выбрал все новые карточки B075–B078 («бери все и делай»): выбрано **P083 → P086**, порядок
за исполнителем. Первой идёт B077: пропуск в карте влияния молча отключает тесты гейта и для следующих веток. Затем
B078 и B076, обе опираются на политику `disallowed`, последней — B075, ремонты для линтов clippy. Карточки в Git на
ревизии `e90d81a`. Открытые вопросы карточек исполнитель решает при доставке и записывает в Delivery и решениях. По
одной карточке за ветку; изменение действующего правила — с пином D051.

2026-11-07 пользователь сохранил B079–B082 и поручил выполнить их автономно: выбрано **P087 → P090**, порядок за
исполнителем. Первой идёт B081: B082 поставляет новым проектам весь профиль dev-гейта, включая линты B081. Затем
новые виды правил B079 и B080, двухфазно с мид-бранч пином ([D051](Decisions/051.md)), последней — B082. Карточки в Git
на ревизии `79be435`. По одной карточке за ветку.

2026-11-07 пользователь поручил довести voxel-rust до готовности в AgentRig, одобрил карточку B083 и выбрал её
выполнение («Сохранить и выполнить»): выбрано **P091**. Карточка в Git на ветке `feature/rust-expansion`, ревизия
первого VAC ветки. Новый разбор не вводит вид правила, пин D051 нужен в конце ветки.

2026-11-07 пользователь поручил подготовить карточку о запрете `Option<Option<_>>` и реализовать её до работ в
voxel-rust («Подготовь черновик и реализуй его перед voxel»): выбрано **P092** (B084). P091 (B083) остаётся на ветке
`feature/rust-expansion` и ждёт проверки AC5 на voxel-rust; P092 идёт отдельной веткой от master. Карточка в Git на
ревизии `3033a2b`.

2026-11-09 пользователь поручил «реализуй карточки все и продолжи с voxel»: показанная в чате карточка B085 сохранена
и выбрана как **P093**, до продолжения работ в voxel-rust. Карточка в Git на ветке `feature/coverage-include`, ревизия
первого VAC ветки. Новое правило не вводит вид проверки, пин D051 нужен в конце ветки.

2026-11-10 пользователь поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты написал,
и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не спрашиваю, я
на все даю подтверждение». Выбрано **P094 → P109** (B086–B101), порядок за исполнителем: сначала ремонты гейта, очистки
и базовой линии, затем правила структуры и сообщения, затем граф зависимостей Rust, тестовый код и паники, последней —
B086 о рантайме на нескольких платформах. P097 зависит от P096: обе меняют отсчёт понижения базовой линии. P105 и P108
зависят от P102: их замеры читают зависимых и рёбра, которые P102 выводит. Карточки в Git: B086–B090, B092, B093 на
ревизии `54aa528`, остальные на `4518103`. Открытые вопросы и выбор, который карточки оставляли пользователю,
исполнитель решает сам и записывает в Delivery и решениях. По одной карточке за ветку; ветка, меняющая Project,
кончается пином D051.

2026-11-10 пользователь подтвердил показанные карточки B102–B105 и поручил: «Все карточки подтверждаю, заводи и делай
автономно». Выбрано **P110 → P113**, порядок за исполнителем: сначала запреты Clippy без кода в риге, затем пересмотр
D107, затем clippy.toml от `init`, последними — свои структурные правила, самая большая работа. Карточки в Git на
ревизии `d45f3a2`. По одной карточке за ветку; ветка, меняющая Project,
кончается пином D051, новый вид правила — двухфазно с мид-бранч пином.

2026-11-10 пользователь одобрил показанную карточку B106 и поручил: «На карточку ок, сохрани и тоже последовательно
запланируй в работу». Выбрано **P114**, после P113 в той же автономной поставке. Карточка в Git на ветке
`feature/structural-bans`, ревизия `dc86001`. P114 идёт своей веткой от master после слияния P113; новый вид
правила — двухфазно с мид-бранч пином, ветка кончается пином D051.

2026-11-11 пользователь поручил после второй партии правил переписать тесты рига с Python на Rust и дал согласие на
всё, что для этого нужно: «изучи, работай автономно, я тебе сразу даю ок на все, что нужно для выполнения этого
задания». Карточка B108 сохранена по этому согласию и выбрана как **P115**. Карточка в Git в первом VAC ветки
`feature/rust-tests`; ветка, меняющая Project, кончается пином D051.

2026-11-11 пользователь одобрил показанные карточки B110–B112 и выбрал две из них: «И вот 110, 111 можешь брать
сразу». Выбрано **P116 → P117** после P115. P116 зависит от P115: её замер сравнивает гейт с переносом тестов, который
делает P115. B112 ждёт доступа к Linux-машине и не выбрана. Карточки в Git в коммите их сохранения на ветке
`feature/rust-tests`. По одной карточке за ветку; ветка, меняющая Project, кончается пином D051.

2026-11-11 после P117 пользователь выбрал вариант Б для мест, где Clippy не видит вложенный Option, одобрил
показанную карточку B113 и поручил: «Да, сохрани и бери в работу». Выбрано **P118**. Карточка в Git в первом VAC ветки
`feature/nested-option-places`; ветка, меняющая Project, кончается пином D051.

2026-11-11 после P118 пользователь выбрал вариант Б для методов и `const` в реализации трейта, одобрил показанную
карточку B114 и поручил: «Да, сохрани и бери в работу». Выбрано **P119**. Карточка в Git в первом VAC ветки
`feature/trait-impl-option`; ветка, меняющая Project, кончается пином D051.

2026-11-11 пользователь принял 20 форм списка IV, одобрил показанные карточки B115–B120 и поручил: «Да, сохрани и
бери в работу». Выбрано **P120 → P125** в порядке карточек. Карточки в Git в первом VAC ветки `feature/error-loss`. По
одной карточке за ветку; новый вид правила — двухфазно с мид-бранч пином, ветка, меняющая Project, кончается пином
D051.
