# State

## Focus

Пользователь 2026-11-07 поручил довести voxel-rust до готовности в AgentRig и затем подготовить и реализовать до работ
в voxel карточку о запрете `Option<Option<_>>` («Подготовь черновик и реализуй его перед voxel»). Выбран **P092**
(B084, `clippy::option_option` в общем профиле Clippy). P091 (B083, анализатор Rust, D136) ждёт на ветке
`feature/rust-expansion` проверки AC5 на voxel-rust.

## Workspace

Branch: `feature/option-option`
Revision: `ef216b2bd9c72dd5abdd073623f6998290075cc8` (база master; первый VAC — карточка B084 и P092)

Ветка `feature/rust-expansion` с VAC A `12ccecc` сохранена, как и `feature/init-probe` и прежние. Установленный
рантайм — `95ee9dc`. macOS: вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- Карточка B084 сохранена, P092 выбран.
- Не сделано: линт в `clippy.rs` и `.agentrig/commands.yaml`, проба фрагментом, D137, пин D051, ротация P092 и B084,
  `just feature-merge`; затем возврат к P091 и voxel-rust.

## Verification

- Проба clippy 1.98.1 в scratchpad: `-D clippy::option_option` ловит закрытые поле, вариант, параметр, возвращаемый
  тип и `Vec<Option<Option<u8>>>`; открытый API — только с `avoid-breaking-exported-api = false`; `type` и `let` —
  нет.
- `git grep` по master: `Option<Option` в Project и tooling не найден.

## Blockers

Нет.

## Next action

Добавить `option_option` в профиль, доказать пробным фрагментом, записать D137 и пройти гейт.
Свободные ID: D137+, I022+, P093+, B085+ (B066 и D111 зарезервированы; D136, P091 и B083 заняты на ветке
`feature/rust-expansion`).
