# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109 и выполняются
автономно по одной за ветку; решения, которые карточки оставляли пользователю, исполнитель принимает сам и
записывает. Риг дорабатывается для себя через докфудинг и проверяется на voxel-rust; под voxel-rust он не
подгоняется.

## Workspace

Branch: `feature/edge-findings` (P102, B088)
Revision: `71fdfba` (VAC1 P102; VAC2 в этом коммите)

Установленный рантайм `6874047` (пин P101). macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; фокусные
тесты через `just run test` тоже требуют этого PATH для сборки кандидата.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.

## Progress

- P094–P101 завершены и слиты в master, пин `6874047`.
- P102 выполнен двумя VAC и ротирован в Archive: `71fdfba` — рёбра в находках бюджета и цикла и единый формат
  печати (D142); второй — команда `lint-edges DIR [--json]` из того же измерения, что и lint (D143), README
  архитектуры со всеми полями контракта. Осталось: пин D051 и `just feature-merge`.
- Для финального прохода voxel-rust: восемь `pub(crate)` в `src/native/host/game.rs` и `profiles.rs` вернуть к
  `pub(in crate::native)`; `#[allow(clippy::expect_used)]` можно заменить на `#[expect]`; ручные исключения
  file-size `src/Cargo.lock` и `src/package-lock.json` заменить поставляемыми шаблонами.

## Verification

- P102 VAC1: гейт коммита `71fdfba` — 420 passed. VAC2: `Project/structural_lint`, `test_hooks.py` и
  `Project/project_setup` на кандидате — 319 passed; Rust-тесты — 167 passed; `just lint`, `memory-check` чистые.
  Тест команды и проверка охраны падают на установленном `6874047`.
- AC3 на клоне voxel-rust `71440d0` с `dependents: 0` у `src/core/world`: lint насчитал и назвал 13 зависимых,
  `lint-edges` показал те же 13 каталогов (171 различное ребро); правка клона откачена.

## Blockers

Нет.

## Next action

Пин D051 на голову ветки, коммит пина, `just feature-merge`; дальше P103 (B090, цели тестов в `config-check`).
Свободные ID: D144+, I022+, P110+, B102+ (B066 и D111 зарезервированы).
