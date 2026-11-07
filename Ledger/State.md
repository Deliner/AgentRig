# State

## Focus

Пользователь 2026-11-07 выбрал для P089 разбиение `Kind` на семейные перечисления («Ок, принимаем 1 пункт. Делай все
оставшиеся карточки»). Выбрано **P087 → P090**; P087, P088 и P090 слиты в master, последний merge `5f02cdc`.
**P089** (B080, `unchecked-deserialize`) активен, см. [P089](Plan/089.md). VAC A, разбиение `Kind` по
[D133](Decisions/133.md), закоммичен. VAC B: фаза 1 нового вида, инертно.

## Workspace

Branch: `feature/unchecked-deserialize`
Revision: `fcc4f1ef017b4a83a20410b994875a26efc08a2c` (VAC A; VAC B не закоммичен)

`stash@{0}` применён к VAC B и ещё не удалён; удалить после коммита VAC B. Ветки `feature/clippy-delivery`,
`feature/init-probe` и прежние сохранены. Установленный рантайм — `5ba1aaf`. macOS: коммиты, слияние и вызовы с cargo
через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- VAC A `fcc4f1e`: `Kind` разбит на пять семейств по D133, ссылки решений перенесены; гейт коммита PASS.
- VAC B: работа из stash; `languages/rust/derived_trait.rs` вместо `secret_debug.rs` с детекторами `secret_debug`
  (D130) и `unchecked_deserialize` (D131); `type_name` и `plain_name` в `languages/rust/context`; восемь тестов в
  `languages/tests/derived_trait.rs`; вид `Signature::UncheckedDeserialize` с дескриптором в `rules/signature.rs`;
  D131 и строка индекса; ссылки D130 перенесены на `derived_trait.rs`. Вид не входит в `catalog::ALL` и в
  обработчик Rust: фаза 1 инертна.
- Осталось: коммит VAC B, мид-пин D051; затем фаза 2: правило в `.agentrig/lint.yaml`, `catalog::ALL`, реестр, навык
  `check-deserialize`, `assets.rs`, список решённых в `test_config_check.py`, проверка AC1–AC4, ротация, пин,
  `just feature-merge`.

## Verification

- `cargo test`: 163 passed; `just run clippy`: код 0; `just candidate lint`: код 0.
- `just candidate memory-check`: только прежнее предупреждение D112 о `tree-sitter-rust/grammar.js`.
- VAC A: `lint-rules` установленного рантайма и кандидата совпадают побайтно; `lint-rule panic-pathx` выдаёт
  «unknown variant `panic-pathx`, expected one of …», код 2.

## Blockers

Нет.

## Next action

Закоммитить VAC B через хук, `git stash drop`, мид-пин D051: HEAD в `stable.txt`, `just bootstrap`, коммит пина.
Свободные ID: D134+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
