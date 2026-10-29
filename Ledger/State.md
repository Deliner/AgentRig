# State

## Focus

Программа улучшений качества кода «из коробки» (мозгоштурм 2026-10-29, кластеры 1–8,
9-й исключён). Набор зафиксирован в Backlog карточками [B026](Backlog/026.md)–[B033](Backlog/033.md).
Дальше — реализация по одной карточке через догфудинг: каждая = своя ветка → гейт → пин
→ merge, наш код обязан проходить новое правило. Пороги не ослаблять ради проходимости.

## Workspace

Branch: feature/quality-backlog (Ledger-only, фиксация набора; Project не менялся, пин не
нужен).

Revision: от master после влития feature/semantic-symbols.

Машина — macOS (Darwin 25.5.0). Реализационные карточки будут менять Project (движок/
поставка init) → каждой нужен пин по D051 перед merge. Merge/бустрап — через
`just scratch -- '...'` с PATH.

## Progress

Зафиксировано 8 карточек: B026 безопасность (`unsafe`-обоснование, bandit, секреты);
B027 строгие профили clippy/ruff/mypy; B028 ошибки/ресурсы; B029 сложность (вложенность,
флаговые параметры, бюджет ветвлений); B030 структура как карта (семьи вместе, качество
architecture.yaml, одна возможность=файл); B031 именование (свалки, осмысленность,
магические числа); B032 тесты (выключенные-с-обоснованием, недетерминизм, конвенции);
B033 зависимости/сборка. Индекс Backlog обновлён.

Уже есть в движке (не дублировать): file-size, function-lines(40), parameter-count(4),
named-if-condition, comment-reference, markdown-size/width/placement, directory-size,
directory-architecture (контракты+циклы+типы фич, ловит необъявленные внешние зависимости).
Clippy сейчас только `-D warnings` без restriction; ruff/mypy — профиль из tooling/pyproject.

## Verification

Коммит-гейт (build, lint, memory, rustfmt, clippy, rust-test, ruff/ruff-format/mypy,
pytest, typos, vulture). Эта ветка — Ledger-only, memory-check индекса Backlog.

## Blockers

Нет.

## Next action

Влить feature/quality-backlog. Затем реализовать карточки по порядку (конфиг/простые
правила раньше, структура позже): предлагаемый порядок B027 → B028 → B029 → B031 → B026
→ B032 → B033 → B030; промоут каждой в Plan при старте её ветки (edit-plan), догфуд до
зелёного, пин D051, merge, архивация. Порядок и границы VAC — за исполнителем. Не пушить
без просьбы.
