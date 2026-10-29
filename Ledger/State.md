# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [B029](Backlog/029.md)
([P036](Plan/036.md)): синтаксические линты сложности, по под-правилам. 4.1 глубина
вложенности — **сделана и влита в master**. Сейчас — 4.2 запрет булевых/флаговых
параметров. Дальше: 4.3 бюджет ветвлений.

## Workspace

Ветка: feature/boolean-parameters (от master `7c80a43`). Делаю 4.2 по двухкоммитной схеме.
A1 (enabling) подготовлен в рабочем дереве: `Kind::BooleanParameter`, дескриптор-политика
(Level), общий `boolean_parameters()` + предикаты `flag` в rust/python, навык
`split-boolean-parameter` (+ его `architecture.yaml` и строка в родительском), запись в
`assets.rs`, юнит-тесты. **Ещё НЕ** в `catalog::ALL` и **НЕ** в `lint.yaml` — I020
сбалансирован. Пин `stable.txt` = `e8553a8`.

Машина — macOS. Сборка/clippy/коммит/мерж — через `just scratch -- '...'` с PATH; `cargo`
в `~/.zshenv`. Новое правило-`Kind` требует двух коммитов с бустрапом между ними
(установленный бинарь валидирует `lint.yaml` и клинит на незнакомом `kind:`):
A1 enabling → пин+бустрап → A2 live (`ALL` + правило + догфуд) → финальный пин → merge.

## Progress

B029 4.1 (nesting-depth) **сделана и влита в master**: A1 `e4bb96a` → A2 `e8553a8` →
пин `341425a` → merge `7c80a43`. Решение [D061](Decisions/061.md).

B029 4.2 (boolean-parameter): A1-форма готова. Измерение — параметр, типизированный Rust-
примитивом `bool`, либо аннотированный `bool` / с дефолтом `True`|`False` в Python; каждый
такой параметр — находка (политика Level). `Option<bool>`, `&bool`, нелитеральные дефолты
не ловятся. Навык `split-boolean-parameter`. Структурные поля и `self` исключены.

Осталось по 4.2: коммит A1 → бустрап → A2 (`ALL` + правило в `lint.yaml` + догфуд наш код к
зелёному + решение D062 + финальный пин) → merge. Затем 4.3 (бюджет ветвлений).

## Verification

Два новых юнит-теста зелёные (`rust_flags_bool_parameters_not_fields_or_other_types`,
`python_flags_bool_annotations_and_true_false_defaults`); `cargo fmt` чистый. Полный гейт
прогонится на коммите A1.

## Blockers

Нет.

## Next action

Закоммитить A1 (enabling) через хук. Затем записать head A1 в `stable.txt` и забустрапить
одной командой scratch. Затем A2: добавить `Kind::BooleanParameter` в `catalog::ALL` и
правило `boolean-parameter` в `.agentrig/lint.yaml`, прогнать candidate lint, найти
догфуд-находки, починить (split/именованный тип, не ослабляя правило), записать D062,
финальный пин, `feature-merge`. Не пушить без просьбы.
