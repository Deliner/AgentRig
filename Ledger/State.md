# State

## Focus

**Набор II** — переносимые на любое приложение, agent-native улучшения качества (10 кластеров C1–C10),
по одной карточке за ветку, автономно. Порядок: C1 зонирование → C2 границы окружения → C4 локальность →
C9 подавление линтов/`dbg!` → C6 эффекты/ошибки → C3 детерминизм → C5 контракты поведения → C10 эволюция
API/данных → C7 анти-дублирование → C8 анти-переусложнение. Слитая **C1+C2 (B034/P043)** — «Зоны и
границы окружения» — доставлена; дальше C4.

## Workspace

Ветка: feature/zoning (база master @ `ba28726`). VAC-B закоммичен — голова `5102816`. stable.txt=`7f44c5f`
(пин ещё не обновлён). Машина — macOS. Сборка/коммит/бустрап/merge — через `just scratch -- '...'` с PATH
(`cargo` в `~/.cargo/bin`, venv `tooling/.venv/bin` первым). `just feature-merge` — ВНУТРИ `just scratch`
с PATH. Agent-Bash — одна top-level `just`-команда без `&&`/хвостового `| tail`.

## Progress

VAC-B (`5102816`) закоммичен, полный гейт зелёный. Доставлены: поле `zone: pure` в контракте
`architecture.yaml` и правило границы окружения (`std::env`/`std::fs`/`std::process` в собственных файлах
`pure`-каталога — ошибка), решение D078, догфуд-тег на `architecture/resolution`. Завершённые карточки
B034/P043 перенесены в Archive (этот коммит ротации): строки убраны из Plan.md/Backlog.md, карточки — в
[Archive/Plan/043.md](Archive/Plan/043.md)/[Archive/Backlog/034.md](Archive/Backlog/034.md), ссылка card в
D078 переведена на `Archive/Backlog/034.md`, в Plan.md добавлен абзац о доставке. Plan и Backlog теперь пусты.

## Verification

Коммит-гейт VAC-B (`5102816`) весь PASS: build/lint/memory/rustfmt/clippy/rust-test 69/ruff-format/ruff/mypy/
pytest 202 (вкл. architecture-набор и test_delivery/I020)/typos/vulture. master впереди origin на 166 (не пушить).

## Blockers

Нет.

## Next action

Закоммитить ротацию карточек в Archive через хук (это голова X, гейт зелёный). Затем D051: (1) коммит
«пин <X-short>» — stable.txt=X (40 симв. + `\n`) + `just bootstrap` ВНУТРИ одного `just scratch` с PATH;
(2) `just feature-merge` ВНУТРИ `just scratch` с PATH (не пушить, ветку сохранить). Отчёт на русском.
Следующий кластер Набора II — C4 (локальность/радиус правки): завести карточку C4 → реализовать → догфуд
→ merge+пин. Свободный ID: P044.
