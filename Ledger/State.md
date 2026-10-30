# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038, B030/P039 (в master) и теперь B031/P040 — линты именования: 6.1
имена-помойки доставлено в init правилом `dump-name` (D072); 6.2/6.3 — норма (D073). P040/B031
перенесены в Archive. Осталась финализация ветки: финальный пин (D051) + feature-merge. Дальше —
B032–B033.

## Workspace

Ветка: feature/naming-lints (от master `e71b1cc`). Машина — macOS. Коммиты: `30520d5` VAC-A,
`2b98cb7` VAC-B1 (enabling), `7d819bc` VAC-B2 (live rule + D072), `ad7dd72` VAC-C (D073). Архив
P040/B031 в рабочем дереве, НЕ закоммичен: Archive/Plan/040.md (+ Delivery) и Archive/Backlog/031.md
созданы; строки P040/B031 убраны из Plan.md/Backlog.md и добавлены в Archive/Plan.md/Archive/Backlog.md;
проза-завершение в Plan.md; ссылки card D072/D073 в Decisions.md → `Archive/Backlog/031.md`; этот
State. Оригиналы Plan/040.md и Backlog/031.md ещё на диске — удалить через `git rm` в коммите
архива. stable.txt = `2b98cb7` (промежуточный пин B1, НЕ закоммичен). Сборка/коммит/бустрап/merge —
через `just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым).

## Progress

Реализация и память B031 завершены. 6.1 — правило `dump-name` (двухкоммитная схема: B1 enabling
пин+бустрап, B2 live), навык переиспользован, I020 по skills не меняется. 6.2/6.3 — норма (D073).
Остался только перенос в Archive (готов в дереве) и финализация ветки.

## Verification

VAC-B2 гейт зелёный (88 Rust + 282 Python, вкл. test_delivery/I020 и architecture/tests). Кандидат
`just lint` exit 0. Ссылки Ledger после переноса: проверить memory-check на коммите архива (должен
быть зелёным; оригиналы удаляются тем же коммитом).

## Blockers

Нет.

## Next action

1) Коммит архива P040/B031 через хук: `git rm Ledger/Plan/040.md Ledger/Backlog/031.md` + `git add`
остальных Ledger-файлов; stable.txt НЕ коммитить. 2) Финальный пин (двухкоммитная схема P036):
stable.txt = голова архивного коммита + `just bootstrap` в одном `just scratch`, отдельный коммит
`pin <head>`. 3) `just feature-merge` (тесты всей ветки); не пушить, ветку сохранить. 4) Отчёт на
русском на границе merge; затем следующая карточка набора (B032 — линты тестов, затем B033).
