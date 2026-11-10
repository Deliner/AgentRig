# Backlog

**Backlog collects future ideas, improvements, observations and hypotheses. Row order and IDs do not express execution priority, deadlines or implementation authorization.**

Use [edit-backlog](../.agents/skills/edit-backlog/SKILL.md) to draft ideas and requirement changes in chat. Save the shown version only after explicit agreement. Preserve context, distinguish observations from hypotheses and check for duplicates, then resume the current task. Do not investigate merely to fill a card or interrupt autonomous execution to seek approval for an unrelated idea.

Rows link stable BNNN IDs to Backlog/NNN.md. Cards retain Context, Proposal and Expected benefit; edit-backlog defines the user-facing content, scenario-based acceptance and agreement process. Ideas may stay brief. Every new product outcome passes through Backlog before the current [Plan](Plan.md), after behavior and acceptance are agreed, consequential questions resolved, and execution and order selected. Plan references the same contract and its source version. After completion, the row and card move to [Archive/Backlog.md](Archive/Backlog.md) with the Plan card. Work required by current acceptance stays in the current task.

Earlier requirements and related ideas are context for later selection, not a delivery queue. Reassess actual prerequisites and refine selected cards in chat; historical saved requirements are not automatic agreement for autonomous delivery.

| ID | Idea | Expected benefit |
| --- | --- | --- |
| [B086](Backlog/086.md) | Рантайм проекта на нескольких платформах | Проект с AgentRig коммитит и сливает ветки на Linux и macOS с одной ревизии, без обхода хуков. |
| [B088](Backlog/088.md) | Находки архитектуры показывают свои рёбра | Находку архитектуры можно исправить по её тексту, без собственного оракула рёбер. |
| [B089](Backlog/089.md) | Что тратит бюджет входящих зависимостей | Бюджет входящих зависимостей считает только рабочий код, и тесты другого каталога не вынуждают его поднимать. |
| [B090](Backlog/090.md) | `config-check` сверяет цели тестов так же, как cargo | `config-check` принимает те цели тестов, по которым cargo выполняет тесты файла, и не требует подбирать префикс под устройство крейта. |
| [B091](Backlog/091.md) | Тестовые файлы Rust по соглашению AgentRig | Неправильно размещённый тестовый файл сразу получает находку с причиной и способом исправления. |
| [B096](Backlog/096.md) | Рёбра зависимостей через вызовы методов | Бюджеты и циклы считаются по реальным зависимостям. |
| [B097](Backlog/097.md) | panic-path в телах `macro_rules!` | Паника в макросе требует того же обоснования, что и в обычном коде. |
| [B099](Backlog/099.md) | Одна ссылка на решение для паники под Clippy | Одно решение отмечается в коде один раз. |
