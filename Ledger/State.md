# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038, B030/P039 (в master). Текущая карточка — [B031](Backlog/031.md) (линты
именования), промоутнута в [P040](Plan/040.md). План: 6.1 имена-помойки — новое правило
`dump-name` (каталог `utils`/`helpers`/`misc`/`common`; `shared` легитимен), ремонт —
существующий `refactor-large-directory`; 6.2/6.3 — норма (анализ области видимости / шум, D035).

## Workspace

Ветка: feature/naming-lints (от master `e71b1cc`), HEAD `e71b1cc` (ещё без коммитов ветки).
Машина — macOS. Рабочее дерево: VAC-A (промоушен) не закоммичен — Plan/040.md создан, строка
P040 (active) и проза-промоушен в Plan.md добавлены, этот State-рефреш. stable.txt = `9f13d52`
(установлен бустрапом на прошлой ветке; будет перезаписан финальным пином этой ветки).
Сборка/clippy/fmt/тесты/коммит/бустрап — через `just scratch -- '...'` с PATH (`cargo` в
`~/.cargo/bin`, venv `tooling/.venv/bin` первым для Python-гейта); `just run`/гейт не видят cargo
в PATH без этого.

## Progress

VAC-A (промоушен B031→P040) готов к коммиту. Дизайн после изучения движка: 6.1 — новый `Kind`
`DumpName` (target directory, Level), структурная мера в `structural_lint/mod.rs`
(`structural_measurements`) по basename каталога против фикс-набора `{utils, helpers, misc,
common}`; `shared` вне набора. Доставка в init требует `Kind` (поставляемый конфиг — шаблон по
`catalog::ALL`), поэтому двухкоммитная схема lint-kind: B1 (вариант+дескриптор+мера+юнит-тест, вне
`catalog::ALL`/lint.yaml, без комментариев) → пин+бустрап → B2 (в `catalog::ALL` + правило
`.agentrig/lint.yaml` + маркеры D072 + догфуд). Навык переиспользуется (`refactor-large-directory`),
I020 по skills не меняется, растёт только rule-kinds. Подтверждено: config::load десериализует
`kind` в enum без проверки членства в `catalog::ALL`, значит B1-бинарь после пина грузит и считает
правило `dump-name` из B2. В Project нет каталогов-свалок (догфуд зелёный заранее). 6.2/6.3 — норма
решением (запись в VAC-C).

## Verification

Пока нет (VAC-A — только память). Изучены: catalog/descriptor/kind, languages/registry+mod
(мера), rules.rs (диспетчер structural_measurements), config/mod.rs (десериализация kind без
catalog-гейта), selection/inventory (directory-target перебирает ключи inventory.directories),
test_delivery.py + I020 (set-based по rule-kinds/skills; worker = кандидат).

## Blockers

Нет.

## Next action

1) Закоммитить VAC-A (Plan/040.md + Plan.md + State) через хук. 2) VAC-B1 (enabling): `DumpName`
в `kind.rs` + дескриптор + арм `structural_measurements` + юнит-тест в `tests/mod.rs`;
`cargo build`+тест (кандидат) через scratch; коммит; пин+бустрап B1 (stable.txt не коммитить).
3) VAC-B2 (live): `catalog::ALL` += DumpName, правило в `.agentrig/lint.yaml`, маркеры D072,
карточка D072, догфуд candidate `just lint`; коммит. 4) VAC-C: D073 (6.2/6.3 — норма). 5) Архив
P040/B031 + финальный пин (двухкоммитная схема P036) + `just feature-merge`; не пушить, ветку
сохранить. 6) Отчёт пользователю на русском на границе merge; затем B032–B033.
