# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038, B030/P039, B031/P040 (в master). Текущая — B032/P041 (линты тестов): 7.1
тихо выключенные тесты → правило `disabled-test`; 7.2/7.3 — вероятно норма. Дальше — B033.

## Workspace

Ветка: feature/test-lints (от master `ca6fa4a`). Машина — macOS. Коммиты: `c93b22a` VAC-A (промоут),
`d8863ff` VAC-B1 (enabling), `6684f28` VAC-B2 (live-правило `disabled-test` в init; хук зелёный),
`47e52c4` VAC-C (нормы 7.2/7.3 — D075 + пункт 7 в `testing-discipline`; хук зелёный). B1 запинён+
забустрапен (stable.txt=`d8863ff`, НЕ закоммичен). Архив-VAC применён в рабочем дереве: P041-карточка
+ `## Delivery` → `Archive/Plan/041.md` (git mv), B032-карточка (ссылки `../`→`../../`) →
`Archive/Backlog/032.md` (git mv), строки Plan/Backlog сняты и добавлены в Archive/*, проза доставки
в Plan.md, ссылки [card] D074/D075 → `Archive/Backlog/032.md`, эта State. Сборка/коммит/бустрап/merge
— через `just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым).

## Progress

B032/P041 доставлен: 7.1 — правило `disabled-test` в init (D074, VAC-B2 6684f28), 7.2/7.3 — норма
(D075, VAC-C 47e52c4). Архив-VAC (эта правка) переносит карточки P041/B032 в Archive и мигрирует
ссылки. Осталась механика D051: финальный пин (stable.txt=голова архив-коммита + bootstrap) отдельным
коммитом, затем `just feature-merge`. После merge — отчёт и B033.

## Verification

VAC-B2 хук зелёный (rustfmt/clippy/90 rust/ruff/mypy/304 py вкл. test_delivery I020/typos/vulture).
VAC-C хук зелёный (181 py-теста вкл. test_delivery I020). Архив-VAC — только память (нет кода); хук
проверит memory-check после переноса карточек.

## Blockers

Нет.

## Next action

Закоммитить архив-VAC (git mv обеих карточек + строки Archive/* + проза Plan.md + миграция [card] +
эта State) через хук. Затем финальный пин D051: в одном `just scratch` записать stable.txt=полный хэш
головы архив-коммита + `just bootstrap`, затем закоммитить только stable.txt (`P041: pin <hash>`
(D051)). Затем `just feature-merge` (не пушить, ветку сохранить) → отчёт на русском на границе merge;
затем B033 (кластер 8: 8.1 неиспользуемые deps, 8.2 lockfiles+`--locked`, 8.3 бюджет зависимостей).
