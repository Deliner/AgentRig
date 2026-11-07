# State

## Focus

Пользователь 2026-11-07 выбрал для P089 разбиение `Kind` на семейные перечисления («Ок, принимаем 1 пункт. Делай все
оставшиеся карточки»). Выбрано **P087 → P090**; P087, P088 и P090 слиты в master, последний merge `5f02cdc`.
**P089** (B080, `unchecked-deserialize`) выполнен по AC1–AC4 и перенесён в архив, см.
[P089](Archive/Plan/089.md). Текущий шаг: финальный пин D051 и слияние ветки.

## Workspace

Branch: `feature/unchecked-deserialize`
Revision: `95ee9dc779582ba5c0584dc1e3c5b38df014bb7d` (фаза 2; коммит финального пина следует)

Ветки `feature/clippy-delivery`, `feature/init-probe` и прежние сохранены. Установленный рантайм — `95ee9dc`. macOS:
коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- VAC A `fcc4f1e` (D133), VAC B `84aba7f` (D131), VAC C `2166104` (D134) и фаза 2 `95ee9dc` (правило, навык,
  поставка `init`, D135, ротация P089 и B080) закоммичены с гейтом PASS; мид-пины `ec9eebe` и `fd08725`.
- Финальный пин: `stable.txt` — `95ee9dc`, `just bootstrap` код 0.
- Осталось: коммит пина и `just feature-merge` с сохранением ветки.

## Verification

- Гейт `95ee9dc`: все проверки PASS, rust-test 164 и test 342 passed.
- Установленный `95ee9dc`: `lint-rule unchecked-deserialize` выводит метрику с `let`.
- Проба в scratchpad: до ремонта `ERROR [unchecked-deserialize]` на `src/lib.rs:20 (Port::try_from)` и
  `src/lib.rs:4 (Interval::new)`, `Span` и `Label` без находок; после ремонта через `serde(try_from)` — код 0.

## Blockers

Нет.

## Next action

Закоммитить финальный пин, затем `just feature-merge`.
Свободные ID: D136+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
