# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. По нему показаны карточки B102–B105, и
пользователь поручил: «Все карточки подтверждаю, заводи и делай автономно». Выбрано P110 → P113. Параллельно идёт
приёмка P022 voxel-rust на ноутбуке с Linux, NVIDIA и Radeon.

## Workspace

Branch: `feature/clippy-config`, от master `7990589`.
Revision: `3b88a613a0c541b1e89cb9ca72803f4256c5451e`; этот снимок закоммичен поверх вместе с пином.

Установленный рантайм `3b88a61` (пин P112): `.agentrig/bin/agentrig` запускает `bin/macos-arm64/agentrig`. macOS:
коммиты и `feature-merge` с cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.

## Progress

- P110 и P111 слиты в master; последний merge `7990589`, пин P111 `06e6137`.
- P112 выполнена VAC `3b88a61`: `Project/clippy.toml` с `avoid-breaking-exported-api = false` и
  `max-struct-bools = 1`, семь структур с двумя `bool` и `Reference::Comment` переведены на enum, `Scope` в выборе
  гейта, `init` дополняет файл Clippy проекта, D154, P112 и B104 в архиве. Пин D051 и `just bootstrap` сделаны.
- P113 ждёт.
- voxel-rust P022 слит в main `20f2c74` и paused. Ветка voxel-rust `feature/p022-laptop` от `20f2c74`: коммит
  `621b999` переносит проверку аппаратного адаптера в начало нативного smoke.
- Ноутбук, `621b999`: smoke на lavapipe падает на проверке адаптера, с префиксом NVIDIA и без него проходит на
  RTX 5080. native-smoke, native-boundary, native-scene и native-profiles проходят в обоих режимах; native-ui прошёл
  один раз из трёх; native-settings устарел для экрана «Создание мира» из `4210251` и падает и на базе `377aaa7`.
- Radeon 890M через его ICD: smoke, native-smoke и native-profiles проходят; native-ui и native-scene превышают
  время с пустым журналом; native-boundary и native-settings падают на xdotool BadWindow, окно игры пропадает при
  вводе. Итог ещё не записан в voxel-rust.

## Verification

- P112 `3b88a61`: хук коммита — build, lint, memory, rustfmt, clippy, rust-test 171, test 423, ruff, mypy, typos
  и vulture PASS. Пробы двух порогов роняют `just run clippy` с кодом 101. Живая проба `init` на потребителе со
  своим `clippy.toml`: Clippy 0 до и 101 после. Рантайм `3b88a61` читает `checks.json` прежнего вида.
- voxel-rust `621b999`: хук коммита прошёл, в том числе native-tests 160.

## Blockers

Нет для P112–P113. Для P022 voxel-rust: native-settings устарел относительно экрана создания мира; это вне P022.

## Next action

`feature-merge` ветки P112, затем P113 на новой ветке `feature/structural-bans`.
Записать итоги ноутбука в Plan/022.md voxel-rust на ветке `feature/p022-laptop` и убрать клон, бандлы и worktree с
ноутбука и локальный файл askpass. Свободные ID: D155+, I022+, P114+, B106+ (B066 и D111 зарезервированы).
