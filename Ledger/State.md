# State

## Focus

Пользователь 2026-11-10 после оценки двух списков запретов для Rust задал стандарт: риг задаёт стандарт кода, код
на 20 лет, AI-first и agent-native; проверка нужна, если повышает качество, даже когда в риге нет её конструкции;
объём рефакторинга не причина отказа; несоответствие ригу чинится; риг полностью настраивает любые проекты.
Стандарт закреплён в AGENTS.md и D151. Параллельно идёт приёмка P022 voxel-rust на ноутбуке с Linux и NVIDIA.

## Workspace

Branch: `feature/quality-standard`, от master.
Revision: `1fd81e7bdaa7954097baf943c9d556a97376819a`; этот снимок закоммичен поверх.

Установленный рантайм `6c13aa2` (пин P109): `.agentrig/bin/agentrig` запускает `bin/macos-arm64/agentrig`. macOS:
коммиты и `feature-merge` с cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.

## Progress

- P094–P109 завершены и слиты в master; Plan и Backlog пусты.
- D151 и раздел «Quality standard» в AGENTS.md записаны: D151 снимает с отказов D127, D132 и D137 основания «нет
  целевого кода», «init не переписывает конфигурации» и «init владел бы clippy.toml»; выбранное поведение этих
  решений действует до решений-преемников. Новые правила по спискам ещё не предложены карточками Backlog.
- voxel-rust P022 слит в main `20f2c74` с отметкой непроверенного и paused. wgpu с vulkan и metal, решение D010
  voxel-rust; сценарии Web требуют адаптер без isFallbackAdapter, нативные требуют тип устройства не Cpu.
- Приёмка на ноутбуке на `20f2c74`: canary smoke-nvidia и texture-smoke-nvidia PASS; нативные 6 из 8 PASS, в том
  числе smoke на AMD Radeon 890M; браузерные 12 из 15 PASS. Падают native-ui, native-settings, web world-smoke,
  streaming-smoke и storage/web-smoke. Отрицательные: smoke-lavapipe упал по таймауту, а не на проверке аппаратного
  адаптера, вывод не сделан; texture-smoke с программным GPU упал на строгом равенстве, значения не сверены.
- На ноутбуке идёт повтор пяти упавших сценариев на базовой ревизии `377aaa7` в отдельном worktree.

## Verification

- AGENTS.md: 70 непустых строк, ни одной шире 120 символов. Остальное проверяет гейт коммита этой ветки.
- P109: `test_runtime.py` 4 passed, гейт коммита и `feature-merge` 239 passed.
- voxel-rust P022: гейт `feature-merge` прошёл lint, memory, tests, clippy, build-checks, native-tests и tools. На Mac
  smoke, native-input и texture-smoke PASS через Metal, ui-smoke падает на захвате указателя. Android input-smoke и
  lifecycle PASS через Vulkan, экран телефона возвращён к плотности 560.

## Blockers

Не приняты в voxel-rust: native-ui и native-settings на ноутбуке, три сценария Web на ноутбуке, ui-smoke на Mac;
отрицательная проверка lavapipe не доказана.

## Next action

Сравнить результаты `377aaa7` с `20f2c74`; при падении Web и на базе повторить на первом родителе `377aaa7`.
Записать итог в Plan/022.md voxel-rust на новой ветке, убрать клон, бандл и worktree с ноутбука и локальный
файл askpass. Показать
в чате черновики Backlog по спискам запретов под D151. Свободные ID: D152+, I022+, P110+, B102+ (B066 и D111
зарезервированы).
