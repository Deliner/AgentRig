# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109 и выполняются
автономно по одной за ветку; решения, которые карточки оставляли пользователю, исполнитель принимает сам и
записывает. Риг дорабатывается для себя через докфудинг и проверяется на voxel-rust; под voxel-rust он не
подгоняется.

## Workspace

Branch: `feature/single-marker` (P106, B099)
Revision: `b4560ce` (слияние P105; VAC P106 в этом коммите)

Установленный рантайм `9efbe20` (пин P105). macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; фокусные
тесты через `just run test` тоже требуют этого PATH для сборки кандидата.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.
Клон voxel-rust в scratchpad `voxel-b101` несёт незакоммиченный перенос 13 тестовых файлов в `tests/`, бюджет
`src/core/terrain` 7 и снятые вторые ссылки D008 и D009; копия переноса — `p104-voxel.patch` и
`p104-voxel-untracked.tgz` там же. Рабочее дерево `voxel-orig` в scratchpad — чистый `71440d0` того же клона.

## Progress

- P094–P105 завершены и слиты в master.
- P106 (B099) выполняется в этом VAC: обоснованный `#[allow]` или `#[expect]` линта Clippy у охватывающего элемента
  покрывает его паники, D147, ротация P106 и B099.
- Для финального прохода voxel-rust: восемь `pub(crate)` в `src/native/host/game.rs` и `profiles.rs` вернуть к
  `pub(in crate::native)`; `#[allow(clippy::expect_used)]` можно заменить на `#[expect]`; ручные исключения
  file-size `src/Cargo.lock` и `src/package-lock.json` заменить поставляемыми шаблонами; в `.agentrig/checks.yaml`
  снять префикс `host::` у целей `game`, `worker` и `hardware`; перенести 13 тестовых файлов и убрать `**/*_tests.rs`;
  снизить бюджет `src/core/terrain` с 10 до 7; снять вторые ссылки D008 в `controller.rs` и D009 в `local.rs`.

## Verification

- Кандидат: Rust-тест `rust_referenced_clippy_allow_on_an_enclosing_item_covers_its_panics` проходит, все Rust-тесты
  169 passed, тесты `Project/structural_lint/languages` 29 passed; Clippy и `lint` AgentRig кандидатом чисты,
  `memory-check` без ошибок.
- Клон voxel-rust со снятыми вторыми ссылками D008 и D009: установленный `9efbe20` даёт 3 находки panic-path,
  кандидат ни одной.

## Blockers

Нет.

## Next action

Коммит VAC P106, пин D051 на голову ветки, коммит пина, `just feature-merge`. Дальше P107 (B097). Свободные ID:
D148+, I022+, P110+, B102+ (B066 и D111 зарезервированы).
