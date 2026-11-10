# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109 и выполняются
автономно по одной за ветку; решения, которые карточки оставляли пользователю, исполнитель принимает сам и
записывает. Риг дорабатывается для себя через докфудинг и проверяется на voxel-rust; под voxel-rust он не
подгоняется.

## Workspace

Branch: `feature/platform-runtime` (P109, B086)
Revision: `13ea029` (слияние P108; VAC P109 в этом коммите)

Установленный рантайм `7bade55` (пин P107). macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; фокусные
тесты через `just run test` тоже требуют этого PATH для сборки кандидата.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.
Клон voxel-rust в scratchpad `voxel-b101` несёт незакоммиченный перенос 13 тестовых файлов в `tests/`, бюджет
`src/core/terrain` 7 и снятые вторые ссылки D008 и D009; копия переноса — `p104-voxel.patch` и
`p104-voxel-untracked.tgz` там же. Рабочее дерево `voxel-orig` в scratchpad — чистый `71440d0` того же клона.
Клон `voxel-p109` в scratchpad — одноразовая проверка P109 с локальной правкой tools, коммитов в нём нет.

## Progress

- P094–P108 завершены и слиты в master; Plan и Backlog после ротации P109 пусты.
- P109 (B086) в этом VAC: рантайм по платформам в `{service}/bin/PLATFORM/agentrig` за скриптом запуска
  `{service}/bin/agentrig`, команда `runtime-add`, шаблон tools `bin/**`, решение D150, ротация P109 и B086.
  Ветка меняет Project, после гейта нужен пин D051.
- Для финального прохода voxel-rust: поставить рантайм нового пина командой `runtime-add` и сменить `bin/*` на
  `bin/**` в `.agentrig/tools.yaml`; восемь `pub(crate)` в `src/native/host/game.rs` и `profiles.rs` вернуть к
  `pub(in crate::native)`; `#[allow(clippy::expect_used)]` можно заменить на `#[expect]`; ручные исключения
  file-size `src/Cargo.lock` и `src/package-lock.json` заменить поставляемыми шаблонами; в `.agentrig/checks.yaml`
  снять префикс `host::` у целей `game`, `worker` и `hardware`; перенести 13 тестовых файлов и убрать `**/*_tests.rs`;
  снизить бюджет `src/core/terrain` с 10 до 7; снять вторые ссылки D008 в `controller.rs` и D009 в `local.rs`;
  обработать или обосновать `.unwrap()` в телах макросов `src/exports/typescript.rs:26` и
  `src/schema/abi/records.rs:19`; в memory/State.md заменить блокер слияния на путь из D150: слияние в main с Linux
  после `runtime-add` там.

## Verification

- Кандидат ветки: `test_runtime.py` 4 passed; затронутые группы project_setup, test_delivery и
  tooling/distribution 84 passed, command_runner, command_line и project_config 34 passed; Rust 171 passed; Clippy
  чисто. С прежним шаблоном `bin/*` тест `runtime-add` падает на проверке tools.
- Клон voxel-rust `71440d0`: `runtime-add` перевёл раскладку, `check --only tools` PASS с `bin/**`, коммит через хуки
  запускает рантайм macOS и останавливается только на двух находках D148; с фальшивым `uname` Linux x86_64 хук
  отказывает с кодом 2 и называет платформу и команду.
- Настоящий Linux x86_64 не проверялся.

## Blockers

Нет.

## Next action

Коммит VAC P109, пин D051 на голову ветки и `just feature-merge`. Затем финальный проход voxel-rust на клоне и
финальный отчёт. Свободные ID: D151+, I022+, P110+, B102+ (B066 и D111 зарезервированы).
