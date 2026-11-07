# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Карточки B079–B082 и D128 слиты в master `dbfad6c`. Выбрано **P087 → P090**: P087 B081 → P088 B079
→ P089 B080 → P090 B082 (зависит от P087). Сейчас **P087** (B081 clippy-shape), VAC 3 — `struct_excessive_bools`
и ротация P087 и B081 в Archive.

## Workspace

Branch: `feature/clippy-shape`
Revision: `56b7332` (VAC 2; VAC 3 не закоммичен)

Ветки `feature/list-rule-cards`, `feature/clippy-types` и прежние, а также `feature/init-probe`, сохранены.
Установленный рантайм — `51423f2`. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- VAC 1 `d9c828e`: `-D clippy::map_err_ignore`; два `join` в `command_runner/jobs` несут текст паники через
  `panic_text` в `streams.rs`; `Display` для `Kind` — под `#[allow(clippy::map_err_ignore)]` с `DECISION: D129`;
  D129 на весь набор B081; строки и карточки P087–P090 в Plan.
- VAC 2 `56b7332`: `-D clippy::partial_pub_fields`; `Session`, `Declaration`, `Selector` с закрытыми полями
  и методами доступа `state_mut`, `files`/`directories`/`kind`, `overrides`; пункт 5 нормы `type-driven-discipline`
  переписан, в разделе Enforcement назван линт.
- VAC 3 в рабочем дереве: `-D clippy::struct_excessive_bools`; P087 с Delivery и B081 перенесены в Archive, ссылка
  card в индексе D129 ведёт в Archive/Backlog/081.md.
- Дальше: пин D051, слияние, затем P088.

## Verification

- Гейт коммита `d9c828e`: все проверки PASS, rust-test 155 и test 395 passed. Проба `map_err(|_| …)` — код 101,
  после удаления — 0.
- Гейт коммита `56b7332`: все проверки PASS, rust-test 155 и test 342 passed. Проба
  `pub struct Probe { pub open: u8, closed: u8 }` — код 101, после удаления — 0.
- VAC 3: `just run clippy` — код 0; проба со структурой из четырёх `bool` — код 101 с «more than 3 bools in a
  struct», после удаления — 0.

## Blockers

Нет.

## Next action

Коммит VAC 3 через хук, пин D051 на голову ветки с `just bootstrap`, коммит пина, `just feature-merge`; затем
`feature/secret-debug` для P088. Свободные ID: D130+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
