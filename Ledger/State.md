# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Карточки B079–B082 и D128 слиты в master `dbfad6c`. Выбрано **P087 → P090**: P087 B081 → P088 B079
→ P089 B080 → P090 B082 (зависит от P087). Сейчас **P087** (B081 clippy-shape), VAC 2 — `partial_pub_fields`.

## Workspace

Branch: `feature/clippy-shape`
Revision: `d9c828e` (VAC 1; VAC 2 не закоммичен)

Ветки `feature/list-rule-cards`, `feature/clippy-types` и прежние, а также `feature/init-probe`, сохранены.
Установленный рантайм — `51423f2`. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- VAC 1 `d9c828e`: `-D clippy::map_err_ignore`; два `join` в `command_runner/jobs` несут текст паники через
  `panic_text` в `streams.rs`; `Display` для `Kind` — под `#[allow(clippy::map_err_ignore)]` с `DECISION: D129`;
  D129 на весь набор B081; строки и карточки P087–P090 в Plan.
- VAC 2 в рабочем дереве: `-D clippy::partial_pub_fields`; `Session`, `Declaration`, `Selector` с закрытыми полями
  и методами доступа `state_mut`, `files`/`directories`/`kind`, `overrides`; пункт 5 нормы `type-driven-discipline`
  переписан, в разделе Enforcement назван линт.
- Дальше: VAC 3 `struct_excessive_bools` с ротацией P087 и B081, пин D051, слияние.

## Verification

- Гейт коммита `d9c828e`: все проверки PASS, rust-test 155 и test 395 passed. Проба `map_err(|_| …)` — код 101,
  после удаления — 0.
- VAC 2: `just run clippy` — код 0; проба `pub struct Probe { pub open: u8, closed: u8 }` в `Project/lib.rs` — код
  101 с «mixed usage of pub and non-pub fields», после удаления — код 0.

## Blockers

Нет.

## Next action

Коммит VAC 2 через хук, затем VAC 3. Свободные ID: D130+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
