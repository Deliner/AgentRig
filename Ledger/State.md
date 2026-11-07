# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Выбрано **P087 → P090**: P087 B081 → P088 B079 → P089 B080 → P090 B082 (зависит от P087). P087
слит в master `43c2f98`. **P088** (B079, правило `secret-debug`) выполнен на ветке, P088 и B079 в Archive; осталось
закоммитить фазу 2, пин и слияние.

## Workspace

Branch: `feature/secret-debug`
Revision: `16ff2fe7ccfbe3833045bb5684b6c867c520300f` (пин фазы 1; коммит фазы 2 следует)

Ветки `feature/clippy-shape`, `feature/list-rule-cards` и прежние, а также `feature/init-probe`, сохранены.
Установленный рантайм — `e821c7a` (пин D051 фазы 1). macOS: в PATH `just write`/`just run`/`just check`/
`just feature-merge` нет cargo — коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P088 фаза 1 `e821c7a`: вид `SecretDebug`, дескриптор, детектор `languages/rust/secret_debug.rs`, тесты, D130;
  классификатор контекста в `languages/rust/context/`, тесты через CLI в `languages/tests/consumer/`.
- Пин D051 фазы 1: `stable.txt` — `e821c7a`, `just bootstrap` — код 0.
- P088 фаза 2 (не закоммичена): правило в `.agentrig/lint.yaml`, `catalog::ALL`, реестр Rust, навык
  `signature/redact-secret` с алиасом и `assets.rs`, `test_config_check.py`, Delivery, ротация P088 и B079, ссылки D130.
- Осталось: коммит фазы 2, финальный пин, `just feature-merge`; затем P089, P090.

## Verification

- Гейт коммита `e821c7a`: все проверки PASS, rust-test 157, test 312.
- `just candidate lint` с правилом: код 0 на AgentRig; проба `Project/secret_probe.rs` — `ERROR [secret-debug]
  … :3 (Probe.password)` с навыком `redact-secret`, проба удалена.
- `init --language rust` в пустом репозитории в scratchpad: правило `secret-debug` уровня error и навык
  `.agentrig/skills/redact-secret/SKILL.md`.

## Blockers

Нет.

## Next action

Коммит фазы 2 через хук, пин D051 на голову ветки, `just feature-merge`, затем `feature/unchecked-deserialize` для
P089. Свободные ID: D131+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
