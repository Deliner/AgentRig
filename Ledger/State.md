# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038, B030/P039, B031/P040 (в master). Текущая — B032/P041 (линты тестов): 7.1
тихо выключенные тесты → правило `disabled-test`; 7.2/7.3 — вероятно норма. Дальше — B033.

## Workspace

Ветка: feature/test-lints (от master `ca6fa4a`). Машина — macOS. Коммиты: `c93b22a` VAC-A
(промоут B032→P041), `d8863ff` VAC-B1 (enabling), `6684f28` VAC-B2 (live-правило `disabled-test`:
catalog::ALL + lint.yaml + навык `justify-skip` + D074 + индекс; хук зелёный). B1 запинён+забустрапен
(stable.txt=`d8863ff`, НЕ закоммичен). VAC-C применён в рабочем дереве (без кода): Ledger/Decisions/075.md
(нормы 7.2/7.3), testing-discipline/SKILL.md (пункт 7 детерминизм + секция Enforcement),
Ledger/Decisions.md (строка+проза D075), эта State. Сборка/коммит/бустрап/merge — через
`just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым).

## Progress

B1 (d8863ff) установлен; VAC-B2 (6684f28) доставил 7.1-правило `disabled-test` в init (хук зелёный,
I020-страж прошёл). VAC-C: 7.2 (детерминизм) и 7.3 (имя/размещение) записаны нормой D075 — 7.2 добавлен
пунктом 7 в доставляемый навык `testing-discipline`, 7.3 уже покрыт его пунктами 1 (размещение, D055)
и 5 (имя). Новых видов правил/навыков/инвариантов нет, наборы доставки I020 не меняются. B032/P041 по
содержанию готов (7.1 правило, 7.2/7.3 нормы); осталась механика доставки (архив + пин + merge).

## Verification

VAC-B2 хук зелёный: rustfmt/clippy/90 rust-тестов/ruff/mypy/304 py-теста (вкл. test_delivery I020,
test_lint_defaults)/typos/vulture — всё PASS. Кандидат `cargo test --lib languages::tests` — 13 passed.
VAC-C — только память+навык (нет кода); хук проверит память и затронутые тесты на коммите.

## Blockers

Нет.

## Next action

Закоммитить VAC-C (D075 + testing-discipline + индекс + эта State) через хук (stable.txt=`d8863ff` НЕ
трогать). Затем: архив P041/B032 (строки+карточки Plan/Backlog → Archive, мигрировать ссылку [card]
D074/D075 на `Archive/Backlog/032.md`, обновить State) отдельным коммитом → финальный пин
(stable.txt=голова архивного коммита + `just bootstrap` в одном scratch, закоммитить пин) →
`just feature-merge` (не пушить, ветку сохранить) → отчёт на русском на границе merge; затем B033.
