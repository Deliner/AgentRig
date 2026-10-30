# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038, B030/P039 (в master). Текущая — [B031](Backlog/031.md)/[P040](Plan/040.md)
(линты именования): 6.1 имена-помойки доставлено в init новым правилом `dump-name` (D072);
6.2 (осмысленные идентификаторы) и 6.3 (магические числа) — норма (D073). Реализация завершена;
осталась финализация ветки (архив + пин D051 + feature-merge).

## Workspace

Ветка: feature/naming-lints (от master `e71b1cc`). Машина — macOS. Коммиты ветки: `30520d5`
VAC-A (промоушен B031→P040), `2b98cb7` VAC-B1 (enabling: `DumpName` kind + мера + юнит-тест),
`7d819bc` VAC-B2 (в `catalog::ALL` + правило `.agentrig/lint.yaml` + маркеры D072 + карточка),
VAC-C (D073, 6.2/6.3 — норма) не закоммичен (Decisions/073.md + Decisions.md + этот State).
stable.txt = `2b98cb7` (промежуточный пин B1, НЕ закоммичен; будет перезаписан финальным пином на
архивную голову). Сборка/clippy/fmt/тесты/коммит/бустрап — через `just scratch -- '...'` с PATH
(`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым).

## Progress

6.1 доставлено двухкоммитной схемой lint-kind: B1 `DumpName` (target directory, Level), мера в
`structural_lint/mod.rs` по basename каталога против `{utils, helpers, misc, common}` (`shared`
вне набора), юнит-тест; пин+бустрап B1 (installed `2b98cb7` знает вариант). B2: `catalog::ALL` +=
DumpName, правило `dump-name` в `.agentrig/lint.yaml` (scope Project), маркеры D072 в
descriptor.rs/mod.rs/tests/mod.rs/lint.yaml, карточка+индекс+проза D072. Навык переиспользован
(`refactor-large-directory`), I020 по skills не меняется. VAC-C: D073 — 6.2/6.3 остаются нормой
(scope-анализ / шум магических чисел против near-zero-FP бара D035), без правила и навыка.

## Verification

VAC-B1 гейт зелёный (65 Rust + 200 Python). VAC-B2 гейт зелёный (88 Rust + 282 Python, вкл.
test_delivery/I020 и architecture/tests — delivered и live rule-kinds совпадают с `dump-name`,
skills без изменений). Кандидат `just lint` exit 0 (в Project нет каталогов-свалок, догфуд
чист). VAC-C — память; гейт предстоит на коммите.

## Blockers

Нет.

## Next action

1) Закоммитить VAC-C (Decisions/073.md + Decisions.md + State) через хук. 2) Архив P040/B031:
Plan/040.md→Archive/Plan/040.md (+ Delivery), Backlog/031.md→Archive/Backlog/031.md, строки
Plan.md/Backlog.md/Archive/Plan.md/Archive/Backlog.md, ссылки card D072/D073 в Decisions.md →
`Archive/Backlog/031.md`; коммит архива. 3) Финальный пин (двухкоммитная схема P036): stable.txt =
голова архивного коммита + `just bootstrap` в одном `just scratch`, отдельный коммит `pin <head>`.
4) `just feature-merge`; не пушить, ветку сохранить. 5) Отчёт на русском на границе merge; затем
B032–B033.
