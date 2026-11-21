# State

## Focus

P194 по карточке B201: запись одобрения кладёт в базу объектов Git только файлы, где бывают исключения lint.
VAC 2: запись одобрения хеширует без записи файлы прочих видов, проверка исключений их не читает.

## Workspace

Branch: feature/hashed-approval
Revision: 2509a7d5385d964d864537c9d3c9671e2302e0a3

## Progress

`holds_exceptions` в source_parsers решает по тем же видам, что и поиск исключений. `Staging` в vcs делит пути на
записываемые и только хешируемые, `exceptions::stageable` делит изменённые пути, `added` читает только такие файлы.

## Verification

`just candidate lint`, `just run clippy` и rustfmt без находок. Тесты: change_delivery 46, project_setup 59,
structural_lint::languages 174, все проходят.

## Blockers

Нет.

## Next action

Проверить rustfmt и затронутые тесты, закоммитить VAC 2, затем замер до и после и архивация в VAC 3.
