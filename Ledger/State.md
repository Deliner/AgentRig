# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Текущая —
[P038](Plan/038.md) ([B028](Backlog/028.md)): дисциплина ошибок и ресурсов. По выбору
пользователя 2026-10-30 (вариант A): 3.1 — структурное правило `silent-error` для Python
`except`-тела из одного `pass` (уезжает в init); Rust `let _ = <Result>` и 3.2 — на dev-гейте
через clippy `let_underscore_must_use` (D068); 3.3 (ресурсы) уже обеспечено группой `SIM` в
ruff-select.

## Workspace

Ветка: feature/error-discipline (от master `9ca3be2`), HEAD `af46b97` (VAC-A). stable.txt =
`743a2bb` (голова P037, установлена). Машина — macOS. Сборка/clippy/fmt/тесты — через
`just scratch -- '...'` с PATH (`cargo` в `~/.cargo/bin`); `just run`/гейт не видят cargo в PATH.

## Progress

VAC-A закоммичен (`af46b97`): clippy `-D let_underscore_must_use`, `process.rs` teardown на
`.ok()`, D068; 3.3 уже покрыт `SIM`. VAC-B1 (enabling) готов в рабочем дереве, не закоммичен:
добавлен `Kind::SilentError` + дескриптор `SILENTERROR` (skill `handle-error`) + регистрация в
Python-хендлере + измерение `silent_except` в `python.rs` (тело `except` из одного `pass`;
escape — ссылка `DECISION:`/`INVARIANT:` строкой прямо над `pass`) + поведенческий тест;
общий `justified` поднят в `languages/mod.rs` и переиспользован Rust и Python. НЕ в
`catalog::ALL`, НЕ в `lint.yaml`, без маркеров — двухкоммитная схема (установленный бинарь
должен распознавать `kind` до правки `lint.yaml`).

## Verification

Кандидат собирается; 11 тестов `structural_lint::languages::tests` зелёные (вкл.
`python_flags_except_pass...` = `[4, 22]` — ловит и отступный, и однострочный `except: pass`,
escape и «тело не только pass» учтены). `cargo fmt --check` и `clippy` (dev-профиль с
`let_underscore_must_use`) — чисто.

## Blockers

Нет.

## Next action

1) Закоммитить VAC-B1 (7 файлов `structural_lint` + State) через хук. 2) Пин: записать голову
B1 в `stable.txt` + `just bootstrap` одним `just scratch`. 3) VAC-B2 (go-live): `silent-error`
в `.agentrig/lint.yaml` (Level/error, `.py`/`.pyi`, skill `handle-error`, маркер
`# DECISION: D069`), `Kind::SilentError` в `catalog::ALL`, навык ремонта `handle-error`
(+ запись в `assets.rs`), маркеры `// DECISION: D069` (дескриптор, `silent_except`,
тест-коллектор), запись D069, рефреш State; догфуд чист (0 `except` в репозитории).
4) Архив P038/B028, финальный пин (D051), `just feature-merge`. Не пушить, ветку сохранить.
