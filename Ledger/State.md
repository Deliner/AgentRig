# State

## Focus

Пользователь 2026-11-07 поручил довести voxel-rust до готовности в AgentRig («Исправь все находки, мне нужно чтобы этот
проект был готов в agentrig») и согласовал B083 («Сохранить и выполнить»). Выбран **P091** (B083, анализатор Rust:
`include!`, `macro_rules!`, glob-импорты, атрибуты, корни и внешние крейты `init`), решение D136. По поручению
«Подготовь черновик и реализуй его перед voxel» P092 (B084, `clippy::option_option`, D137) выполнен и слит в master
`02071e7`. После P091 — миграция voxel-rust на текущий `init` и исправление его находок в самом voxel-rust, без push.

## Workspace

Branch: `feature/rust-expansion`
Revision: `02071e7b49a7f932066391317246a0794b5ead09` (master после слияния P092; VAC A перебазирован поверх)

Ветки `feature/option-option`, `feature/unchecked-deserialize`, `feature/init-probe` и прежние сохранены.
Установленный рантайм — `eef6cd4`. macOS: вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- VAC A: анализатор и `init` по D136, юнит-тесты извлечения и резолвера, CLI-тесты include, макросов, атрибутов и
  `init`, README и `describe()`, D136, карточка B083, P091 в Plan.
- Не сделано: AC5 на voxel-rust после исправления путей ABI в voxel, финальный пин D051, ротация P091 и B083,
  `just feature-merge`.

## Verification

- VAC A до перебазирования, `12ccecc` поверх `ef216b2`: гейт PASS, rust-test 141 и test 309 passed.
- Проба voxel-rust кандидатом с конфигурацией `init` до исправлений в voxel: незавершённый анализ Rust остался только
  на путях ABI в крейте generate.

## Blockers

Нет.

## Next action

Ветка voxel-rust: миграция `init`, пути ABI, проверка AC5.
Свободные ID: D138+, I022+, P093+, B085+ (B066 и D111 зарезервированы).
