# State

## Focus

Программа **P083 → P086** (пользователь 2026-11-07: «бери все и делай») — карточки B075–B078 по одной за ветку,
автономно. P083, P084 и P085 выполнены и слиты, master `52cfe55`. Сейчас **P086** (B075 clippy-types), VAC 5 —
AC5: `-D clippy::unchecked_time_subtraction`, сайтов нет; Delivery и ротация P086 и B075 в архив.

## Workspace

Branch: `feature/clippy-types`
Revision: `0073f22f1378b127035eb6273f7e328d1dd53fa1`

База — master `52cfe55`. Ветки `feature/exit-code`, `feature/disallowed-methods` и прежние, а также
`feature/init-probe`, сохранены. Установленный рантайм — `76de959`. macOS: в PATH `just write`/`just run`/
`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.
Диск был заполнен до 128 МБ: первый коммит VAC 1 упал с «No space left on device»; `cargo clean --profile dev` для
`.cache/worker` освободил 19,2 ГБ.

## Progress

- P086 VAC 1 `666e61a`: `-D clippy::string_slice`, 15 сайтов без среза строки, `text()` в
  `structural_lint/languages/measure/mod.rs` под justified allow, решение D127 на весь набор.
- P086 VAC 2 `b28b9bd`: `-D clippy::iter_over_hash_type`; `graph` в `plan` и параметр `acyclic` — `BTreeMap`.
- P086 VAC 3 `211b6b1` и VAC 4 `0073f22`: `-D clippy::collection_is_never_read` и `-D clippy::read_zero_byte_vec`.
- P086 VAC 5: `-D clippy::unchecked_time_subtraction`; Delivery в Archive/Plan/086.md; P086 и B075 перенесены в
  Archive, ссылка card в индексе D127 ведёт в Archive/Backlog/075.md; Plan и Backlog пусты.
- Осталось: пин D051 и слияние.

## Verification

- `just run clippy` с новым `-D` — код 0; проба в `Project/lib.rs`: `timeout - elapsed` для `Duration` — код 101 с
  ошибкой «unchecked subtraction of a `Duration`», после удаления — код 0.
- Гейт коммита `0073f22`: все проверки PASS, test 114 passed; проба `read` в `Vec::with_capacity(8192)` — код 101 с
  «reading zero byte data to `Vec`».
- Гейты `666e61a` (rust-test 155, test 352), `b28b9bd` (test 233), `211b6b1` (test 114): все проверки PASS.

## Blockers

Нет.

## Next action

Коммит VAC 5 через хук, затем пин D051 (`git rev-parse HEAD > tooling/distribution/stable.txt && just bootstrap`),
коммит пина и `just feature-merge`. Свободные ID: D128+, I022+, P087+, B079+ (B066 и D111 зарезервированы).
