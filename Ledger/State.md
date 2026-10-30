# State

## Focus

Набор улучшений качества (кластеры 1–8, B026–B033) **завершён**. Доставлены и влиты в master: B027/P035,
B029/P036, B026/P037, B028/P038, B030/P039, B031/P040, B032/P041. Последняя карточка набора —
**B033/P042** (кластер 8, гигиена зависимостей и сборки) — реализована и архивируется: 8.1 неиспользуемые
объявленные external — правило; 8.2 локфайлы + `--locked` и 8.3 бюджет зависимостей — нормы.

## Workspace

Ветка: feature/dep-hygiene (база master @ `baca2f2`). VAC-A=`ba7610b`, VAC-B=`bf80117`. Текущий коммит —
«archive»: перенос карточек P042/B033 в Archive, проза Plan.md/Archive/Plan.md/Backlog.md/Archive/Backlog.md,
миграция `[card]`-ссылок D076/D077 на `Archive/Backlog/033.md`, этот State. stable.txt пока `5b32c87` —
финальный пин D051 идёт следующим коммитом на голову-«archive». Машина — macOS. Сборка/коммит/бустрап/merge —
через `just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым). ВАЖНО:
`just feature-merge` надо запускать ВНУТРИ `just scratch` с PATH. Agent-Bash — одна top-level `just`-команда
без `&&`/хвостового `| tail`.

## Progress

VAC-B (`bf80117`): 8.1 реализовано расширением правила `directory-architecture` (новый Kind/навык не нужен,
I020 без изменений). Движок: `RUST_SEED`/`PYTHON_SEED` (seed std/core/alloc и `__future__` — последнее
из-за слепого пятна tree-sitter на `future_import_statement`); `rust()` сохраняет `external`;
`Externals`/`record` агрегируют первый сегмент использованных external; `unused_externals`/`unused` в
runner/mod.rs дают находку «declared external … is never imported» на владельце. Из `.agentrig/lint.yaml`
убраны 7 реально неиспользуемых python-external. D076 (правило 8.1) и D077 (8.2/8.3 нормы) записаны.
Этот коммит — архив карточек P042/B033 и ротация таблиц/прозы.

## Verification

VAC-B зелёный: architecture-набор 106 passed; rust-юнит 90 passed; candidate-lint репо — только
предсуществующие warning (file-size/directory-size), 0 error, 0 находок «never imported». Коммит архива идёт
через хук — gate запустит memory-check (ссылки карточек/решений согласованы: D076/D077 `[card]` →
`Archive/Backlog/033.md`; карточки перенесены). master впереди origin на 161 (не пушить).

## Blockers

Нет.

## Next action

Закоммитить этот «archive» через хук (включая этот State). Затем финальный пин D051: в ОДНОМ `just scratch`
записать 40-символьную голову-«archive» в `tooling/distribution/stable.txt` и запустить `just bootstrap`,
затем коммит «P042: pin <archive-short> (D051)» через хук. Затем `just feature-merge` ВНУТРИ `just scratch`
с PATH (не пушить, ветку сохранить). Отчёт по-русски на границе merge. Свободные ID: D078.
