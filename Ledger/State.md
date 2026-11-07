# State

## Focus

Программа **P083 → P086** (пользователь 2026-11-07: «бери все и делай») — карточки B075–B078 по одной за ветку,
автономно. P083 (B077) выполнена и слита в master `d0d4f2e`. Текущая — **P084** (B078 disallowed-methods): запись
`disallowed` перечисляет методы, вызов без аргументов с пробросом ошибки через `?` или `.map_err(Into::into)` вне
`allow` — находка; решение D125. Этот VAC — фаза 1 (AC1): поле и проверка без изменения `.agentrig/lint.yaml`.

## Workspace

Branch: `feature/disallowed-methods`
Revision: `d0d4f2e762b5acc9eadf596c4343565a164fc23f`

База — master `d0d4f2e`. Ветки `feature/impact-coverage`, `feature/rule-cards` и прежние, а также
`feature/init-probe`, сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P084 фаза 1: поле `methods` в `structural_lint/rules/architecture.rs`, извлечение вызовов в
  `architecture/source/rust/calls.rs`, в том числе в аргументах макросов через `macros.rs`, политика
  `runner/disallowed.rs`, вызов из `runner/analysis.rs`, тест `test_architecture_disallowed.py`, README, D125.
- Установленный runtime `f2b4962` отвергает поле `methods` в `.agentrig/lint.yaml` (deny_unknown_fields), поэтому
  после этого VAC нужен пин D051, затем фаза 2.
- Plan: P084 active, P085 (B076) и P086 (B075) pending.

## Verification

- `cargo clippy --all-targets -- -D warnings` — чисто; `test_architecture_disallowed.py` — 11 passed.
- Проба кандидата с копией конфига в scratch, где запись `std::fs` перечисляет `canonicalize`, `read_dir`,
  `metadata`, `symlink_metadata`, `read_link` и `try_exists`: 15 строк, 16 вызовов, как в карточке B078; до
  разбора аргументов макросов было 11 строк, пропущены 4 строки в `ensure!`.

## Blockers

Нет.

## Next action

Коммит фазы 1, пин D051 (`stable.txt` = голова ветки, `just bootstrap`). Фаза 2: обёртки `try_exists` и
`file_metadata` в `Project/shared/fs/mod.rs`, перевод 15 строк на владельца, `methods` в `.agentrig/lint.yaml`,
`just candidate lint` — 0 находок, Delivery и ротация P084/B078. Свободные ID: D126+, I022+, P087+, B079+ (B066 и
D111 зарезервированы).
