# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. По нему показаны карточки B102–B105, и
пользователь поручил: «Все карточки подтверждаю, заводи и делай автономно». Выбрано P110 → P113. Параллельно идёт
приёмка P022 voxel-rust на ноутбуке с Linux и NVIDIA.

## Workspace

Branch: `feature/clippy-revisit`, от master `48310aa`.
Revision: `e0eb185968b606b27fd8071e9c6c823c48f36f8e`; этот снимок закоммичен поверх вместе с пином.

Установленный рантайм `e0eb185` (пин P111): `.agentrig/bin/agentrig` запускает `bin/macos-arm64/agentrig`. macOS:
коммиты и `feature-merge` с cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.

## Progress

- P110 слита в master `48310aa` с пином `7128bc3`.
- P111 выполнена VAC `e0eb185`: профиль Clippy из 32 lint-ов, полный match, `TryFrom` и счётчики линтера в
  `usize`, `#[expect]` вместо `#[allow]`, D153, P111 и B103 в архиве. Пин D051 и `just bootstrap` сделаны. P112 и
  P113 ждут.
- Замер P112 с `max-struct-bools = 1` и `avoid-breaking-exported-api = false`: 7 мест `struct_excessive_bools` и
  одно `enum_variant_names` в `structural_lint/rules/reference.rs`.
- voxel-rust P022 слит в main `20f2c74` и paused. Ветка voxel-rust `feature/p022-laptop` от `20f2c74`: коммит
  `621b999` переносит проверку аппаратного адаптера в начало нативного smoke.
- Ноутбук, `621b999`: smoke на lavapipe падает на проверке адаптера, с префиксом NVIDIA и без него проходит на
  RTX 5080. native-smoke, native-boundary, native-scene и native-profiles проходят в обоих режимах; native-ui прошёл
  один раз из трёх; native-settings устарел для экрана «Создание мира» из `4210251` и падает и на базе `377aaa7`.
- На ноутбуке идёт прогон на встроенном Radeon через его ICD: smoke и native-smoke прошли, native-boundary упал на
  xdotool BadWindow, native-ui превысил 900 с.

## Verification

- P111 `e0eb185`: хук коммита — build, lint, memory, rustfmt, clippy, rust-test 171, test 423, ruff, mypy, typos
  и vulture PASS. Пробные фрагменты трёх новых lint-ов роняют `just run clippy` с кодом 101.
- voxel-rust `621b999`: хук коммита прошёл, в том числе native-tests 160.

## Blockers

Нет для P112–P113. Для P022 voxel-rust: native-settings устарел относительно экрана создания мира; это вне P022.

## Next action

`feature-merge` ветки P111, затем P112 на ветке `feature/clippy-config`. По завершении прогона
на Radeon записать итог в Plan/022.md voxel-rust на ветке `feature/p022-laptop` и убрать клон, бандлы и worktree с
ноутбука и локальный файл askpass. Свободные ID: D154+, I022+, P114+, B106+ (B066 и D111 зарезервированы).
