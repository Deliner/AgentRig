# State

## Focus

На 2026-10-28 идёт доставка трёх согласованных исходов аудита докфудинга на ветке
feature/dogfood-init: [B019](Backlog/019.md)/[P028](Plan/028.md) — правило
`architecture` и маршрут структуры в поставке `init` (complete, коммит 40dbc30);
[B020](Backlog/020.md)/[P029](Plan/029.md) — три markdown-правила и навык
`condense-markdown` в поставке (complete, коммит на этом VAC);
[B021](Backlog/021.md)/[P030](Plan/030.md) — статический страж согласованности
поставки и живой конфигурации (pending). Порядок P028 → P029 → P030, P030 зависит
от P028 и P029.

## Workspace

Branch: feature/dogfood-init

Revision: 561f1aed61a2dd043f1468cd481bbb5184b6f9ec

Машина — macOS (Darwin 25.5.0). Шаблон потребителя — в Project/project_setup
(lint.rs строит правила, template.rs — checks/routes/commands, assets.rs — навыки).
Живая конфигурация репозитория — .agentrig/*.yaml и навыки Project/shared/skills;
`lint::template` формирует только поставку, не трогая собственный .agentrig.
Сборки и тесты идут через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:
/Users/admin/.local/bin:/opt/homebrew/bin:...; ...'`, кандидат собирается из
tooling/worker/build (фикстура `worker` в Project/conftest.py).

## Progress

P028 закоммичен как 40dbc30 (правило `architecture` и маршрут структуры в шаблоне
`init`, адаптация тестов движка хуков). P029 реализован и помечен complete: три
markdown-правила и helper `markdown` (Project/project_setup/lint.rs), навык
`condense-markdown` в поставке и перенос генерируемого AGENTS.md по словам
(Project/project_setup/assets.rs, `wrap`/`fold`), тест
`test_shipped_markdown_bounds` (Project/project_setup/tests/test_lint_defaults.py).
Изменения P029 ещё не закоммичены. P030 не начат.

## Verification

На кандидате прошли pytest test_lint_defaults.py (10 passed, включая
`test_shipped_markdown_bounds`), test_delivery.py (3 passed — потребительский
`check` с markdown-правилами и перенесённым AGENTS.md) и
test_hooks.py::test_ledger_edit_guidance (19 passed). P028 прошёл полный гейт при
коммите 40dbc30. Полный гейт P029 — на предстоящем коммите.

## Blockers

Нет.

## Next action

Закоммитить VAC P029 (lint.rs, assets.rs, test_lint_defaults.py + Plan/029.md,
Plan.md, State.md) через hook. Затем начать P030: статический страж
согласованности поставки `init` и живой конфигурации репозитория — манифест
universal/worker-local (D053), предикат-инвариант с оракулом-тестом (I020),
запускаемый в гейте.
