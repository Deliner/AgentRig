# State

## Focus

P173 по B180, VAC 2 закрывает AC1: правка lint.yaml, которая только добавляет правила, коммитится без подтверждения.

## Workspace

Branch: feature/standing-consent
Revision: f7b74972877010f85de372f07b10ef4ac39e249e

Ветка идёт от последнего коммита P170 и сливает P170 вместе с P173.

## Progress

- Хук агента больше не запрещает правку lint.yaml; `Protection::closed_to_agents` исключает только его.
- Гейт коммита принимает lint.yaml без подтверждения, когда все прежние правила стоят без изменений и по порядку, а
  остальные настройки равны: `structural_lint::lint_files::only_adds_rules`. Каталог `baseline_file` переименован
  в `lint_files`: он уже читал базовую линию через strict_yaml, а в lint_config или в корне проверка превышала
  бюджеты зависимых strict_yaml, 10 при 9, и structural_lint, 7 при 6.
- Решение D226 записано на весь выбор B180; навык propose-protected-change говорит, что правило добавляют правкой.

## Verification

- `just test change_delivery::git::tests::approval`: 6 passed, вместе с новым тестом добавленного, удалённого и
  выключенного правила.
- `just test agent_hooks::tests::protected`: 6 passed.

## Blockers

Нет. Карта тестов в checks.yaml не связывает `Project/structural_lint/rule_set/lint_config/**` с тестами
`change_delivery::git::`; правка карты войдёт в предложение к подтверждению в конце P173.

## Next action

VAC 3: команды `consent`, `revoke-consent`, `consented-pins` в рантайме и `propose-pin` под согласием (AC2, AC3).
