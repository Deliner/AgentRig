# State

## Focus

Программа **P083 → P086** (пользователь 2026-11-07: «бери все и делай») — карточки B075–B078 по одной за ветку,
автономно. P083, P084 и P085 выполнены и слиты, master `52cfe55`. Сейчас **P086** (B075 clippy-types), VAC 1 —
AC1: `-D clippy::string_slice` в профиле clippy, 15 сайтов без среза строки, владелец `text()` под justified allow,
решение D127 на весь набор, AC6.

## Workspace

Branch: `feature/clippy-types`
Revision: `52cfe55969974b0beca314bf209408dd02f00ad0`

База — master `52cfe55`. Ветки `feature/exit-code`, `feature/disallowed-methods` и прежние, а также
`feature/init-probe`, сохранены. Установленный рантайм — `76de959`. macOS: в PATH `just write`/`just run`/
`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P086 VAC 1: `-D clippy::string_slice` в `.agentrig/commands.yaml`; сайты в `project_memory/format.rs`,
  `project_memory/mod.rs`, `project_memory/source.rs` (5), `shared/syntax/mod.rs`,
  `structural_lint/architecture/resolution/python/mod.rs`, `structural_lint/architecture/source/rust/conditions.rs`
  (2), `structural_lint/config/mod.rs`, `structural_lint/languages/manifest.rs`,
  `structural_lint/languages/rust/global_state.rs` и `structural_lint/languages/rust/mod.rs`; `text()` в
  `structural_lint/languages/measure/mod.rs` под `#[allow(clippy::string_slice)]` с `DECISION: D127`.
- Осталось: VAC 2 `iter_over_hash_type` с графом Plan в `BTreeMap`; VAC 3–5 `collection_is_never_read`,
  `read_zero_byte_vec`, `unchecked_time_subtraction` без сайтов; Delivery и ротация P086/B075; пин D051; слияние.
- Plan: P086 — active.

## Verification

- На базе `52cfe55` пять линтов в режиме `-W`: `string_slice` 16 сайтов, `iter_over_hash_type` 1, остальные 0.
- После ремонта: `just run clippy` с новым `-D` — код 0; проба `pub fn probe_slice(text: &str) -> &str { &text[1..] }`
  в `Project/lib.rs` — код 101 с ошибкой «indexing into a string may panic», после удаления — код 0.
- `cargo clippy --all-targets -- -D warnings` чисто; `cargo test --lib` — 155 passed; `just candidate lint` — без
  находок; `just config-check` — конфигурация валидна.

## Blockers

Нет.

## Next action

Коммит VAC 1 через хук, затем VAC 2 для `iter_over_hash_type`. Свободные ID: D128+, I022+, P087+, B079+ (B066 и
D111 зарезервированы).
