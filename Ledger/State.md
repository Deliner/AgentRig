# State

## Focus

Пользователь 2026-11-07 выбрал для P089 разбиение `Kind` на семейные перечисления («Ок, принимаем 1 пункт. Делай все
оставшиеся карточки»). Выбрано **P087 → P090**; P087, P088 и P090 слиты в master, последний merge `5f02cdc`.
**P089** (B080, `unchecked-deserialize`) выполнен по AC1–AC4 и перенесён в архив, см.
[P089](Archive/Plan/089.md). Текущий VAC: фаза 2 с правилом, навыком, поставкой `init`, [D135](Decisions/135.md) и
ротацией.

## Workspace

Branch: `feature/unchecked-deserialize`
Revision: `fd08725f355519551bb9f2b9f9f9da2d3a12c891` (второй мид-пин; фаза 2 в рабочем дереве до коммита)

Ветки `feature/clippy-delivery`, `feature/init-probe` и прежние сохранены. Установленный рантайм — `2166104`. macOS:
коммиты, слияние и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- VAC A `fcc4f1e` (D133), VAC B `84aba7f` (D131) и VAC C `2166104` (D134) закоммичены с гейтом PASS; мид-пины
  `ec9eebe` и `fd08725`.
- Фаза 2 в рабочем дереве: правило уровня error в `.agentrig/lint.yaml`, `catalog::ALL`, обработчик Rust в реестре,
  навык `signature/check-deserialize` с symlink и `architecture.yaml`, `assets.rs`, список решённых исключений в
  `test_config_check.py`.
- Проба `init --language rust` показала: при условиях в стиле `named-if-condition` детектор молчал. Детектор теперь
  учитывает локальную переменную из `let`, выведенную из параметра; тест, метрика, навык и D135 добавлены.
- Ротация: P089 в `Archive/Plan`, B080 в `Archive/Backlog`, ссылки карточки в D131, D133, D134 и D135 обновлены.

## Verification

- `cargo test derived_trait`: 9 passed. Кандидат: `lint` на AgentRig код 0; `config-check` и `lint-config-check`
  проходят; `memory-check` после ротации код 0; `just run clippy` без замечаний.
- Проба в scratchpad: до ремонта `ERROR [unchecked-deserialize]` на `src/lib.rs:20 (Port::try_from)` и
  `src/lib.rs:4 (Interval::new)`, `Span` и `Label` без находок; после ремонта через `serde(try_from)` — код 0.

## Blockers

Нет.

## Next action

Закоммитить фазу 2 через хук, вписать хеш и итог гейта в Delivery P089 и сделать
финальный пин D051, затем `just feature-merge`.
Свободные ID: D136+, I022+, P091+, B083+ (B066 и D111 зарезервированы).
