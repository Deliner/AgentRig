# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038, B030/P039, B031/P040 (в master). Текущая — B032/P041 (линты тестов): 7.1
тихо выключенные тесты → правило `disabled-test`; 7.2/7.3 — вероятно норма. Дальше — B033.

## Workspace

Ветка: feature/test-lints (от master `ca6fa4a`). Машина — macOS. Коммиты: `c93b22a` VAC-A
(промоут B032→P041). VAC-B1 (enabling) в рабочем дереве, готов к коммиту: kind.rs (вариант
`DisabledTest`), descriptor.rs (арм+const DISABLEDTEST, skill `justify-skip`), registry.rs
(DisabledTest в оба handler.rules), rust.rs (`attribute_item`→`disabled_test`), python.rs
(`decorator`→`disabled_test`), languages/tests/mod.rs (2 теста). Всё comment-free (без D074). НЕ в
catalog::ALL/lint.yaml/assets.rs. Сборка/коммит/бустрап/merge — через `just scratch -- '...'` с PATH
(`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым).

## Progress

Дизайн B032 завершён (подтверждён чтением движка). 7.1 = новый синтаксический Kind `disabled-test`
в обоих обработчиках (Rust `#[ignore]` через `attribute_item`→`attribute`→named_child(0); Python
`@pytest.mark.skip`/`skipif` через узел `decorator`, callee из call.function или самого выражения),
переиспользует `justified`. Двухкоммитная схема: B1 (готов) → пин+бустрап → B2 (catalog::ALL +
правило lint.yaml + навык `references/justify-skip` + flat-symlink + assets.rs +
references/architecture.yaml + D074 + маркеры). 7.2/7.3 → норма D075. Новых инвариантов нет. В нашем
коде выключенных тестов нет (догфуд зелёный заранее). Следующие D-номера: D074 (правило), D075 (нормы).

## Verification

Кандидат: `cargo test --lib languages::tests` — 13 passed (вкл. новые
`rust_flags_ignore_without_a_reference_on_the_line_above`,
`python_flags_skip_markers_without_a_justifying_reference`). Навигация по узлам подтверждена.

## Blockers

Нет.

## Next action

Закоммитить VAC-B1 (код + эта State) через хук. Затем пин+бустрап B1 (stable.txt=голова B1 +
`just bootstrap` в одном scratch, stable.txt НЕ коммитить) → VAC-B2 (live-правило + навык + D074) →
VAC-C (нормы D075) → архив P041/B032 → финальный пин (двухкоммитная схема P036) →
`just feature-merge` (не пушить, ветку сохранить) → отчёт на русском на границе merge; затем B033.
