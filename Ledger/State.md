# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109 и выполняются
автономно по одной за ветку; решения, которые карточки оставляли пользователю, исполнитель принимает сам и
записывает. Риг дорабатывается для себя через докфудинг и проверяется на voxel-rust; под voxel-rust он не
подгоняется.

## Workspace

Branch: `feature/method-edges` (P108, B096)
Revision: `fc7c4b4` (слияние P107; VAC P108 в этом коммите)

Установленный рантайм `7bade55` (пин P107). macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; фокусные
тесты через `just run test` тоже требуют этого PATH для сборки кандидата.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.
Клон voxel-rust в scratchpad `voxel-b101` несёт незакоммиченный перенос 13 тестовых файлов в `tests/`, бюджет
`src/core/terrain` 7 и снятые вторые ссылки D008 и D009; копия переноса — `p104-voxel.patch` и
`p104-voxel-untracked.tgz` там же. Рабочее дерево `voxel-orig` в scratchpad — чистый `71440d0` того же клона.

## Progress

- P094–P107 завершены и слиты в master.
- P108 (B096) закрывается в этом VAC отчётом без изменения рига: замер потерянных рёбер через вызовы методов,
  решение D149 оставить D059 в силе, ротация P108 и B096. Ветка не меняет Project, пин D051 не нужен.
- Для финального прохода voxel-rust: восемь `pub(crate)` в `src/native/host/game.rs` и `profiles.rs` вернуть к
  `pub(in crate::native)`; `#[allow(clippy::expect_used)]` можно заменить на `#[expect]`; ручные исключения
  file-size `src/Cargo.lock` и `src/package-lock.json` заменить поставляемыми шаблонами; в `.agentrig/checks.yaml`
  снять префикс `host::` у целей `game`, `worker` и `hardware`; перенести 13 тестовых файлов и убрать `**/*_tests.rs`;
  снизить бюджет `src/core/terrain` с 10 до 7; снять вторые ссылки D008 в `controller.rs` и D009 в `local.rs`;
  обработать или обосновать `.unwrap()` в телах макросов `src/exports/typescript.rs:26` и
  `src/schema/abi/records.rs:19`.

## Verification

- Оракул `method_edges.py` в scratchpad против `lint-edges` установленного `7bade55`: потеряно 2 ребра каталогов из
  235 в AgentRig и 1 из 105 на клоне voxel-rust `71440d0`, все три проверены вручную; ни одно не превышает бюджет
  `dependents` и не замыкает цикл. `memory-check` установленным рантаймом без ошибок.

## Blockers

Нет.

## Next action

Коммит VAC P108 и `just feature-merge` без пина. Дальше P109 (B086). Свободные ID: D150+, I022+, P110+, B102+
(B066 и D111 зарезервированы).
