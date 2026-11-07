# State

## Focus

Пользователь 2026-11-07 выбрал для P089 разбиение `Kind` на семейные перечисления («Ок, принимаем 1 пункт. Делай все
оставшиеся карточки»). Выбрано **P087 → P090**; P087, P088 и P090 слиты в master, последний merge `5f02cdc`.
**P089** (B080, `unchecked-deserialize`) активен, см. [P089](Plan/089.md). VAC A: разбиение `Kind` по
[D133](Decisions/133.md) без изменения поведения.

## Workspace

Branch: `feature/unchecked-deserialize`
Revision: `5f02cdcea46bb84ef3d404216aba36a6142e5e58` (master; VAC A не закоммичен)

Работа фазы 1 P089 лежит в `stash@{0}` этой ветки и в патче
`agentrig-research/scratchpad/meta/wip/p089-unchecked-deserialize.patch`. Ветки `feature/clippy-delivery`,
`feature/init-probe` и прежние сохранены. Установленный рантайм — `5ba1aaf`. macOS: коммиты, слияние и вызовы с cargo
через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- VAC A: `rules/{structure,control_flow,signature,ownership,reference}.rs` с перечислениями, `descriptor` и
  константами; `descriptor.rs` — форма и диспетчеризация; `kind.rs` — обёртка `#[serde(untagged)]`; разбор имени по
  `catalog::ALL` в `catalog/mod.rs`; потребители пишут `Kind::Family(Family::Variant)`. Тест Python except-pass
  перенесён в `tests/silent_error.rs`, повторяющиеся виды в двух тестовых файлах названы константами. Ссылки 36
  решений перенесены с `descriptor.rs` на файлы семейств, ссылка тестов D069 — на `tests/silent_error.rs`; D133;
  контракты `architecture.yaml`; P089 — active.
- Осталось: коммит VAC A; затем stash, вид `UncheckedDeserialize` в `Signature`, D131, мид-пин; затем фаза 2.

## Verification

- `cargo test`: 157 passed; `just run clippy`: код 0; `just candidate lint`: код 0.
- `just candidate memory-check`: только прежнее предупреждение D112 о `tree-sitter-rust/grammar.js`.
- `lint-rules` установленного рантайма и кандидата совпадают побайтно; `lint-rule --example` для трёх видов тоже.
  `lint-rule panic-pathx`: «unknown variant `panic-pathx`, expected one of …», код 2.

## Blockers

Нет.

## Next action

Закоммитить VAC A через хук. Затем `git stash pop`, привести `derived_trait.rs` к `Kind::Signature(...)`, добавить
`UncheckedDeserialize` с дескриптором и D131, коммит, мид-пин D051. Свободные ID: D134+, I022+, P091+, B083+ (D131
занят работой P089, B066 и D111 зарезервированы).
