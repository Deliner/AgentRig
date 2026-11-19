# Backlog

**Backlog collects future ideas, improvements, observations and hypotheses. Row order and IDs do not express execution priority, deadlines or implementation authorization.**

Use [edit-backlog](../.agents/skills/edit-backlog/SKILL.md) to draft ideas and requirement changes in chat. Save the shown version only after explicit agreement. Preserve context, distinguish observations from hypotheses and check for duplicates, then resume the current task. Do not investigate merely to fill a card or interrupt autonomous execution to seek approval for an unrelated idea.

Rows link stable BNNN IDs to Backlog/NNN.md. Cards retain Context, Proposal and Expected benefit; edit-backlog defines the user-facing content, scenario-based acceptance and agreement process. Ideas may stay brief. Every new product outcome passes through Backlog before the current [Plan](Plan.md), after behavior and acceptance are agreed, consequential questions resolved, and execution and order selected. Plan references the same contract and its source version. After completion, the row and card move to [Archive/Backlog.md](Archive/Backlog.md) with the Plan card. Work required by current acceptance stays in the current task.

Earlier requirements and related ideas are context for later selection, not a delivery queue. Reassess actual prerequisites and refine selected cards in chat; historical saved requirements are not automatic agreement for autonomous delivery.

| ID | Idea | Expected benefit |
| --- | --- | --- |
| [B107](Backlog/107.md) | Имена узлов сверяются с грамматикой; требует дополнительного согласования, реализовывать пока нельзя | Опечатка в имени узла не выключает часть правила молча, а останавливает коммит с точным местом. |
| [B109](Backlog/109.md) | Оракул cargo привязан к своему файлу | Тест с тем же именем в другом файле не может обмануть проверку инвариантов. |
| [B112](Backlog/112.md) | Риг на Linux работает так же, как на Mac; ждёт доступа к машине | Риг можно ставить в проекты на Linux с тем же поведением, что на Mac. |
| [B126](Backlog/126.md) | Блокирующие правила lint на числах понятности структуры | Структура проекта не становится непонятнее незаметно. |
| [B137](Backlog/137.md) | Clippy запускается как часть lint рига | Гейт Rust-проекта всегда проверяет полный профиль Clippy рига, и молча ослабить его нельзя. |
| [B138](Backlog/138.md) | Свой навык для каждого lint-а профиля Clippy; выполняется после B137 | Агент с ошибкой Clippy получает навык рига: как исправить этот lint по стандарту рига и не ослабить запрет. |
| [B139](Backlog/139.md) | Сравнение работы агента с ригом и без него | Замер показывает, улучшает ли риг итог работы агента, где именно и какие правила окупаются. |
| [B140](Backlog/140.md) | Почему `lint-structure` ставит эталон ниже деревьев агентов | Итог `lint-structure` снова показывает, какая структура понятнее. |
| [B141](Backlog/141.md) | Значения в коде только через именованные константы; есть открытые вопросы | Каждое значение в коде имеет имя, которое говорит, что оно значит. |
| [B148](Backlog/148.md) | AGENTS.md и CLAUDE.md с одним текстом и проверкой при коммите | Codex и Claude Code получают одни и те же правила, и разъехаться они не могут. |
| [B150](Backlog/150.md) | Убрать `just scratch`: хватает `just read`, `just write` и каталога | Агент запускает только то, что разрешают списки `just read`, `just write` и каталог. |
| [B151](Backlog/151.md) | Разрешённые языки программирования в проекте | В проекте под ригом код пишется только на выбранных языках и проходит языковые правила lint. |
| [B152](Backlog/152.md) | Хук проверяет каждый вызов инструмента и отказывает, если проверить не может | Нет инструмента или сбоя, при котором действие агента проходит мимо проверок рига. |
| [B153](Backlog/153.md) | Хук понимает команду так же, как оболочка агента и just | Хук разрешает ровно ту команду, которая потом выполнится. |
| [B154](Backlog/154.md) | Аргументы разрешённых программ не запускают другой код | Разрешённая программа делает только своё дело и не запускает чужой код. |
| [B155](Backlog/155.md) | Программы `just read` и `just write` не выходят за пределы проекта | Команда агента ничего не меняет вне проекта и не запускает процессы мимо песочницы. |
| [B156](Backlog/156.md) | Коммит всегда проходит гейт, а базу меняет только `feature-merge` | Правила AGENTS.md о хуке и базе выполняются командами, а не только текстом. |
| [B157](Backlog/157.md) | Агент не выключает и не ослабляет механизмы контроля; есть открытые вопросы | Проверки рига нельзя выключить или ослабить изнутри сессии агента. |
| [B158](Backlog/158.md) | Гейт проверяет правку по правилам, которые она не может ослабить; есть открытые вопросы | Проверку нельзя ослабить той же правкой, которую она проверяет. |
| [B159](Backlog/159.md) | Тексты для агентов не противоречат друг другу и командам рига | Агент, следуя любому тексту рига, выполняет допустимую команду. |
| [B161](Backlog/161.md) | Индекс в литерал массива вычисляет все элементы | Код не вычисляет значения, которые сразу выбрасывает. |
| [B162](Backlog/162.md) | Одна классификация enum написана в нескольких местах; есть открытый вопрос | Одна проверка над enum живёт в одном месте. |
| [B163](Backlog/163.md) | Ошибка не стирается через `.ok().flatten()` и запасное значение у `join()` | Ошибка разбора или паника потока не превращается молча в значение по умолчанию. |
| [B164](Backlog/164.md) | Флаг проверяется до вызова, а не в `.filter` по внешнему флагу; есть открытый вопрос | Работа, результат которой выбросит флаг, не выполняется. |
| [B165](Backlog/165.md) | Запасной `MAX` перед сложением или умножением | Запасное значение-граница не переполняется и не теряет смысл в следующей арифметике. |
| [B166](Backlog/166.md) | Вычитание счётчиков битов проверяется, как вычитание длины | Вычитание счётчиков не паникует на данных, которые должны дать обычную ошибку. |
| [B168](Backlog/168.md) | Индекс из одного массива не читает другой массив | Связанные значения меняются в одном месте и не расходятся по позициям. |