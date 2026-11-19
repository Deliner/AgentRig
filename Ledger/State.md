# State

## Focus

P145 по B146: команды для чтения работают в песочнице без записи. VAC 1: механизм D193, тесты, решение и описание.

## Workspace

Branch: `feature/read-only-sandbox`, от master `de8cb13`.
Revision: `de8cb13bc21084a4fc40bb7a1625d4ce10772af1`

## Progress

- VAC 1, этот коммит: если каталог журнала создать нельзя, команда для чтения пробует временный файл в своём рабочем
  каталоге и при отказе по правам или по файловой системе только для чтения идёт без песочницы рига и журнала (D193).
  Иначе прежняя ошибка. Тесты `read_commands_run_inside_an_environment_that_refuses_writes` и
  `read_commands_without_a_run_log_in_a_writable_directory_are_refused`, описание в `Project/README.md`.

## Verification

- `just run test -- command_runner::tests::commands::` прошёл, 8 тестов; без исправления первый тест падает с ошибкой
  каталога журнала.
- `just lint` и `just check --only memory` проходят.
- Codex 0.170 с `-s read-only` и `gpt-6.1-sol` в пустом проекте после `init` кандидатом: `just read`, `just lint`,
  `just config-check` и `just resume` вернули 0 с выводом, `just write` вернул 127 и файл не создал.

## Blockers

Нет.

## Next action

Коммит VAC 1 через гейт. Затем пин по D051, перенос P145 и B146 в архив и `just feature-merge`.
