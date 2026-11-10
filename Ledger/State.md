# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109 и выполняются
автономно по одной за ветку; решения, которые карточки оставляли пользователю, исполнитель принимает сам и
записывает. Риг дорабатывается для себя через докфудинг и проверяется на voxel-rust; под voxel-rust он не
подгоняется.

## Workspace

Branch: `feature/test-placement` (P104, B091)
Revision: `1ad190b` (база master после слияния P103; фаза 1 P104 в этом коммите)

Установленный рантайм `2ee3320` (пин P103). macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; фокусные
тесты через `just run test` тоже требуют этого PATH для сборки кандидата.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.
Клон voxel-rust в scratchpad `voxel-b101` несёт незакоммиченный перенос 13 тестовых файлов в `tests/` для AC4;
его копия — `p104-voxel.patch` и `p104-voxel-untracked.tgz` там же.

## Progress

- P094–P103 завершены и слиты в master, пин `2ee3320`.
- P104 фаза 1: вид `test-placement` вне каталога `ALL`, измерение `languages/rust/test_placement.rs`, навык
  `relocate-tests`, тесты и D145. Новый вид должен попасть в установленный рантайм до записи в
  `.agentrig/lint.yaml`, а I020 требует, чтобы вид из поставки `init` был и в конфиге AgentRig; поэтому дальше пин
  D051 посреди ветки, затем фаза 2: `ALL`, правило в `.agentrig/lint.yaml` и тест поставляемых исключений.
- AC4 B091 зависит от P105: после переноса в `tests/` пять каталогов voxel-rust превышают бюджет входящих
  зависимостей за счёт тестовых каталогов, а это и есть B089.
- Для финального прохода voxel-rust: восемь `pub(crate)` в `src/native/host/game.rs` и `profiles.rs` вернуть к
  `pub(in crate::native)`; `#[allow(clippy::expect_used)]` можно заменить на `#[expect]`; ручные исключения
  file-size `src/Cargo.lock` и `src/package-lock.json` заменить поставляемыми шаблонами; в `.agentrig/checks.yaml`
  снять префикс `host::` у целей `game`, `worker` и `hardware`; перенести 13 тестовых файлов и убрать `**/*_tests.rs`.

## Verification

- P104 на кандидате: Rust-тесты 168 passed, новый тест размещения среди них. `lint`, Clippy, `memory-check` и
  `config-check` AgentRig чистые. Первая попытка коммита с видом в `ALL` упала на
  `test_dogfooding_capabilities_stay_universal`: вид был в поставке `init`, но не в `.agentrig/lint.yaml`.
- Клон voxel-rust: правило даёт ровно 13 находок, из них две через `#[path]`; после переноса с `#[path]` и снятия 16
  исключений находок test-placement нет, остаются пять превышений бюджета входящих зависимостей.

## Blockers

Нет.

## Next action

Коммит фазы 1 P104, пин D051 на голову ветки и коммит пина; фаза 2: правило в `.agentrig/lint.yaml`, P104 на паузу
до P105 с записью Delivery; пин, `just feature-merge`; затем P105. Свободные ID: D146+, I022+, P110+, B102+ (B066 и
D111 зарезервированы).
