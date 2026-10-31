# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров C1–C10),
по одной карточке за ветку, автономно. Порядок: C1 зонирование → C2 границы окружения → C4 локальность →
C9 подавление линтов/`dbg!` → C6 эффекты/ошибки → C3 детерминизм → C5 контракты поведения → C10 эволюция
API/данных → C7 анти-дублирование → C8 анти-переусложнение. Идёт **слитая C1+C2 (B034/P043)** — «Зоны и
границы окружения».

## Workspace

Ветка: feature/zoning (база master @ `ba28726`, голова `c3ae642`). stable.txt=`7f44c5f` (установлен).
Машина — macOS. Сборка/коммит/бустрап/merge — через `just scratch -- '...'` с PATH (`cargo` в
`~/.cargo/bin`, venv `tooling/.venv/bin` первым). `just feature-merge` — ВНУТРИ `just scratch` с PATH.
Agent-Bash — одна top-level `just`-команда без `&&`/хвостового `| tail`.

## Progress

VAC-A (`e2c20fc`) и re-scope (`c3ae642`) закоммичены. VAC-B реализован в рабочем дереве (НЕ закоммичен):
опциональное поле `zone` (enum `Pure`) в `Declaration` (`policy.rs`); общий `ENVIRONMENT`/`EnvironmentUse`/
`environment()` (`mod.rs`); сбор обращений к окружению (`runner/analysis.rs`, `runner/mod.rs`);
`check_zones` (`policy.rs`); два поведенческих теста (`tests.rs`); самоописание контракта
(`rules/architecture.rs`); догфуд-тег `zone: pure` на `architecture/resolution`. Записано решение
**D078** (индекс + карточка, расширяет D044). Доставляемый минимум — один класс `pure`, три запрещённых
namespace (`std::env`/`std::fs`/`std::process`, near-zero-FP). Нового rule-kind/skill нет → I020 без изменений.

## Verification

Кандидат-линт по дереву ЧИСТ (`tooling/worker/build lint`). Архитектурные тесты: 42 passed, 0 failed,
включая `zone_pure_directories_reject_environment_access_in_their_own_files` и
`environment_namespaces_match_only_at_a_path_boundary`. Полный гейт на коммите ещё не прогонялся.
master впереди origin на 166 (не пушить).

## Blockers

Нет.

## Next action

Закоммитить VAC-B (движок + тесты + D078 + самоописание + догфуд-yaml + этот State) через хук, гейт
зелёный. Затем по D050/D051: (1) коммит «архивировать завершённые B034/P043» — строку+карточку Backlog и
Plan перенести в Archive/ (edit-plan/edit-backlog), это голова X; (2) коммит «пин X (D051)» —
stable.txt=X (40 симв. + `\n`) + `just bootstrap` ВНУТРИ одного scratch; (3) `just feature-merge` ВНУТРИ
scratch с PATH (не пушить, ветку сохранить). Отчёт на русском. Следующий кластер — C4 локальность/радиус.
Свободный ID: P044.
