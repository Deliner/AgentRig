# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Карточки B079–B082 и D128 слиты в master `dbfad6c`. Выбрано **P087 → P090**: P087 B081 → P088 B079
→ P089 B080 → P090 B082 (зависит от P087). Сейчас **P087** (B081 clippy-shape), VAC 1 — `map_err_ignore` и выбор
P087–P090 в Plan.

## Workspace

Branch: `feature/clippy-shape`
Revision: `dbfad6c` (база, VAC 1 не закоммичен)

Ветки `feature/list-rule-cards`, `feature/clippy-types` и прежние, а также `feature/init-probe`, сохранены.
Установленный рантайм — `51423f2`. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- VAC 1 в рабочем дереве: `-D clippy::map_err_ignore`; два `join` в `command_runner/jobs` несут текст паники через
  `panic_text` в `streams.rs`; `Display` для `Kind` — под `#[allow(clippy::map_err_ignore)]` с `DECISION: D129`;
  D129 на весь набор B081; строки и карточки P087–P090 в Plan.
- Дальше в P087: VAC 2 `partial_pub_fields` — `Session`, `Declaration`, `Selector` с закрытыми полями и методами
  доступа, пункт 5 нормы `type-driven-discipline`; VAC 3 `struct_excessive_bools` с ротацией P087 и B081.

## Verification

- `just run clippy` с `-D clippy::map_err_ignore` — код 0.
- Проба в `Project/lib.rs`: `text.parse::<u8>().map_err(|_| String::from("bad"))` — код 101 с ошибкой
  «`map_err(|_|...` wildcard pattern discards the original error», после удаления — код 0.

## Blockers

Нет.

## Next action

Коммит VAC 1 через хук, затем VAC 2. Свободные ID: D130+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
