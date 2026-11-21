# State

## Focus

P179 по B186: код, который никто не вызывает, удалён. VAC 2: удалены функции без вызовов в продукте.

## Workspace

Branch: feature/unused-code
Revision: b1f082ef9dced9acd97c013c5cd5c0aa1c177833

VAC 1 `b1f082e` сохранил карточку и пункт плана. Изменены файлы в Project/shared/atomic_files, fs_with_context и vcs.

## Progress

Удалены `FileKind::mode`, `Repository::diff` с проверкой в тесте `revisions_are_read_exactly`, файл
`bounded_read.rs` с `read_regular`, его двумя тестами и записью в architecture.yaml, реэкспорт `OpenOptionsExt`.
Проверка `digest` из удалённого теста стала тестом `fingerprints_are_lowercase_sha256_hex_of_the_content`.

## Verification

- `just candidate --version` собирается без предупреждений.
- Пробная сборка с закрытыми модулями находит только реэкспорты типов `AffectedTests`, `Error` и `Observation`,
  которые вызывающий получает из сигнатур; функций без вызовов нет. Пробная правка lib.rs откачена.
- `just test shared::`: 32 прошли, отфильтровано 581, всего 613 вместо 614.
- `just lint` без находок.

## Blockers

Нет.

## Next action

Закоммитить VAC 2, затем архивировать P179 и B186 с Delivery и слить ветку через `just feature-merge`.
