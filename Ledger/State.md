# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Доставлены B027/P035, B029/P036,
B026/P037, B028/P038 (в master). [P039](Archive/Plan/039.md) ([B030](Archive/Backlog/030.md)) —
структура как карта функционала — реализована и завершена: 5.2 (качество `purpose`) уехало в
init ужесточением правила `directory-architecture` (D070); 5.1 (семьи вместе) и 5.3 (одна
возможность = файл) — норма (D071). Осталась финализация ветки (пин D051 + feature-merge).
Следующие карточки набора — B031–B033.

## Workspace

Ветка: feature/structure-map (от master `0ed526f`), HEAD `7e03276` (VAC-C). Машина — macOS.
Рабочее дерево: карточки P039/B030 перенесены в Archive (Plan/039→Archive/Plan/039 с Delivery,
Backlog/030→Archive/Backlog/030), индексы Plan.md/Backlog.md/Archive/Plan.md/Archive/Backlog.md
обновлены, card-ссылки D070/D071 в Decisions.md переведены на `Archive/Backlog/030.md`, State —
этот рефреш; всё не закоммичено. stable.txt = `3ad2838` (установлен бустрапом на прошлой ветке;
будет перезаписан финальным пином этой ветки). Сборка/clippy/fmt/тесты/коммит/бустрап — через
`just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым для
Python-гейта); `just run`/гейт не видят cargo в PATH без этого.

## Progress

Реализация завершена и закоммичена: VAC-A (`c62149b`) plan-only промоушен B030→P039; VAC-B
(`165dfd9`) 5.2 — ужесточена валидация `purpose` в `directory-architecture` (`policy.rs`:
фикс-набор болванок/дженерик-токенов), тесты (reject + позитивный на фразу с дженерик-словом),
маркер+карточка D070; без нового `Kind`/`catalog::ALL`/навыка, I020 не меняется; VAC-C
(`7e03276`) норма 5.1/5.3 решением D071. Архив P039/B030 выполнен в рабочем дереве (см.
Workspace).

## Verification

VAC-B: `cargo test --lib architecture::` 40 passed; кандидат-линт репозитория exit 0 (0 находок
болванок). Коммит-гейт каждого VAC зелёный; VAC-B полный гейт — 64 Rust + 200 Python (вкл.
architecture/tests и Delivery/I020) + ruff/mypy/typos/vulture все PASS. Ссылки Ledger после
переноса проверены grep — битых нет, кроме этого State (чинится рефрешем).

## Blockers

Нет.

## Next action

1) Закоммитить архив P039/B030 через хук (Ledger-файлы; stable.txt НЕ коммитить). 2) Финальный
пин (D051, двухкоммитная схема P036): stable.txt = голова архивного коммита + `just bootstrap`
в одном `just scratch`, отдельный коммит `pin <head>`. 3) `just feature-merge` (тесты всей
ветки). Не пушить, ветку сохранить. 4) Отчёт пользователю на русском на границе merge; затем
следующая карточка набора (B031–B033).
