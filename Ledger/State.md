# State

## Focus

P173 по B180, VAC 3 закрывает в рантайме AC2 и AC3: согласие на задачу и список пинов, поставленных по нему.

## Workspace

Branch: feature/standing-consent
Revision: 7e3302c318503cd286d1503b92c90c1e4a2643c1

Ветка идёт от последнего коммита P170 и сливает P170 вместе с P173.

## Progress

- `consent.rs`: `consent` записывает пункты Plan в статусе pending и active в `<runtime>/consent`, `revoke-consent`
  снимает их, `consented-pins` печатает каждый пин с изменёнными исходниками. Путь `consent` защищён.
- `propose-pin` при действующем согласии берёт только чистый `HEAD` без учёта State, проверяет потомка пина, гоняет
  гейт ветки `feature-merge`, ставит ревизию через `git.install`, пишет пин и вносит его в запись одобренного; без
  согласия готовит предложение для `just approve`, как раньше.
- Пин пишется через новую `atomic_files::replace`, которой теперь пользуется и `atomic_json::save`: каталог git
  не добавляет зависимого к fs_with_context с бюджетом 23.
- Рецептов и отказа хука для `consent` ещё нет: они идут в VAC 4 вместе с пином этой ревизии.

## Verification

- `just test change_delivery::git::`: 32 passed, с тремя новыми тестами согласия в `tests/pin.rs`.
- `just test change_delivery::git::tests::pin shared::atomic_files`: 11 passed после переноса записи пина.

## Blockers

Нет. Карта тестов в checks.yaml не связывает `Project/structural_lint/lint_files/**` и
`Project/project_memory/format.rs` с тестами `change_delivery::git::`; правка войдёт в предложение к подтверждению.

## Next action

`just propose-pin HEAD`, предложения delivery.just с тремя рецептами и checks.yaml, затем `! just approve` от
пользователя.
