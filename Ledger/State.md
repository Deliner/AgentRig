# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Сохраняются B079 `secret-debug`, B080 `unchecked-deserialize`, B081 профиль dev-гейта
(`map_err_ignore`, `partial_pub_fields`, `struct_excessive_bools`), B082 поставка профиля clippy через `init` и решение
D128 о стандарте для нового кода. Затем по одной ветке на карточку: P087 B081 → P088 B079 → P089 B080 → P090 B082
(B082 зависит от B081).

## Workspace

Branch: `feature/list-rule-cards`
Revision: `cb258b786f120cb189f925c485b4aeb3c03fdba4`

База — master `cb258b7`, P086 слит. Ветки `feature/clippy-types` и прежние, а также `feature/init-probe`, сохранены.
Установленный рантайм — `51423f2`. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- Карточки B079–B082 и строки индекса Backlog записаны; D128 и строка индекса записаны. Не закоммичено.
- Замер clippy на `cb258b7` для B081: `map_err_ignore` 3, `partial_pub_fields` 3, `struct_excessive_bools` 0.

## Verification

Нет проверки для текущего содержимого ветки.

## Blockers

Нет.

## Next action

Коммит карточек и D128 через хук, `just feature-merge`, затем ветка P087 для B081 с выбором P087–P090 в Plan.
Свободные ID: D129+, I022+, P087+, B083+ (B066 и D111 зарезервированы).
