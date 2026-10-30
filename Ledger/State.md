# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038, B030/P039, B031/P040 (в master). Текущая — B032/P041 (линты тестов): 7.1
тихо выключенные тесты → правило `disabled-test`; 7.2/7.3 — вероятно норма. Дальше — B033.

## Workspace

Ветка: feature/test-lints (от master `ca6fa4a`). Машина — macOS. Коммиты: `c93b22a` VAC-A
(промоут B032→P041), `d8863ff` VAC-B1 (enabling: вариант `DisabledTest` + descriptor + измерение +
регистрация в handler.rules + юнит-тесты; НЕ в catalog::ALL/lint.yaml). B1 запинён+забустрапен
(stable.txt=`d8863ff`, НЕ закоммичен). VAC-B2 (live) применён в рабочем дереве: catalog::ALL
(+`DisabledTest`), lint.yaml (правило `disabled-test` + `# DECISION: D074`), assets.rs (навык
`justify-skip`), references/justify-skip/{SKILL.md,architecture.yaml}, flat-symlink
`shared/skills/justify-skip`, references/architecture.yaml, маркеры `// DECISION: D074` (descriptor.rs,
rust.rs, python.rs, tests/mod.rs), Ledger/Decisions/074.md + индекс. Сборка/коммит/бустрап/merge —
через `just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым).

## Progress

B1 доставлен (d8863ff), B1 установлен. VAC-B2: 7.1-правило `disabled-test` live в lint.yaml +
catalog::ALL, навык `justify-skip` в группе `references`, D074 записан (карточка + индекс-строка +
newest-first проза + ссылки на владельцев). Кандидатный догфуд зелёный (`just lint` exit 0 после
добавления references/justify-skip/architecture.yaml; `just lint-config-check` valid). Установленный
B1 понимает `kind: disabled-test` (serde + descriptor + handler.rules), поэтому хук на коммите VAC-B2
пройдёт. 7.2/7.3 → норма D075 (VAC-C). Новых инвариантов нет. В нашем коде выключенных тестов нет.

## Verification

Кандидат: `cargo test --lib languages::tests` — 13 passed (вкл. новые
`rust_flags_ignore_without_a_reference_on_the_line_above`,
`python_flags_skip_markers_without_a_justifying_reference`). Кандидат `just lint` exit 0,
`just lint-config-check` — valid. Навигация по узлам подтверждена.

## Blockers

Нет.

## Next action

Закоммитить VAC-B2 (код + навык + D074 + эта State) через хук (stable.txt=`d8863ff` НЕ трогать).
Затем VAC-C (нормы D075: 7.2 недетерминизм, 7.3 имя/размещение — норма в `testing-discipline`) →
архив P041/B032 (мигрировать ссылку [card] на Archive/Backlog/032.md) → финальный пин
(двухкоммитная схема P036) → `just feature-merge` (не пушить, ветку сохранить) → отчёт на русском на
границе merge; затем B033.
