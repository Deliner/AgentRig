# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109 и выполняются
автономно по одной за ветку; решения, которые карточки оставляли пользователю, исполнитель принимает сам и
записывает. Риг дорабатывается для себя через докфудинг и проверяется на voxel-rust; под voxel-rust он не
подгоняется.

## Workspace

Branch: `feature/cargo-targets` (P103, B090)
Revision: `0259204` (база master после слияния P102; VAC1 P103 в этом коммите)

Установленный рантайм `3c99a34` (пин P102). macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; фокусные
тесты через `just run test` тоже требуют этого PATH для сборки кандидата.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.

## Progress

- P094–P102 завершены и слиты в master, пин `3c99a34`.
- P103 выполнен одним VAC и ротирован в Archive: `config-check` берёт путь модуля тестового файла из дерева `mod`,
  `#[path]` и `include!` от корней крейтов манифеста команды и принимает цель внутри пути или с его целых последних
  сегментов (D144). Осталось: пин D051 и `just feature-merge`.
- Для финального прохода voxel-rust: восемь `pub(crate)` в `src/native/host/game.rs` и `profiles.rs` вернуть к
  `pub(in crate::native)`; `#[allow(clippy::expect_used)]` можно заменить на `#[expect]`; ручные исключения
  file-size `src/Cargo.lock` и `src/package-lock.json` заменить поставляемыми шаблонами; в `.agentrig/checks.yaml`
  снять префикс `host::` у целей `game`, `worker` и `hardware`.

## Verification

- P103: `test_coverage.py` на кандидате — 18 passed; на установленном `3c99a34` три новых случая падают, случай AC2
  проходит на обоих. Rust-тесты — 167 passed; `just lint` и Clippy чистые. Настоящий cargo на фикстуре выбирает по
  `game::tests::` и `widget::` те же тесты и ничего по `host::parts::thing::tests::`.
- Клон voxel-rust `71440d0`: без правок оба рантайма чисты; без `host::` установленный отклоняет `game.rs`,
  `hardware.rs`, `worker.rs`, кандидат принимает; цель `gamez::tests::` кандидат отклоняет. Правка откачена.
- `config-check` на AgentRig: кандидат и установленный рантайм чисты.

## Blockers

Нет.

## Next action

Коммит VAC P103, пин D051 на голову ветки, коммит пина, `just feature-merge`; дальше P104 (B091, размещение
тестов). Свободные ID: D145+, I022+, P110+, B102+ (B066 и D111 зарезервированы).
