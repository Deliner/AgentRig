# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Выбрано **P087 → P090**: P087 B081 → P088 B079 → P089 B080 → P090 B082 (зависит от P087). P087
слит в master `43c2f98`. **P088** (B079, правило `secret-debug`) выполнен на ветке, P088 и B079 в Archive; осталось
слияние.

## Workspace

Branch: `feature/secret-debug`
Revision: `b4920cfe8b2db1220e08ff50b40e6df49b5d4448` (фаза 2; коммит пина следует)

Ветки `feature/clippy-shape`, `feature/list-rule-cards` и прежние, а также `feature/init-probe`, сохранены.
Установленный рантайм — `b4920cf`. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет
cargo — коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P088 фаза 1 `e821c7a`: вид `SecretDebug`, детектор, тесты, D130; классификатор контекста в
  `languages/rust/context/`, тесты через CLI в `languages/tests/consumer/`. Пин фазы 1 `16ff2fe`.
- P088 фаза 2 `b4920cf`: правило в `.agentrig/lint.yaml`, каталог, реестр Rust, навык `signature/redact-secret`,
  Delivery, ротация P088 и B079.
- Пин D051: `stable.txt` — `b4920cf`, `just bootstrap` — код 0.
- Осталось: коммит пина, `just feature-merge`; затем P089, P090.

## Verification

- Гейты коммитов `e821c7a` (rust-test 157, test 312) и `b4920cf` (rust-test 157, test 342): все проверки PASS.
- `just candidate lint` с правилом: код 0 на AgentRig; проба с `password: String` — `ERROR [secret-debug]`.
- `init --language rust` в scratchpad: правило `secret-debug` и навык `redact-secret` поставлены.
- `just bootstrap` на `b4920cf`: код 0, `agentrig 0.3.0`.

## Blockers

Нет.

## Next action

Коммит пина D051 через хук, `just feature-merge` для `feature/secret-debug`, ветку сохранить; затем
`feature/unchecked-deserialize` для P089. Свободные ID: D131+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
