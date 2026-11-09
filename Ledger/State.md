# State

## Focus

Пользователь 2026-11-07 поручил довести voxel-rust до готовности в AgentRig («Исправь все находки, мне нужно чтобы этот
проект был готов в agentrig»). 2026-11-09 он поручил «реализуй карточки все и продолжи с voxel. диск уже почищен»:
завершить P091 (B083, D136), сохранить и выполнить B085 в показанной в чате версии, затем продолжить voxel-rust без
push.

## Workspace

Branch: `feature/rust-expansion`
Revision: `2c163ed670e29c41efb0350b027bbaf9cde263c2` (VAC A поверх master `02071e7`)

Ветки `feature/option-option`, `feature/unchecked-deserialize`, `feature/init-probe` и прежние сохранены.
Установленный рантайм — `eef6cd4`. macOS: вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.
voxel-rust: ветка `feature/agentrig-migration`, коммит `52dac3e` поверх main `716c099`; stash `abi-crate-name-paths`
с путями `quarterworld::`.

## Progress

- VAC A: анализатор и `init` по D136, тесты, D136, карточка B083, P091 в Plan.
- Ротация P091 и B083 в Archive с Delivery по AC1–AC5 — в рабочем дереве, VAC B.
- Не сделано: коммит VAC B, финальный пин D051, `just feature-merge`.

## Verification

- VAC A на `2c163ed`: `just run rust-test -- structural_lint::architecture` 52 passed, тесты включений, макросов,
  атрибутов и `init` 54 passed, `just run lint` код 0.
- AC5: базовая линия voxel без правки ABI содержит 135 находок architecture в src/schema/abi и одну в
  src/ui/touch/tests.rs; с правкой из stash кандидатный lint неполных находок Rust не даёт.

## Blockers

Нет.

## Next action

Закоммитить VAC B, пин D051, `just feature-merge`; затем B085 → P093 на новой ветке.
Свободные ID: D138+, I022+, P093+, B085+ (B066 и D111 зарезервированы).
