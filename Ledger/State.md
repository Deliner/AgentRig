# State

## Focus

P180 по B187: регистрация хуков Git читается один раз на решение. VAC 1: карточка B187 сохранена и выбрана как P180.

## Workspace

Branch: feature/hook-registration-read
Revision: 55081f88eabdc5638d798393663bccf9093ceb40

Ветка от master `55081f8`. Изменены Backlog.md, Backlog/187.md, Plan.md и Plan/180.md.

## Progress

Временный замер `init` в новой директории: около 570 мс, шаг регистрации хуков около 24 мс с тремя процессами Git.
Замер откачен, код ещё не менялся.

## Verification

`just check --only memory` перед коммитом этого VAC.

## Blockers

Нет.

## Next action

Закоммитить VAC 1, затем в Project/shared/vcs/hooks.rs читать `core.hooksPath` один раз в `register_hooks` и в
проверке перед `init`.
