# State

## Focus

Программа **P083 → P086** (пользователь 2026-11-07: «бери все и делай») — карточки B075–B078 по одной за ветку,
автономно. P083 (B077) выполнена и слита в master `d0d4f2e`. **P084** (B078 disallowed-methods) выполнена в VAC
фазы 2 `757bc30`: сайты AgentRig вызывают обёртки `shared/fs`, запись `std::fs` в `.agentrig/lint.yaml` перечисляет
методы ФС; P084 и B078 в Archive, решение D125. Этот коммит — финальный пин D051.

## Workspace

Branch: `feature/disallowed-methods`
Revision: `757bc30f299aece4d54d5f385af5be750fa90144`

База — master `d0d4f2e`. Ветки `feature/impact-coverage`, `feature/rule-cards` и прежние, а также
`feature/init-probe`, сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P084 фаза 1 — VAC `f7a539f`: поле `methods`, извлечение вызовов в `architecture/source/rust/calls.rs`, в том
  числе в аргументах макросов, политика, тест, README, D125. Пин D051 `5a05e0b`: `stable.txt` = `f7a539f`.
- P084 фаза 2: обёртки `file_metadata` и `try_exists` в `Project/shared/fs/mod.rs` с тестом, 15 строк переведены на
  владельца; корень проекта разрешает `project_root` в `project_config/settings`, иначе `project_config` стал бы
  21-м зависимым `shared/fs` при бюджете 20.
- Plan: P085 (B076) и P086 (B075) — pending.

## Verification

- Проба кандидата с копией конфига в scratch до ремонта: 15 строк, 16 вызовов, как в карточке B078.
- После ремонта: `cargo clippy --all-targets -- -D warnings` чисто; `cargo test shared::fs` — 3 passed;
  `just candidate lint` — код 0; `just candidate config-check` — «Scaffold configuration is valid (runtime 0.3.0)».
- Гейт коммита `f7a539f`: все проверки PASS, test 226 passed; `just bootstrap` на `f7a539f` прошёл.
- Гейт коммита `757bc30`: все проверки PASS, rust-test 152 и test 385 passed; `just bootstrap` на `757bc30` —
  установлен `agentrig 0.3.0`.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/disallowed-methods`. Затем ветка
`feature/exit-code` для P085 (B076). Свободные ID: D126+, I022+, P087+, B079+ (B066 и D111 зарезервированы).
