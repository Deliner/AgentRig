# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Выбрано **P087 → P090**. P087 и P088 слиты в master, последний merge `21c0b30`. **P089**
(B080, `unchecked-deserialize`) приостановлен: нужен выбор пользователя о структуре реестра видов, см. Delivery
[P089](Plan/089.md). **P090** (B082, поставка clippy в `init`) выполнен на ветке, P090 и B082 в Archive.

## Workspace

Branch: `feature/clippy-delivery`
Revision: `21c0b3005d838fdcdc1c0b0d7c4922ffa95c9a2f` (база; коммит VAC следует)

Ветка `feature/unchecked-deserialize` без коммитов, работа P089 в `stash@{0}` и в патче
`agentrig-research/scratchpad/meta/wip/p089-unchecked-deserialize.patch`. Ветки `feature/secret-debug`,
`feature/init-probe` и прежние сохранены. Установленный рантайм — `b4920cf`. macOS: коммиты, слияние и вызовы с cargo
через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P090: `project_setup/clippy.rs` с профилем из 23 линтов; `template.rs` создаёт Rust-проекту команду и проверку
  `clippy`; мастер `init --interactive` предлагает созданные шаблоном проверки; тест `test_clippy_profile.py`; README;
  D132; Delivery и ротация P090, B082.
- Осталось: коммит VAC, пин D051 (`stable.txt` и `just bootstrap`), коммит пина, `just feature-merge`.

## Verification

- `pytest Project/project_setup`: 62 passed.
- `just candidate lint`: код 0.
- Проба AC1 в scratchpad: `check --only clippy` с `Some(1).unwrap()` — `ERROR [clippy]`, `clippy::unwrap_used`;
  полный `check` с `&text[..1]` — `ERROR [clippy]` после PASS остальных; после исправления `check` — код 0.

## Blockers

P089 ждёт выбора пользователя; P090 не блокирован.

## Next action

Коммит VAC P090 через хук, пин D051, `just feature-merge` для `feature/clippy-delivery`, ветку сохранить; затем
итоговый отчёт с вариантами для P089. Свободные ID: D133+, I022+, P091+, B083+ (D131 занят работой P089, B066 и D111
зарезервированы).
