# State

## Focus

Пользователь 2026-11-10 поручил: «Сохрани показанные карточки, заведи пункт вот по этому по линту, который ты
написал, и все карточки, и вообще все карточки в бэклоге, которые есть, автономно реализуй сам. У меня ничего не
спрашиваю, я на все даю подтверждение». Все карточки Backlog B086–B101 выбраны в Plan как P094–P109 и выполняются
автономно по одной за ветку; решения, которые карточки оставляли пользователю, исполнитель принимает сам и
записывает. Риг дорабатывается для себя через докфудинг и проверяется на voxel-rust; под voxel-rust он не
подгоняется.

## Workspace

Branch: `feature/test-dependents` (P105, B089; завершение P104, B091)
Revision: `acafddd` (VAC P105; завершение P104 в этом коммите)

Установленный рантайм `3ddf1cc` (пин P104). macOS: коммиты и `feature-merge` с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; фокусные
тесты через `just run test` тоже требуют этого PATH для сборки кандидата.
Ветки прежних поставок сохранены, `feature/init-probe` не трогать.
voxel-rust: ветка `feature/agentrig-migration`, коммит `71440d0`, рабочее дерево чистое, в main не слита, push нет.
В memory/State.md voxel-rust число тестовых модулей 21 неверно: живой счёт 13 файлов `*_tests.rs`.
Клон voxel-rust в scratchpad `voxel-b101` несёт незакоммиченный перенос 13 тестовых файлов в `tests/` для AC4 B091 и
бюджет `src/core/terrain` 7; копия переноса — `p104-voxel.patch` и `p104-voxel-untracked.tgz` там же. Рабочее
дерево `voxel-orig` в scratchpad — чистый `71440d0` того же клона для замеров.

## Progress

- P094–P103 завершены и слиты в master.
- P105 (B089) выполнена в `acafddd`: тестовые рёбра не тратят бюджет входящих зависимостей, D146, ротация.
- P104 (B091) завершается в этом VAC: AC4 подтверждён на клоне кандидатом с P105, ротация P104 и B091.
- Для финального прохода voxel-rust: восемь `pub(crate)` в `src/native/host/game.rs` и `profiles.rs` вернуть к
  `pub(in crate::native)`; `#[allow(clippy::expect_used)]` можно заменить на `#[expect]`; ручные исключения
  file-size `src/Cargo.lock` и `src/package-lock.json` заменить поставляемыми шаблонами; в `.agentrig/checks.yaml`
  снять префикс `host::` у целей `game`, `worker` и `hardware`; перенести 13 тестовых файлов и убрать `**/*_tests.rs`;
  снизить бюджет `src/core/terrain` с 10 до 7.

## Verification

- Кандидат: тест `test_dependents_budget_counts_only_production_uses` и все тесты `Project/structural_lint/architecture`
  — 140 passed; на установленном `3ddf1cc` новый тест падает с 4 зависимыми. Rust-тесты 168 passed, Clippy и
  `lint` AgentRig кандидатом чисты, `memory-check` без ошибок.
- Клон voxel-rust: кандидат насчитывает у `src/core/terrain` 7 зависимых на исходном и перенесённом дереве против 10 и
  12 у установленного рантайма; с бюджетом 7 lint клона чист, с 6 даёт находку.
- AC4 B091 на клоне после `acafddd`: исключений `_tests.rs` в lint.yaml 0, файлов `*_tests.rs` 0, в `tests/` 13;
  кандидат с правилом test-placement даёт чистый `lint` и `config-check`. Гейт коммита `acafddd`: 249 passed.

## Blockers

Нет.

## Next action

Коммит завершения P104, пин D051 на голову ветки, коммит пина, `just feature-merge`. Дальше P106 (B099). Свободные
ID: D147+, I022+, P110+, B102+ (B066 и D111 зарезервированы).
