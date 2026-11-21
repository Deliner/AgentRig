# State

## Focus

P179 по B186: код, который никто не вызывает, удалён. VAC 1: карточка B186 сохранена и выбрана как P179.

## Workspace

Branch: feature/unused-code
Revision: b03c5fd2e72284ad03c97d66af7f077039186cec

Ветка от master `b03c5fd`. Изменены Backlog.md, Backlog/186.md, Plan.md и Plan/179.md.

## Progress

Пробная сборка с закрытыми модулями назвала функции без вызовов в продукте: `FileKind::mode`, `Repository::diff` и
`atomic_files::read_regular`. Карточка и пункт плана записаны, код ещё не менялся.

## Verification

`just check --only memory` перед коммитом этого VAC.

## Blockers

Нет.

## Next action

Закоммитить VAC 1, затем удалить три функции, их тесты и проверку `diff` в тесте `revisions_are_read_exactly`.
