# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров
C1–C10), по одной карточке за ветку, автономно. Порядок: C1 зонирование → C2 границы окружения →
C4 локальность → C9 подавление линтов/`dbg!` → C6 эффекты/ошибки → C3 детерминизм → C5 контракты
поведения → C10 эволюция API/данных → C7 анти-дублирование → C8 анти-переусложнение. Слитая
**C1+C2 (B034/P043)** и **C4 (B035/P044)** доставлены. Дальше — **C9** (подавление линтов/`dbg!`).

## Workspace

Ветка feature/locality (база master @ `7cd8207`). VAC-B закоммичен — голова `1324dac`. Ротация
карточек B035/P044 в Archive — в рабочем дереве, пин ещё не обновлён (stable.txt=`5d01af9`). Машина
— macOS. Сборка/кандидат/коммит/бустрап/merge — через `just scratch -- '...'` с PATH (`cargo` в
`~/.cargo/bin`, venv `tooling/.venv/bin` первым). Agent-Bash — одна top-level `just`-команда без
`&&`/хвостового `| tail`.

## Progress

VAC-B (`1324dac`) закоммичен, полный гейт зелёный. Доставлено: поле `dependents` в контракте
`architecture.yaml` и правило входящей связности (`directory-architecture` считает радиус каждого
каталога по графу и сообщает о превышении бюджета, дефолт 6, на уровне error), решение D079
(расширяет D078). Догфуд: шесть общих хабов объявили реальный радиус (`shared/paths`/`shared/vcs`
12, `project_config` 11, `shared/testing`/`shared/diagnostics` 10, `shared/yaml` 9). Завершённые
карточки B035/P044 перенесены в Archive (эта ротация в рабочем дереве): строки убраны из
Backlog.md/Plan.md, карточки — в [Archive/Plan/044.md](Archive/Plan/044.md)/[Archive/Backlog/035.md](Archive/Backlog/035.md),
ссылка card в D079 переведена на `Archive/Backlog/035.md`, в Plan.md добавлен абзац о доставке.

## Verification

Коммит-гейт VAC-B (`1324dac`) весь PASS: build/lint/memory/rustfmt/clippy/rust-test 87/ruff-format/
ruff/mypy/pytest 358 (вкл. architecture-набор и test_delivery/I020)/typos/vulture. Ротацию карточек
проверит memory-check при коммите.

## Blockers

Нет.

## Next action

Закоммитить ротацию карточек в Archive через хук (голова X, гейт зелёный). Затем D051: (1) коммит
«пин <X-short>» — stable.txt=X (40 симв. + `\n`) + `just bootstrap` ВНУТРИ одного `just scratch` с
PATH; (2) `just feature-merge` ВНУТРИ `just scratch` с PATH (не пушить, ветку сохранить). Отчёт на
русском. Следующий кластер Набора II — **C9** (подавление линтов/`dbg!`): завести карточку → реализовать
→ догфуд → merge+пин. Свободные ID: P045, B036, D080.
