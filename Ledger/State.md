# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Выбрано **P087 → P090**: P087 B081 → P088 B079 → P089 B080 → P090 B082 (зависит от P087). P087
слит в master `43c2f98`. **P088** (B079, правило `secret-debug`) в работе, фаза 1 из двух (D051).

## Workspace

Branch: `feature/secret-debug`
Revision: `e821c7a4449bbd0a0bcb3df6f0c4d410df6456e8` (фаза 1; коммит пина следует)

Ветки `feature/clippy-shape`, `feature/list-rule-cards` и прежние, а также `feature/init-probe`, сохранены.
Установленный рантайм — `e821c7a` (пин D051 фазы 1). macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P088 фаза 1 `e821c7a`: вид `SecretDebug` в `kind.rs`, дескриптор `secret-debug` (D130), детектор
  `languages/rust/secret_debug.rs` с разбором `derive(Debug)` и `cfg_attr`, диспетчер `struct_item`, тесты
  `languages/tests/secret_debug.rs`, D130. Правило ещё не включено. Линт размера каталогов (16 из 15 в
  `languages/rust` и `languages/tests`): классификатор контекста узла (`test_context.rs`, `async_context.rs`)
  вынесен в `languages/rust/context/`, тесты через CLI — в `languages/tests/consumer/`; ссылки D115, D117, D118, I015
  и цель оракула I015 обновлены. Вариант с `rust/async_code/` давал циклы каталогов и отвергнут.
- Пин D051: `stable.txt` — `e821c7a`, `just bootstrap` — код 0.
- Осталось: коммит пина, фаза 2 — правило в `.agentrig/lint.yaml`, каталог, реестр, навык
  `redact-secret`, проверка AC4, Delivery, ротация; затем P089, P090.

## Verification

- Гейт коммита `e821c7a`: lint, memory, rustfmt, clippy, rust-test 157, test 312 — PASS.
- Оракул I015 по новому пути и `test_language_delivery.py`: 6 PASS.
- `just bootstrap` на `e821c7a`: код 0, `agentrig 0.3.0`.

## Blockers

Нет.

## Next action

Коммит пина D051 через хук, затем фаза 2. Свободные ID: D131+, I022+,
P091+, B083+ (B066 и D111 зарезервированы).
