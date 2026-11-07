# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Карточки B079–B082 и D128 слиты в master `dbfad6c`. Выбрано **P087 → P090**: P087 B081 → P088 B079
→ P089 B080 → P090 B082 (зависит от P087). **P087** выполнен на ветке, осталось слияние.

## Workspace

Branch: `feature/clippy-shape`
Revision: `8901ff965cd7e88d7aeb3e59a69fc90365e6f102` (VAC 3; коммит пина следует)

Ветки `feature/list-rule-cards`, `feature/clippy-types` и прежние, а также `feature/init-probe`, сохранены.
Установленный рантайм — `8901ff9`. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P087 VAC 1 `d9c828e`: `-D clippy::map_err_ignore`, текст паники в двух `join`, justified allow в `Kind`, D129,
  выбор P087–P090 в Plan.
- P087 VAC 2 `56b7332`: `-D clippy::partial_pub_fields`, `Session`, `Declaration`, `Selector` с закрытыми полями,
  пункт 5 нормы `type-driven-discipline`.
- P087 VAC 3 `8901ff9`: `-D clippy::struct_excessive_bools`; P087 с Delivery и B081 в Archive.
- Пин D051: `stable.txt` — `8901ff9`, `just bootstrap` — код 0.
- Осталось: коммит пина, слияние; затем P088, P089, P090.

## Verification

- Гейты коммитов `d9c828e` (rust-test 155, test 395), `56b7332` (rust-test 155, test 342), `8901ff9` (rust-test 155,
  test 114): все проверки PASS.
- Пробы в `Project/lib.rs` для каждого из трёх линтов: код 101 с ошибкой линта, после удаления — код 0.
- `just bootstrap` на `8901ff9`: код 0, `agentrig 0.3.0`.

## Blockers

Нет.

## Next action

Коммит пина D051 через хук, `just feature-merge` для `feature/clippy-shape`, ветку сохранить; затем
`feature/secret-debug` для P088. Свободные ID: D130+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
