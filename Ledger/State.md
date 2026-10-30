# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены и влиты в master: B027/P035,
B029/P036, B026/P037, B028/P038, B030/P039, B031/P040, B032/P041. Идёт последняя карточка набора —
**B033/P042** (кластер 8, гигиена зависимостей и сборки): 8.1 неиспользуемые объявленные
external-зависимости [правило], 8.2 локфайлы + `--locked` [конфиг/проверка, уже так], 8.3 бюджет числа
зависимостей на фичу [мягкий порог]. VAC-A (промоут) и VAC-B (реализация 8.1) выполнены.

## Workspace

Ветка: feature/dep-hygiene (база master @ `baca2f2`). VAC-A закоммичен как `ba7610b`. stable.txt=`5b32c87`
(финальный пин D051 — на этапе архива). Машина — macOS. Сборка/коммит/бустрап/merge — через
`just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым). ВАЖНО:
`just feature-merge` надо запускать ВНУТРИ `just scratch` с PATH. Agent-Bash — одна top-level `just`-команда
без `&&`/хвостового `| tail`.

## Progress

VAC-A: B033 промоутнут в P042. VAC-B: 8.1 реализовано как расширение правила `directory-architecture`
(новый Kind/навык не нужен). Движок: `RUST_SEED`/`PYTHON_SEED` (seed-исключения std/core/alloc и
`__future__` — последнее из-за слепого пятна tree-sitter на `future_import_statement`); `rust()` сохраняет
`external`; `Externals`/`record` агрегируют первый сегмент использованных external; `dependencies()`
возвращает used; `unused_externals`/`unused` в runner/mod.rs дают находку «declared external … is never
imported» на владельце (feature_root). Конфиг: из `.agentrig/lint.yaml` убраны 7 реально неиспользуемых
python-external (contextlib, copy, hashlib, io, re, tarfile, tomllib). Тесты: 2 новых поведенческих
(python-unused + rust-seed) в test_architecture.py; фикстуры макро/атрибутов приведены к объявлению только
используемых external (иначе новая проверка справедливо ловила лишние). Решения D076 (правило 8.1) и
D077 (8.2/8.3 нормы) записаны в Ledger/Decisions.md + детальные файлы (гейт памяти требует D076 до кода).

## Verification

VAC-B зелёный: architecture-набор — 106 passed; rust-юнит — 90 passed; candidate-lint репо — только
предсуществующие warning (file-size/directory-size), ни одной error и ни одной находки «never imported».
Коммит идёт через хук (гейт на installed-бинаре 5b32c87, без новой проверки → находки installed ⊆ candidate).
master впереди origin на 161 (не пушить).

## Blockers

Нет.

## Next action

Закоммитить VAC-B через хук (движок 8.1 + конфиг + тесты + D076/D077 + этот State). Затем архив P042/B033
(двухкоммитная схема P036: сначала коммит «archive», затем коммит «pin» со stable.txt на archive-head;
миграция `[card]`-ссылок D076/D077 Backlog/033.md→Archive/Backlog/033.md, углубление ссылок карточки
`../`→`../../`, таблицы/проза Plan.md/Archive/Plan.md/Backlog.md/Archive/Backlog.md), финальный пин D051,
`just feature-merge` ВНУТРИ scratch с PATH (не пушить, ветку сохранить), отчёт по-русски. Свободные ID: D078.
