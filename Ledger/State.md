# State

## Focus

P180 по B187: регистрация хуков Git читается один раз на решение. VAC 2: одно чтение `core.hooksPath` на решение.

## Workspace

Branch: feature/hook-registration-read
Revision: 2dda69225fc770792cc3ceed645bcff19aabe703

VAC 1 `2dda692` сохранил карточку и пункт плана. Изменены Project/shared/vcs/hooks.rs и Project/project_setup/
initialization: no_overwrite_init.rs, tests/mod.rs, tests/architecture.yaml.

## Progress

`validate_registration` заменён на `compatible_registration`: он читает настройку один раз, отказывает при чужой
регистрации и возвращает текущую. `register_hooks` и проверка перед `init` берут значение из него; чтение
`hook_registration` закрыто внутри hooks.rs. Новый тест `init_refuses_an_existing_hook_registration_and_keeps_it`
закрепляет оба отказа с прежними текстами; на коде до правки он тоже проходит.

## Verification

- `just test project_setup::`: 59 прошли, всего тестов 614.
- `just candidate --version` без предупреждений, `just lint` без находок.
- Временный замер шага регистрации хуков в `init`, 6 запусков: до 25,0 мс в среднем, после 15,8 мс; откачен.

## Blockers

Нет.

## Next action

Закоммитить VAC 2, затем архивировать P180 и B187 с Delivery и слить ветку через `just feature-merge`.
