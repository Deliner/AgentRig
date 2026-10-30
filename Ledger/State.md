# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. [P036](Archive/Plan/036.md)
([B029](Archive/Backlog/029.md)) завершён: три под-правила сложности доставлены — 4.1
вложенность ([D061](Decisions/061.md)), 4.2 булевы параметры ([D062](Decisions/062.md)),
4.3 бюджет ветвлений ([D063](Decisions/063.md)); наш код зелёный. Следующие карточки набора
— B026, B028, B030–B033 (порядок за исполнителем).

## Workspace

Ветка: feature/branch-budget (от master `88c5e48`). Коммиты: A1 `239c9f5` (enabling, старая
arms-метрика), A1′ `817fd8e` (коррекция «match = 1», забустрапен), A2 `d522f39`
(branch-budget live, D063). Рабочее дерево: архив P036/B029 выполнен, не закоммичен;
`tooling/distribution/stable.txt` = `817fd8e` (промежуточный пин A1′, не закоммичен, в коммит
архива НЕ входит).

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

B029 4.1 ([D061](Decisions/061.md)) и 4.2 ([D062](Decisions/062.md)) влиты в master.

B029 4.3 доставлено (A2 `d522f39`): правило `branch-budget` (`error: 5`, `# DECISION: D063`)
в lint.yaml; `Kind::BranchBudget`; маркеры на владельцах; запись D063. Порог `5` по колену
распределения «match = 1». Четыре функции ≥6 приведены к ≤5 поведение-сохраняющими правками
(`outer_control`, `arrange`, питон-тест через `match`, `run_output`), в рамках D062 и D017.

Архив P036/B029 выполнен в рабочем дереве: карточки → `Archive/Plan/036.md` (с секцией
Delivery) и `Archive/Backlog/029.md` (ссылки `../../`); строка P036 убрана из Plan.md (проза
завершения), строка B029 убрана из Backlog.md; строки добавлены в Archive/Plan.md и
Archive/Backlog.md; card-ссылки D061/D062/D063 переведены на `Archive/Backlog/029.md`.
Осталось закоммитить архив через гейт.

## Verification

Гейт A2 `d522f39` полностью зелёный: build, lint (установленный A1′, branch-budget error 5,
0 находок), memory, rustfmt, clippy, rust-test 83, ruff-format, ruff, mypy, pytest 356, typos,
vulture. Коммит архива через гейт ещё не прогонялся.

## Blockers

Нет.

## Next action

Застейджить только Ledger-изменения архива (`git add Ledger`; БЕЗ `stable.txt`) и закоммитить
через гейт. Затем финальный пин (D051): голова ветки в `tooling/distribution/stable.txt` +
`just bootstrap` одним `just scratch`, коммит пина. Затем `just feature-merge` (прогоняет
затронутые ветвью тесты, ветку сохраняет). Отчёт о границе 4.3/P036 по-русски. Не пушить,
ветку сохранить. Затем следующая карточка набора (B026, B028, B030–B033).
