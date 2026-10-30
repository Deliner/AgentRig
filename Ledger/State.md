# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Текущая —
[P038](Plan/038.md) ([B028](Backlog/028.md)): дисциплина ошибок и ресурсов. По выбору
пользователя 2026-10-30 (вариант A): 3.1 — структурное правило `silent-error` для Python
`except`-тела из одного `pass` (уехало в init, D069); Rust `let _ = <Result>` и 3.2 — на
dev-гейте через clippy `let_underscore_must_use` (D068); 3.3 (ресурсы) уже обеспечено `SIM`.

## Workspace

Ветка: feature/error-discipline (от master `9ca3be2`), HEAD `07d7991` (VAC-B1). Рабочее дерево:
VAC-B2 готов, не закоммичен. stable.txt в рабочем дереве = `07d7991` (B1, установлен
бустрапом; промежуточный пин, не коммитить); в HEAD stable.txt = `743a2bb`. Машина — macOS.
Сборка/clippy/fmt/тесты/коммит/бустрап — через `just scratch -- '...'` с PATH (`cargo` в
`~/.cargo/bin`, venv `tooling/.venv/bin` первым для Python-гейта); `just run`/гейт не видят
cargo в PATH без этого.

## Progress

VAC-A (`af46b97`): clippy `let_underscore_must_use`, `process.rs` на `.ok()`, D068.
VAC-B1 (`07d7991`): `Kind::SilentError` + дескриптор + Python-хендлер + измерение
`silent_except` + тест; `justified` в общий `languages/mod.rs`; пин+бустрап B1. VAC-B2
(go-live) готов в рабочем дереве: правило `silent-error` в `.agentrig/lint.yaml` (Python,
error) и `Kind::SilentError` в `catalog::ALL`; навык `handle-error` под `references/`
(тело + architecture.yaml + плоский симлинк `shared/skills/handle-error` + запись в
`assets.rs`); маркеры `// DECISION: D069` (дескриптор, `silent_except`, тест-коллектор);
карточка D069 + индекс/проза в Decisions.md; State. Догфуд чист (0 `except` в репозитории).

## Verification

86 Rust-тестов (`cargo test --lib`) зелёные. Линт репозитория установленным бинарём с
рабочим `lint.yaml` — чисто (0 находок `silent-error`). Delivery/I020
(`test_delivery.py`) — 4 passed (delivered == live по rule-kinds/skills/routes). config-check —
валиден.

## Blockers

Нет.

## Next action

1) Закоммитить VAC-B2 (lint.yaml, catalog, descriptor/python/tests маркеры, навык handle-error
+ architecture.yaml + симлинк + assets.rs, references/architecture.yaml, Decisions.md,
Decisions/069.md, State) через хук; stable.txt НЕ коммитить. 2) Архив P038/B028: Plan/038→
Archive/Plan/038, Backlog/028→Archive/Backlog/028, перенести card-ссылки D068 и D069 на
Archive/Backlog/028.md, обновить индексы Plan.md/Backlog.md, добавить Delivery в карточку.
3) Финальный пин (D051): stable.txt = голова архивного коммита + `just bootstrap`, отдельный
коммит `pin <head>`. 4) `just feature-merge`. Не пушить, ветку сохранить.
