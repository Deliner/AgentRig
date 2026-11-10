# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. По нему показаны карточки B102–B105, и
пользователь поручил: «Все карточки подтверждаю, заводи и делай автономно». Выбрано P110 → P113. Параллельно идёт
приёмка P022 voxel-rust на ноутбуке с Linux и NVIDIA.

## Workspace

Branch: `feature/clippy-zero-hit`, от master.
Revision: `44d1380f8135888420655e7b8c4ad4ebdeff3f06`; этот снимок закоммичен поверх.

Установленный рантайм `6c13aa2` (пин P109): `.agentrig/bin/agentrig` запускает `bin/macos-arm64/agentrig`. macOS:
коммиты и `feature-merge` с cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.

## Progress

- D151 и раздел «Quality standard» в AGENTS.md слиты в master `44d1380`.
- Карточки B102–B105 сохранены, P110 активна, P111–P113 ждут. VAC выбора идёт первым на ветке P110.
- voxel-rust P022 слит в main `20f2c74` и paused. Ветка voxel-rust `feature/p022-laptop` от `20f2c74`: коммит
  `621b999` переносит проверку аппаратного адаптера в начало нативного smoke.
- Ноутбук, `20f2c74` против базы `377aaa7`: native-settings и три сценария Web падают на обеих с тем же текстом, то
  есть не из-за P022. native-settings кликает по старому экрану, а приложение сначала показывает «Создание мира».
  native-ui на базе прошёл, на `20f2c74` упал на проверке неверного создания мира. texture-smoke с программным GPU
  падает на проверке fallbackAdapter, как требует A2.
- На ноутбуке идёт прогон ветки `p022-laptop`: smoke на lavapipe, NVIDIA и адаптере по умолчанию и шесть нативных
  сценариев node с префиксом NVIDIA и без него.

## Verification

- master `44d1380`: гейт коммита D151 и `feature-merge` прошли.
- voxel-rust `621b999`: хук коммита прошёл, в том числе native-tests 160.

## Blockers

Нет для P110–P113. Для P022 voxel-rust: native-settings устарел относительно экрана создания мира; это вне P022.

## Next action

Закоммитить выбор P110–P113 и выполнить P110. По завершении прогона на ноутбуке записать итог в Plan/022.md
voxel-rust на ветке `feature/p022-laptop` и убрать клон, бандлы и worktree с ноутбука и локальный файл askpass.
Свободные ID: D152+, I022+, P114+, B106+ (B066 и D111 зарезервированы).
