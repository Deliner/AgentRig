# Нормы «хорошего состояния»

Навигационный индекс ключевых норм, к которым пришла кодовая база. Это указатель, а не
второй источник истины: смысл и обязательность каждой нормы живут в её записи (решение,
инвариант или навык), ссылки ведут туда. При изменении нормы правится её запись, а не
этот индекс.

| Норма | Где закреплено |
| --- | --- |
| Стандарт рига: код на 20 лет, AI-first и agent-native; проверка нужна, если повышает качество, даже без цели в риге; несоответствие ригу чинится | [D151](Decisions/151.md), [AGENTS](../AGENTS.md) |
| Докфудинг: universal-возможности совпадают в поставке `init` и в репозитории | [D053](Decisions/053.md), [I020](Invariants/020.md) |
| Доставка от результата; согласие в чате, затем автономное исполнение выборки | [D009](Decisions/009.md), [D032](Decisions/032.md), [D031](Decisions/031.md) |
| Ветки-фичи, VAC через коммит-гейт, пин+bootstrap перед merge меняющих Project | [D010](Decisions/010.md), [D011](Decisions/011.md), [D051](Decisions/051.md) |
| Затронутые тесты: на коммите и всей веткой на merge | [D035](Decisions/035.md) |
| Тестовая дисциплина: поведение через публичную поверхность на правильном уровне | [D054](Decisions/054.md), [D055](Decisions/055.md), [навык](../.agents/skills/testing-discipline/SKILL.md) |
| Продукт в `Project`, память в `Ledger`; фичи первого уровня — из двух слов | [D008](Decisions/008.md), [D041](Decisions/041.md), [D042](Decisions/042.md) |
| Контракты архитектуры только в `Project`; тип у фич нижнего уровня; без циклов | [D052](Decisions/052.md), [D043](Decisions/043.md), [D044](Decisions/044.md), [навык](../.agents/skills/refactor-large-directory/SKILL.md) |
| Вертикальные срезы (принцип; отдельного решения нет — закреплён запросом и правилом архитектуры) | [Requests](Requests.md) ч.1 п.20, [D028](Decisions/028.md) |
| Комментарии только как ссылки на решения/инварианты | [D037](Decisions/037.md), [навык](../.agents/skills/replace-comment/SKILL.md) |
| Markdown-дисциплина: размер/ширина README и AGENTS, прочий MD вне `Project` | [D036](Decisions/036.md), [навык](../.agents/skills/condense-markdown/SKILL.md) |
| Модульный YAML-конфиг в `.agentrig`; рецепты `just` по группам; строгий YAML | [D038](Decisions/038.md), [D039](Decisions/039.md), [D026](Decisions/026.md) |
| Минимализм «lint-first»: без upgrade/review/delegation и внешних VCS | [D033](Decisions/033.md) |
| Решения и инварианты раздельно; решения неизменны, ссылки мигрируют; norm-or-choice | [D001](Decisions/001.md), [D020](Decisions/020.md), [навык](../.agents/skills/norm-or-choice/SKILL.md) |
| Дисциплина сложности: минимально необходимая сложность на каждом шаге | [D006](Decisions/006.md), [навык](../.agents/skills/complexity-discipline/SKILL.md) |
| Снимок восстановления State, сверяемый с Git | [D013](Decisions/013.md), [навык](../.agents/skills/edit-state/SKILL.md) |
| N1, типовая дисциплина: тип закрывает названный инвариант, режим или жизненный цикл; разбор один раз на границе, дальше `enum` | [D110](Decisions/110.md), [навык](../.agents/skills/type-driven-discipline/SKILL.md) |
| N2, fail-closed вердикты: `Allow` только из позитивной проверки, ошибка или неизвестность — `Deny` с причиной, негативный тест на сайт | [D110](Decisions/110.md), [I021](Invariants/021.md), [навык](../.agents/skills/fail-closed-enforcement/SKILL.md) |
