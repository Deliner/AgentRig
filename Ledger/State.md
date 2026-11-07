# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Выбрано **P087 → P090**: P087 B081 → P088 B079 → P089 B080 → P090 B082 (зависит от P087). P087
слит в master `43c2f98`. **P088** (B079, правило `secret-debug`) в работе, фаза 1 из двух (D051).

## Workspace

Branch: `feature/secret-debug`
Revision: `43c2f98dc7b7479c3c51006bdd134ea331e42bf7` (база; коммит фазы 1 следует)

Ветки `feature/clippy-shape`, `feature/list-rule-cards` и прежние, а также `feature/init-probe`, сохранены.
Установленный рантайм — `8901ff9`. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P088 фаза 1: вид `SecretDebug` в `kind.rs`, дескриптор `secret-debug` (D130), детектор
  `languages/rust/secret_debug.rs` с разбором `derive(Debug)` и `cfg_attr`, диспетчер `struct_item`, тесты
  `languages/tests/secret_debug.rs`, D130. Правило ещё не включено. Линт размера каталогов (16 из 15 в
  `languages/rust` и `languages/tests`): классификатор контекста узла (`test_context.rs`, `async_context.rs`)
  вынесен в `languages/rust/context/`, тесты через CLI — в `languages/tests/consumer/`; ссылки D115, D117, D118, I015
  и цель оракула I015 обновлены. Вариант с `rust/async_code/` давал циклы каталогов и отвергнут.
- Осталось: коммит фазы 1, пин D051, фаза 2 — правило в `.agentrig/lint.yaml`, каталог, реестр, навык
  `redact-secret`, проверка AC4, Delivery, ротация; затем P089, P090.

## Verification

- `cargo test --lib secret_debug`: 2 теста PASS; полный `cargo test`: 157 PASS после переноса.
- Оракул I015 по новому пути и `test_language_delivery.py`: 6 PASS.

## Blockers

Нет.

## Next action

Коммит фазы 1 через хук, затем пин D051 (`stable.txt` и `just bootstrap`) и фаза 2. Свободные ID: D131+, I022+,
P091+, B083+ (B066 и D111 зарезервированы).
