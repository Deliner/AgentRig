# State

## Focus

Пользователь 2026-11-07 поручил довести voxel-rust до готовности в AgentRig («Исправь все находки, мне нужно чтобы этот
проект был готов в agentrig»). 2026-11-09 он поручил «реализуй карточки все и продолжи с voxel. диск уже почищен»:
P091 завершён и слит; B085 сохранена в показанной в чате версии и выполняется как **P093**, решение D138. Затем
voxel-rust без push.

## Workspace

Branch: `feature/coverage-include`
Revision: `b74548ab906d1655c4a60bb4110628f2aa6a70e7` (VAC A поверх master `d7d7dc7`)

Ветки `feature/rust-expansion`, `feature/option-option`, `feature/init-probe` и прежние сохранены.
Установленный рантайм — `f31c6ea`. macOS: вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.
voxel-rust: ветка `feature/agentrig-migration`, коммит `52dac3e` поверх main `716c099`; stash `abi-crate-name-paths`
с путями `quarterworld::`. На Mac установлены wasm32, aarch64-linux-android, nightly-2026-09-08, wasm-bindgen 0.2.128,
cargo-apk, openjdk@21, Android SDK 36, build-tools 36.0.0 и NDK 29.0.14206865 в ~/Android/Sdk.

## Progress

- VAC A: карточка B085, выбор P093, сопоставление include проверки в `included` из `change_delivery/gate/mod.rs`,
  которым пользуются `skip_reason` и `project_setup/coverage.rs`, три случая в `test_coverage.py`, README гейта, D138.
- VAC B: ротация P093 и B085 в Archive с Delivery по AC1–AC3 — в рабочем дереве.
- Не сделано: коммит VAC B, пин D051, `just feature-merge`.
- Пользователь 2026-11-09 напомнил: voxel-rust проверяет работу AgentRig, риг под voxel не переделывается и остаётся
  переносимым для любых Rust-проектов. Находки voxel правятся в voxel; дефекты AgentRig идут в отчёт как наблюдения.

## Verification

- Гейт коммита `b74548a`: все проверки PASS, rust-test 11 и test 199 passed.
- `just run test -- Project/project_setup/tests/test_coverage.py Project/change_delivery/gate/tests`: 48 passed на
  итоговом коде. Без фильтра по include два новых случая вне include падают.
- Кандидатный `config-check` на voxel-rust `52dac3e` с итоговым кодом: ошибок нет, конфигурация валидна; до правки
  было 4 ошибки.

## Blockers

Нет.

## Next action

Закоммитить VAC B, пин D051, `just feature-merge`; затем скопировать установленный рантайм в voxel-rust и продолжить
там.
Свободные ID: D139+, I022+, P094+, B086+ (B066 и D111 зарезервированы).
