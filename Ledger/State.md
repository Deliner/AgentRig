# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [B029](Backlog/029.md)
([P036](Plan/036.md)): синтаксические линты сложности. 4.1 (вложенность) и 4.2 (булевы
параметры) влиты в master. Сейчас — 4.3 бюджет ветвлений: `match`/`case` считается как ОДНО
многоходовое ветвление наряду с каждым `if`/`else-if`/тернарником ([D063](Decisions/063.md)).
Это последнее под-правило, оно закрывает P036.

## Workspace

Ветка: feature/branch-budget (от master `88c5e48`). Коммиты: A1 `239c9f5` (enabling,
старая arms-метрика), A1′ `817fd8e` (коррекция на «match = 1», забустрапен — установленный
бинарь теперь по option-B). Рабочее дерево: A2 (live) готов, не закоммичен.

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

B029 4.1 ([D061](Decisions/061.md)) и 4.2 ([D062](Decisions/062.md)) влиты в master.

B029 4.3 A2 (live) в рабочем дереве: правило `branch-budget` (`error: 5`, `# DECISION: D063`)
в lint.yaml; `Kind::BranchBudget` в `catalog::ALL`; маркеры `// DECISION: D063` на владельцах
(descriptor.rs, languages/mod.rs, rust.rs, python.rs, tests/mod.rs); запись D063
(Decisions/063.md + строка индекса + абзац); State. Порог `5` выбран по распределению option-B
(кластер 4–5, хвост 6–7). Догфуд 4 функций ≥6 приведён к ≤5 поведение-сохраняющими правками:
`outer_control` (классификатор роли символа), `arrange` (слияние решения о группе с его
эффектом), питон-тест (dispatch через `match failure`), `run_output` (mode+command одним
решением); все остаются в рамках D062 и D017. `just lint` чисто; `just lint-config-check`
принимает правило установленным бинарём.

## Verification

`just lint` (candidate, option-B, порог 5): 0 находок branch-budget и boolean-parameter.
`just lint-config-check`: конфиг валиден установленным A1′. Полный гейт на A2 ещё не прогонялся.

## Blockers

Нет.

## Next action

Застейджить A2 (lint.yaml, catalog/mod.rs, 5 файлов-маркеров, 4 догфуд-файла, Decisions.md,
Decisions/063.md, этот State; БЕЗ stable.txt) и закоммитить через гейт. Затем архив P036/B029:
перенести Plan/036.md→Archive/Plan, Backlog/029.md→Archive/Backlog, обновить card-ссылки
D061/D062/D063 на `Archive/Backlog/029.md`, строки Plan.md/Backlog.md/Archive-индексов; коммит
через гейт. Затем финальный пин: head ветки в stable.txt + `just bootstrap` одним scratch,
коммит пина; `just feature-merge`. Отчёт о границе 4.3/P036 по-русски. Не пушить, ветку
сохранить.
