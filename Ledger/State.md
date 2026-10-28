# State

## Focus

На 2026-10-28 идёт доставка трёх согласованных исходов аудита докфудинга на ветке
feature/dogfood-init: [B019](Backlog/019.md)/[P028](Plan/028.md) — правило
`architecture` и маршрут структуры в поставке `init`; [B020](Backlog/020.md)/
[P029](Plan/029.md) — три markdown-правила и навык `condense-markdown` в поставке;
[B021](Backlog/021.md)/[P030](Plan/030.md) — статический страж согласованности
поставки и живой конфигурации. Порядок P028 → P029 → P030, P030 зависит от P028 и
P029.

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

VAC планирования закоммичен как 561f1ae. P028 реализован и помечен complete:
правило `architecture` и маршрут структуры отданы в шаблон `init`
(Project/project_setup/lint.rs, template.rs); добавлены тесты K1
(Project/project_setup/tests/test_lint_defaults.py) и K3 (structure_route в
Project/shared/testing/test_delivery.py). Тесты движка хуков адаптированы к
маршруту структуры на routes[0] (Project/agent_hooks/tests/requirements.rs,
mod.rs): убран лишний ручной префикс маршрута, сдвинуты индексы Source, строка
конфликта порядка и нейтральные пути проверки. Изменения ещё не закоммичены.
P029 и P030 не начаты.

## Verification

На кандидате прошли pytest test_lint_defaults.py (9 passed, включая новый K1) и
test_delivery.py (3 passed, python и rust — K2 и K3), а также rust-тесты
agent_hooks (11 passed) после адаптации к маршруту структуры. Полный гейт P028 —
на предстоящем коммите; memory-check в гейте этого VAC.

## Blockers

Нет.

## Next action

Закоммитить VAC P028 (четыре исходника + Plan/028.md, Plan.md, State.md) через
hook. Затем начать P029: отдать markdown-size/width/placement и навык
`condense-markdown` в поставку (Project/project_setup/lint.rs, assets.rs),
проверив, что потребитель держит те же markdown-рамки, что репозиторий.
