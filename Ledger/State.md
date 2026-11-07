# State

## Focus

Программа **P083 → P086** (пользователь 2026-11-07: «бери все и делай») — карточки B075–B078 по одной за ветку,
автономно. P083 (B077 impact-coverage) выполнена в VAC `f2b4962`: `config-check` сообщает файлы, которые карта
`affected` оставляет без выбранных тестов, и тесты, которые не выбирает ни одна цель; P083 и B077 в Archive, решение
D124. Этот коммит — пин D051.

## Workspace

Branch: `feature/impact-coverage`
Revision: `f2b496214dc4839a3c4a3e3da774456effcde6fc`

База — master `f10cb30`. Ветки `feature/rule-cards`, `feature/ratchet-baseline` и прежние, а также
`feature/init-probe`, сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P083: модуль `Project/project_setup/coverage.rs`, вызов из `config-check` в `project_setup/doctor.rs`, тест
  `project_setup/tests/test_coverage.py`, README гейта и установки, решение D124. Вариант в `change_delivery/gate`
  превышал бюджет зависимых `shared/fs` (21 при 20) и отвергнут; сопоставление с группами — общий тип
  `AffectedMap` в `change_delivery/gate/selection.rs`. Карта AgentRig: `Project/shared/mod.rs` и
  `Project/shared/console/**` сопоставлены с `::` в `rust-test`, пример `test_sample.py` добавлен в цели `test`.
- Пин: `stable.txt` = `f2b4962`.
- Plan: P084 (B078), P085 (B076), P086 (B075) — pending, карточки `Ledger/Plan/084.md`–`086.md`.

## Verification

- До ремонта карты `just candidate config-check` — код 1, три находки: `Project/shared/console/mod.rs`,
  `Project/shared/mod.rs` в `rust-test`, `Project/shared/testing/examples/python/application/test_sample.py` в `test`;
  после ремонта — «Scaffold configuration is valid (runtime 0.3.0)».
- Гейт коммита `f2b4962`: build, lint, memory, rustfmt, clippy, rust-test 151 passed, ruff-format, ruff, mypy, test
  224 passed, typos, vulture — PASS.
- `just bootstrap` на `f2b4962` — установлен `agentrig 0.3.0`.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/impact-coverage`. Затем ветка `feature/disallowed-methods` для P084 (B078).
Свободные ID: D125+, I022+, P087+, B079+ (B066 и D111 зарезервированы).
