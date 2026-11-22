# State

## Focus

P198 (B205): риг без режима ratchet. VAC 1 выбирает B205 как P198 в Plan.

## Workspace

Branch: feature/lint-ratchet-removal
Revision: 0ca33584fc6ec76111ed0de2e7bce77311cb510c

## Progress

VAC 1: строка P198 в Ledger/Plan.md и карточка Ledger/Plan/198.md со ссылкой на B205 на ревизии `51e0f98`.

## Verification

`just check --only memory` перед коммитом.

## Blockers

Нет.

## Next action

Закоммитить VAC 1, затем удалить команду `lint-baseline`, чтение baseline, защиту и предложение в `init`.
