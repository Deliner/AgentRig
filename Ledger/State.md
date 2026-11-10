# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109, выполнены и
слиты. Риг проверялся на voxel-rust и под него не подгонялся. Затем пользователь поручил: «тебе надо Linux VM
поднять и на ней слияние сделать», а также приёмку GPU, Web и Android и подготовку Mac; это сделано. Последнее
поручение: «Найди корректное архитектурное решение на долгосрок и примени его» про Vulkan, RHI и texture-smoke;
выполнено в voxel-rust как P022 по одобренной карточке B001, AgentRig не менялся.

## Workspace

Branch: `feature/voxel-portable-gpu-state`, от master.
Revision: `10676f148de3a3853d5d0137910091cd75a141a3`; этот снимок закоммичен поверх.

Установленный рантайм `6c13aa2` (пин P109): `.agentrig/bin/agentrig` запускает `bin/macos-arm64/agentrig`. macOS:
коммиты и `feature-merge` с cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать. Клоны `voxel-b101`, `voxel-orig` и `voxel-p109`
в scratchpad одноразовые, коммитов в них нет.

## Progress

- P094–P109 завершены и слиты в master; Plan и Backlog пусты.
- Финальный проход voxel-rust на ветке `feature/agentrig-migration` закоммичен через её хуки рантаймом `6c13aa2`:
  `dee3088` — скрипт запуска и рантайм macOS arm64 через `runtime-add`, tools `bin/**`, паники в телах макросов
  `exports!` и `records!`; `0eee45e` — 13 тестовых модулей в `tests/`, правило test-placement, бюджет
  `src/core/terrain` 7, одна ссылка D008 и D009; `787d89b` — `pub(in crate::native)` в host, цели без `host::`,
  исключения lock-файлов из шаблона. `ABI_REVISION` voxel-rust стал 518149772, core.ts перегенерирован.
- В voxel-rust `#[allow(clippy::expect_used)]` не заменён на `#[expect]`: AgentRig распознаёт оба.
- voxel-rust слит в main по D150 из Linux x86_64: в контейнере podman `voxel-linux` с Ubuntu 24.04 под эмуляцией
  Rosetta собран AgentRig `6c13aa2`, `runtime-add` поставил рантайм linux-x86_64, коммит `76b09ab` и слияние
  `377aaa7` прошли через хуки. Результат перенесён на Mac бандлом перемоткой без слияния; Mac voxel-rust на main
  `377aaa7`. Ничего не отправлено в origin.
- Mac подготовлен: `JAVA_HOME` openjdk@21 в ~/.zshenv, ripgrep, platform-tools в ~/Android/Sdk, Rosetta.
- voxel-rust P022 слит в main `20f2c74` через `feature-merge`, ветка `feature/portable-gpu` сохранена. wgpu остаётся
  слоем над графическими API, у него включены vulkan и metal, решение D010 voxel-rust: нативный клиент работает на
  macOS через Metal. Сценарии Web требуют адаптер без isFallbackAdapter, нативные сценарии требуют тип устройства не
  Cpu вместо NVIDIA. P022 в voxel-rust paused: по решению пользователя слито с отметкой непроверенного.

## Verification

- P109: `test_runtime.py` 4 passed, гейт коммита и `feature-merge` 239 passed.
- voxel-rust `dee3088`, `0eee45e` и `787d89b`: в каждом коммите хук прошёл lint, memory, tests, clippy,
  build-checks и native-tests; у двух последних tests 132 и native-tests 160.
- Linux x86_64 под эмуляцией, не на настоящем железе: гейт `feature-merge` voxel-rust прошёл lint, memory, tests 132,
  clippy, build-checks, native-tests 160 и tools; хук старого рантайма main при переключении не помешал. После
  слияния `doctor` и `check --only tools` чистые и на Linux, и на Mac.
- Android voxel-rust на телефоне SM-S918B с геометрией SM-A336B через `wm size` и `wm density`: android-touch,
  input-smoke и lifecycle PASS; экран телефона возвращён к исходному размеру и плотности 560.
- Web voxel-rust: на Mac через Metal precision-smoke и lod/journey-smoke PASS; smoke, world-smoke, mining-smoke,
  streaming-smoke и storage/web-smoke падают на main в тех же строках, в контейнере с программным GPU smoke
  одинаково падает на main и на ветке.
- voxel-rust P022: гейт `feature-merge` прошёл lint, memory, tests, clippy, build-checks, native-tests и tools. На Mac
  smoke и пример native-input PASS через Metal, texture-smoke PASS с разбросом 0, ещё семь сценариев Web PASS,
  ui-smoke падает на захвате указателя, с `SOFTWARE_GPU=1` texture-smoke падает на проверке адаптера. Android
  input-smoke и lifecycle PASS через Vulkan, экран возвращён к плотности 560. В контейнере Linux `native-check` и
  `clippy` PASS.

## Blockers

Не приняты: нативный клиент и пять нативных сценариев node voxel-rust на опорном ноутбуке с Linux и NVIDIA, пять
сценариев Web, которые не проходят ни на main, ни на ветке в доступных средах, и ui-smoke на Mac.

## Next action

Ждать пользователя: приёмка непринятого из Blockers на опорном ноутбуке по условию возобновления в Plan/022.md
voxel-rust. Свободные ID: D151+, I022+, P110+, B102+ (B066
и D111 зарезервированы).
