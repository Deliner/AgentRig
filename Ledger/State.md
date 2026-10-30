# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038, B030/P039, B031/P040 (в master). Текущая — B032/P041 (линты тестов): 7.1
тихо выключенные тесты → правило `disabled-test`; 7.2/7.3 — вероятно норма. Дальше — B033.

## Workspace

Ветка: feature/test-lints (от master `ca6fa4a`). Машина — macOS. VAC-A (промоут B032→P041):
создан Ledger/Plan/041.md; строка P041 (active) и проза-промоут добавлены в Plan.md; эта State.
Ещё НЕ закоммичено. Сборка/коммит/бустрап/merge — через `just scratch -- '...'` с PATH
(`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым).

## Progress

Дизайн B032 завершён (подтверждён чтением движка). 7.1 = новый синтаксический Kind `disabled-test`
в обоих обработчиках (Rust `#[ignore]` через `attribute_item`→`attribute`→named_child(0); Python
`@pytest.mark.skip`/`skipif` через узел `decorator`), переиспользует `justified` (ссылка на строке
прямо над узлом). Двухкоммитная схема: B1 (вариант+дескриптор+меры+регистрация в handler.rules+
юнит-тесты, comment-free, НЕ в catalog::ALL/lint.yaml) → пин+бустрап → B2 (catalog::ALL + правило
lint.yaml + навык `references/<name>` + flat-symlink + assets.rs + references/architecture.yaml +
D074 + маркеры). 7.2/7.3 → норма D075. Новых инвариантов нет. В нашем коде выключенных тестов нет
(догфуд зелёный заранее). Следующие D-номера: D074 (правило), D075 (нормы).

## Verification

Пока не запускалось (VAC-A — только память). Нет коммитов на ветке.

## Blockers

Нет.

## Next action

Закоммитить VAC-A (промоут) через хук: `git add Ledger/Plan.md Ledger/Plan/041.md Ledger/State.md`.
Затем VAC-B1 (enabling, comment-free) → пин+бустрап B1 (stable.txt+bootstrap в одном scratch,
НЕ коммитить) → VAC-B2 (live-правило + навык + D074) → VAC-C (нормы D075) → архив P041/B032 →
финальный пин (двухкоммитная схема P036) → `just feature-merge` (не пушить, ветку сохранить) →
отчёт на русском на границе merge; затем B033.
