# State

## Focus

Пользователь 2026-11-07 поручил довести voxel-rust до готовности в AgentRig («Исправь все находки, мне нужно чтобы этот
проект был готов в agentrig»). 2026-11-09 он поручил «реализуй карточки все и продолжи с voxel. диск уже почищен»:
P091 завершён и слит; B085 сохранена в показанной в чате версии и выполняется как **P093**, решение D138. Затем
voxel-rust без push.

## Workspace

Branch: `feature/coverage-include`
Revision: `d7d7dc759ffca2a2ebacaf83a8383a5b852accca` (master после слияния P091)

Ветки `feature/rust-expansion`, `feature/option-option`, `feature/init-probe` и прежние сохранены.
Установленный рантайм — `f31c6ea`. macOS: вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.
voxel-rust: ветка `feature/agentrig-migration`, коммит `52dac3e` поверх main `716c099`; stash `abi-crate-name-paths`
с путями `quarterworld::`. На Mac установлены wasm32, aarch64-linux-android, nightly-2026-09-08, wasm-bindgen 0.2.128,
cargo-apk, openjdk@21, Android SDK 36, build-tools 36.0.0 и NDK 29.0.14206865 в ~/Android/Sdk.

## Progress

- VAC A: карточка B085, выбор P093, сопоставление include проверки в `included` из `change_delivery/gate/mod.rs`,
  которым пользуются `skip_reason` и `project_setup/coverage.rs`, три случая в `test_coverage.py`, README гейта, D138.
- Не сделано: ротация P093 и B085, пин D051, `just feature-merge`.

## Verification

- `just run test -- Project/project_setup/tests/test_coverage.py Project/change_delivery/gate/tests`: 48 passed на
  итоговом коде. Без фильтра по include два новых случая вне include падают.
- Кандидатный `config-check` на voxel-rust `52dac3e` с итоговым кодом: ошибок нет, конфигурация валидна; до правки
  было 4 ошибки.

## Blockers

Нет.

## Next action

Закоммитить VAC A, затем ротация P093, пин D051, `just feature-merge`; затем voxel-rust.
Свободные ID: D139+, I022+, P094+, B086+ (B066 и D111 зарезервированы).
