# State

## Focus

На 2026-10-28, по итогам аудита докфудинга, пользователь согласовал и поручил
реализовать три исхода: [B019](Backlog/019.md) (правило `architecture` и маршрут
структуры в поставке `init`), [B020](Backlog/020.md) (три markdown-правила и навык
`condense-markdown` в поставке) и [B021](Backlog/021.md) (статический страж
согласованности поставки `init` и живой конфигурации репозитория). Выбраны в
[Plan](Plan.md) как P028 → P029 → P030; порядок по зависимостям, P030 зависит от
P028 и P029.

## Workspace

Branch: feature/dogfood-init

Revision: 332292b65854149a1f0ac8ae4e987522eb756ff0

Машина — macOS (Darwin 25.5.0). Ветка отходит от master. Шаблон потребителя живёт
в Project/project_setup (lint.rs фильтрует виды правил; template.rs строит checks,
routes, commands; assets.rs перечисляет навыки). Живая конфигурация — в
.agentrig/*.yaml и навыки в Project/shared/skills. Сборки и коммиты идут через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:...; ...'`, так как агентские
оболочки не имеют cargo/uv в PATH.

## Progress

VAC планирования: сохранены карточки B019/B020/B021 и P028/P029/P030, обновлены
индексы Backlog и Plan, обновлён State. Реализация ещё не начата.

## Verification

Пока только планирование памяти; memory-check запускается в гейте этого VAC.

## Blockers

Нет. memory-check не запускается внутри `just read`.

## Next action

Закоммитить VAC планирования, затем начать P028: отдать правило `architecture` и
маршрут структуры в шаблоне `init` (Project/project_setup/lint.rs и template.rs),
проверив, что пустой проект после `init` проходит `lint`.
