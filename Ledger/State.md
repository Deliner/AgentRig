# State

## Focus

Программа **P083 → P086** (пользователь 2026-11-07: «бери все и делай») — карточки B075–B078 по одной за ветку,
автономно. P083, P084 и P085 выполнены и слиты, master `52cfe55`. Сейчас **P086** (B075 clippy-types), VAC 2 —
AC2: `-D clippy::iter_over_hash_type`, граф зависимостей Plan в `project_memory/mod.rs` хранится в `BTreeMap`.

## Workspace

Branch: `feature/clippy-types`
Revision: `666e61a7e178b3a45346b2c56c3bbe48dd8459a4`

База — master `52cfe55`. Ветки `feature/exit-code`, `feature/disallowed-methods` и прежние, а также
`feature/init-probe`, сохранены. Установленный рантайм — `76de959`. macOS: в PATH `just write`/`just run`/
`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.
Диск был заполнен до 128 МБ: первый коммит VAC 1 упал с «No space left on device»; `cargo clean --profile dev` для
`.cache/worker` освободил 19,2 ГБ.

## Progress

- P086 VAC 1 `666e61a`: `-D clippy::string_slice`, 15 сайтов без среза строки, `text()` в
  `structural_lint/languages/measure/mod.rs` под justified allow, решение D127 на весь набор.
- P086 VAC 2: `-D clippy::iter_over_hash_type`; `graph` в `plan` и параметр `acyclic` — `BTreeMap`, метка
  `DECISION: D127`, ссылка graph в индексе D127.
- Осталось: VAC 3–5 `collection_is_never_read`, `read_zero_byte_vec`, `unchecked_time_subtraction` без сайтов;
  Delivery и ротация P086/B075; пин D051; слияние.
- Plan: P086 — active.

## Verification

- После ремонта: `just run clippy` с новым `-D` — код 0; проба `for key in map.keys()` по `HashMap` в
  `Project/lib.rs` — код 101 с ошибкой «iteration over unordered hash-based type», после удаления — код 0.
- Гейт коммита `666e61a`: все проверки PASS, rust-test 155 и test 352 passed.

## Blockers

Нет.

## Next action

Коммит VAC 2 через хук, затем VAC 3 для `collection_is_never_read`. Свободные ID: D128+, I022+, P087+, B079+ (B066
и D111 зарезервированы).
