# State

## Focus

P173 по B180, финальный VAC: пин d725979 по согласию, тест AC1 с поднятым порогом, архив P173 и B180.

## Workspace

Branch: feature/standing-consent
Revision: d725979186f6c6293affe96b2106d288089fd2ce

Ветка идёт от последнего коммита P170 и сливает P170 вместе с P173.

## Progress

- Пользователь дал согласие на P173, P171 и P172; `just propose-pin HEAD` сам поставил пин d725979 после гейта ветки.
- Тест `a_lint_configuration_commits_without_approval_only_when_it_adds_rules` проверяет и поднятый порог, и текст
  отказа `has not approved`, как называет AC1.
- P173 и B180 в архиве с Delivery; ссылки D226 ведут в архив.

## Verification

- `just propose-pin HEAD`: гейт ветки, 605 тестов; `just consented-pins` показал d725979 с 6 файлами.
- `just test change_delivery::git::tests::approval::a_lint_configuration`: 1 passed. `just check --only memory`: PASS.

## Blockers

Нет.

## Next action

Пин этого коммита по согласию, коммит пина, `just feature-merge`; затем P171 и P172 одной веткой от master.
