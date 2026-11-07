# State

## Focus

Пользователь 2026-11-07: «сохраняй все карточки, заведи отдельную карточку на клипе, и после этого автономно
реализуй все это». Выбрано **P087 → P090**. P087 и P088 слиты в master, последний merge `21c0b30`. **P089**
(B080, `unchecked-deserialize`) приостановлен: нужен выбор пользователя о структуре реестра видов, см. Delivery
[P089](Plan/089.md). **P090** (B082, поставка clippy в `init`) выполнен на ветке, P090 и B082 в Archive.

## Workspace

Branch: `feature/clippy-delivery`
Revision: `5ba1aafcc4101e427678957bb18b19010339dc4f` (VAC P090; коммит пина следует)

Ветка `feature/unchecked-deserialize` без коммитов, работа P089 в `stash@{0}` и в патче
`agentrig-research/scratchpad/meta/wip/p089-unchecked-deserialize.patch`. Ветки `feature/secret-debug`,
`feature/init-probe` и прежние сохранены. Установленный рантайм — `5ba1aaf`. macOS: коммиты, слияние и вызовы с cargo
через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P090: `project_setup/clippy.rs` с профилем из 23 линтов; `template.rs` создаёт Rust-проекту команду и проверку
  `clippy`; мастер `init --interactive` предлагает созданные шаблоном проверки; тест `test_clippy_profile.py`; README;
  D132; Delivery и ротация P090, B082. VAC `5ba1aaf`.
- Пин D051: `stable.txt` — `5ba1aaf`, `just bootstrap` — код 0.
- Осталось: коммит пина, `just feature-merge`.

## Verification

- Гейт коммита `5ba1aaf`: все проверки PASS, rust-test 11 и test 206 passed.
- `pytest Project/project_setup`: 62 passed.
- `just candidate lint`: код 0.
- Проба AC1 в scratchpad: `check --only clippy` с `Some(1).unwrap()` — `ERROR [clippy]`, `clippy::unwrap_used`;
  полный `check` с `&text[..1]` — `ERROR [clippy]` после PASS остальных; после исправления `check` — код 0.

## Blockers

P089 ждёт выбора пользователя; P090 не блокирован.

## Next action

Коммит пина D051 через хук, `just feature-merge` для `feature/clippy-delivery`, ветку сохранить; затем
итоговый отчёт с вариантами для P089. Свободные ID: D133+, I022+, P091+, B083+ (D131 занят работой P089, B066 и D111
зарезервированы).
