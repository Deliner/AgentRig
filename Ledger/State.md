# State

## Focus

Программа **P083 → P086** (пользователь 2026-11-07: «бери все и делай») — карточки B075–B078 по одной за ветку,
автономно. Первая — P083 (B077 impact-coverage): `config-check` сообщает файлы, которые карта `affected` оставляет без
выбранных тестов, и тесты, которые не выбирает ни одна цель. В этом VAC P083 и B077 ротированы в Archive, решение
D124.

## Workspace

Branch: `feature/impact-coverage`
Revision: `f10cb30541d2f0302a5330fe8928316b886757c3`

База — master `f10cb30`. Ветки `feature/rule-cards`, `feature/ratchet-baseline` и прежние, а также
`feature/init-probe`, сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P083: модуль `Project/project_setup/coverage.rs`, вызов из `config-check` в `project_setup/doctor.rs`, тест
  `project_setup/tests/test_coverage.py`, README гейта и установки, решение D124. Вариант в `change_delivery/gate`
  превышал бюджет зависимых `shared/fs` (21 при 20) и отвергнут; сопоставление с группами — общий тип
  `AffectedMap` в `change_delivery/gate/selection.rs`. Карта AgentRig:
  `Project/shared/mod.rs` и `Project/shared/console/**` сопоставлены с `::` в `rust-test`, пример
  `test_sample.py` добавлен в цели `test`.
- Plan: P084 (B078), P085 (B076), P086 (B075) — pending, карточки `Ledger/Plan/084.md`–`086.md`.

## Verification

- 2026-11-07, рабочее дерево до ремонта карты: `just candidate config-check` — код 1, три находки:
  `Project/shared/console/mod.rs`, `Project/shared/mod.rs` в `rust-test`,
  `Project/shared/testing/examples/python/application/test_sample.py` в `test`.
- После ремонта: `just candidate config-check` — «Scaffold configuration is valid (runtime 0.3.0)».
- pytest `Project/change_delivery/gate/tests/test_coverage.py` — 11 passed.

## Blockers

Нет.

## Next action

Коммит VAC через гейт, пин D051 (`stable.txt` = голова ветки, `just bootstrap`), `just feature-merge`. Затем ветка
`feature/disallowed-methods` для P084 (B078). Свободные ID: D125+, I022+, P087+, B079+ (B066 и D111
зарезервированы).
