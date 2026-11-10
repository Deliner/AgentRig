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
Revision: `89baa3a` (пин D051 фазы 1 `2b6db8b`; фаза 2 P104 в этом коммите)

Установленный рантайм `2b6db8b` (пин фазы 1 P104). macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; фокусные
тесты через `just run test` тоже требуют этого PATH для сборки кандидата.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.
Клон voxel-rust в scratchpad `voxel-b101` несёт незакоммиченный перенос 13 тестовых файлов в `tests/` для AC4 B091;
его копия — `p104-voxel.patch` и `p104-voxel-untracked.tgz` там же.

## Progress

- P094–P103 завершены и слиты в master.
- P104: фаза 1 `2b6db8b` дала вид `test-placement`, измерение, навык `relocate-tests`, тесты и D145, пин `89baa3a`;
  фаза 2 включает вид в каталог `init` и правило в `.agentrig/lint.yaml`. P104 на паузе до P105: AC4 требует, чтобы
  тестовые каталоги не тратили бюджет входящих зависимостей (B089).
- Для финального прохода voxel-rust: восемь `pub(crate)` в `src/native/host/game.rs` и `profiles.rs` вернуть к
  `pub(in crate::native)`; `#[allow(clippy::expect_used)]` можно заменить на `#[expect]`; ручные исключения
  file-size `src/Cargo.lock` и `src/package-lock.json` заменить поставляемыми шаблонами; в `.agentrig/checks.yaml`
  снять префикс `host::` у целей `game`, `worker` и `hardware`; перенести 13 тестовых файлов и убрать `**/*_tests.rs`.

## Verification

- Фаза 2 на кандидате: `test_config_check.py`, `test_delivery.py` и тесты project_setup — 62 passed; на
  установленном `2b6db8b` падают тест поставляемых исключений и `test_dogfooding_capabilities_stay_universal`.
  `just lint` установленным рантаймом с новым правилом чист, `just config-check` чист.
- Фаза 1: Rust-тесты 168 passed; гейт коммита 322 passed.
- Клон voxel-rust: правило даёт ровно 13 находок, из них две через `#[path]`; после переноса с `#[path]` и снятия 16
  исключений находок test-placement нет, cargo `--lib` проходит 132 теста, из них 77 перенесённых; остаются пять
  превышений бюджета входящих зависимостей.

## Blockers

AC4 B091 ждёт P105.

## Next action

Коммит фазы 2 P104, пин D051 на голову ветки, коммит пина, `just feature-merge`; затем P105 (B089), после него
проверка AC4 B091 на клоне и ротация P104. Свободные ID: D146+, I022+, P110+, B102+ (B066 и D111 зарезервированы).
