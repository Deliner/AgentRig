# State

## Focus

P144 по B142: State описывает текущую VAC. VAC 4: правило AC1 в текстах для агентов и граница карточки о
долгосрочных заметках раздела Focus.

## Workspace

Branch: `feature/state-current-vac`, от master `339818e`.
Revision: `51db72811f9a1a07f90edb2b1937ecaeb80ceb4b`

Установленный рантайм `9a4f621` не знает `state-reset`: хук `post-commit` рига останавливается до `feature-push`, и
State после коммита не сбрасывается до закрепления P144.

## Progress

- VAC 4, этот коммит: AGENTS.md, текст AGENTS от `init` и навык edit-state говорят, что State описывает только
  текущую VAC, что каждый коммит на ветке несёт State своей VAC, и объясняют сброс, отказ пустого шаблона, коммит
  сброса в `feature-merge` и пределы lint. CLAUDE.md рига подключает AGENTS.md. Заметки Focus убраны: стандарт рига
  записан в AGENTS.md и D151, отказ от совместимости в D185, ожидание B107 и B112 в их карточках, выбор P144–P165 и
  пауза P156–P157 без ответа пользователя в Plan, запрет трогать voxel-rust в P147.

## Verification

Тесты `project_setup::` 54 из 54 прошли на кандидате; lint и проверка памяти прошли.

## Blockers

Нет.

## Next action

Закоммитить VAC 4. Затем последняя VAC P144: закрепить принятую голову в `stable.txt` с `just bootstrap`, перенести
P144 и B142 в Archive и выполнить `just feature-merge`.
