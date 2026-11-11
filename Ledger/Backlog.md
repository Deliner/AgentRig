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
| [B116](Backlog/116.md) | Типы называют флаг и не выпускают замок | Смысл `true` виден на месте вызова, отсутствие ключа не толкуется по-разному, а вызывающий не держит замок сколько захочет. |
| [B117](Backlog/117.md) | Общее состояние без гонок | Инкременты не теряются, ложное пробуждение не проскакивает мимо условия, а соседние потоки и тесты не видят чужого окружения. |
| [B118](Backlog/118.md) | Вычисление там, где его видно | Аргумент вычисляется один раз и только когда нужен, выход из функции виден на месте вызова, а тип значения записан там, где значение появилось. |
| [B119](Backlog/119.md) | Тесты не проходят ложно | Тест не проходит от посторонней ошибки или паники и не зависит от скорости машины, а проверяется тот же код, что работает в продукте. |
| [B120](Backlog/120.md) | Граница с внешним миром | Данные не меняют команду, запрос или формат, а между проверкой и действием нет гонки. |