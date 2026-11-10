# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109 и выполняются
автономно по одной за ветку; решения, которые карточки оставляли пользователю, исполнитель принимает сам и
записывает. Риг дорабатывается для себя через докфудинг и проверяется на voxel-rust; под voxel-rust он не
подгоняется.

## Workspace

Branch: `feature/macro-panics` (P107, B097)
Revision: `ae63ead` (слияние P106; VAC P107 в этом коммите)

Установленный рантайм `5b5a682` (пин P106). macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; фокусные
тесты через `just run test` тоже требуют этого PATH для сборки кандидата.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.
Клон voxel-rust в scratchpad `voxel-b101` несёт незакоммиченный перенос 13 тестовых файлов в `tests/`, бюджет
`src/core/terrain` 7 и снятые вторые ссылки D008 и D009; копия переноса — `p104-voxel.patch` и
`p104-voxel-untracked.tgz` там же. Рабочее дерево `voxel-orig` в scratchpad — чистый `71440d0` того же клона.

## Progress

- P094–P106 завершены и слиты в master.
- P107 (B097) выполняется в этом VAC: panic-path видит паники в телах `macro_rules!`, D148; детекторы паник
  вынесены в `languages/rust/panic_surface.rs`; ротация P107 и B097.
- Для финального прохода voxel-rust: восемь `pub(crate)` в `src/native/host/game.rs` и `profiles.rs` вернуть к
  `pub(in crate::native)`; `#[allow(clippy::expect_used)]` можно заменить на `#[expect]`; ручные исключения
  file-size `src/Cargo.lock` и `src/package-lock.json` заменить поставляемыми шаблонами; в `.agentrig/checks.yaml`
  снять префикс `host::` у целей `game`, `worker` и `hardware`; перенести 13 тестовых файлов и убрать `**/*_tests.rs`;
  снизить бюджет `src/core/terrain` с 10 до 7; снять вторые ссылки D008 в `controller.rs` и D009 в `local.rs`;
  обработать или обосновать `.unwrap()` в телах макросов `src/exports/typescript.rs:26` и
  `src/schema/abi/records.rs:19`.

## Verification

- Кандидат: Rust-тесты `rust_flags_unjustified_panics_inside_macro_rules_bodies` и
  `rust_skips_macro_rules_bodies_in_test_code_or_under_a_referenced_clippy_allow` проходят, до правки первый падал;
  все Rust-тесты 171 passed, тесты `Project/structural_lint/languages` 16 passed; Clippy и `lint` AgentRig кандидатом
  чисты, `memory-check` без ошибок.
- Клон voxel-rust `71440d0`: кандидат даёт 2 находки panic-path в телах `macro_rules!`, установленный `5b5a682` ни
  одной; на AgentRig ноль у обоих.

## Blockers

Нет.

## Next action

Коммит VAC P107, пин D051 на голову ветки, коммит пина, `just feature-merge`. Дальше P108 (B096). Свободные ID:
D149+, I022+, P110+, B102+ (B066 и D111 зарезервированы).
