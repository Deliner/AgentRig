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
| [B206](Backlog/206.md) | Новые решения в трёх разделах: что решили, почему, что отклонили; индексы вне лимита размера | Решение содержит только своё и не пересказывает карточку; новое решение можно записать, не подгоняя индекс под лимит. |
| [B207](Backlog/207.md) | Навыки проекта: стандарты проекта поверх стандартов рига; требует дообсуждения, в работу брать нельзя | Правило проекта, которое не проверить программой, агент получает тогда, когда правит код, к которому оно относится. |
| [B208](Backlog/208.md) | North Star и словарь проекта приходят агенту в начале каждой сессии | Каждая сессия с первого сообщения знает, зачем существует проект, и понимает его слова так же, как пользователь. |
| [B209](Backlog/209.md) | Одна карточка на задачу: три каталога, ключ очереди и номера | У задачи один номер и одна карточка от идеи до архива; агент читает только карточки каталога, с которым работает. |
| [B210](Backlog/210.md) | Карточка в новых разделах и проверка готовности перед Plan/; выполняется после B209 | Карточка говорит только то, что действует сейчас, и одинаково читается пользователем и агентом; недоработанная карточка не попадает в работу. |
| [B211](Backlog/211.md) | Режим карточки: агент спрашивает или решает сам; остановка и приёмка; выполняется после B210 | Пользователь заранее выбирает, где агент решает сам, а где спрашивает; решения о поведении продукта записаны, спорная карточка не попадает в базовую ветку. |
| [B212](Backlog/212.md) | Критерий карточки при сдаче становится инвариантом реестра; выполняется после B211 | Поведение, о котором договорились в карточке, остаётся проверенным после её сдачи: это инвариант, который нельзя тихо убрать или переписать. |