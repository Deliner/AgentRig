# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Находки оформлены карточками B052–B064 (B064 — идея, не выбрана),
дополнена B049 (AC2); выбраны в Plan **P055–P068** (порядок — в Plan.md). Текущая карточка —
P055 (B052, новый вид правила `global-state`).

## Workspace

Ветка `feature/global-state` (база master `32debc8`), наблюдаемый HEAD до memory-VAC — `32debc8`.
Первый коммит — memory-only (`Ledger/`); затем VAC-B1 P055 (новый `kind:` → мид-бранч пин D051).
macOS; коммиты/cargo — через `just scratch` с восстановленным PATH (`cargo`/`uv`), иначе гейт
падает на `build: cargo not found`.

## Progress

- Memory-VAC подготовлен, не закоммичен: карточки `Ledger/Backlog/052..064.md`, строки индекса
  Backlog, дополнение B049 (AC2 + границы), строки Plan P055–P068 с абзацем выбора, карточки
  `Ledger/Plan/055..068.md`, State.
- Код не менялся; P055 не начат.

## Verification

Карточки и Plan — чистая память; реализация не начиналась. Memory-VAC проходит через гейт
(exported-index + затронутые тест-группы + smoke) при коммите. Приёмочные критерии правил не
верифицированы (будущая работа при доставке каждой карточки).

## Blockers

Нет.

## Next action

Закоммитить memory-VAC через `.agentrig/hooks/pre-commit` (`Ledger/Backlog.md`,
`Ledger/Backlog/049.md`, `Ledger/Backlog/052..064.md`, `Ledger/Plan.md`, `Ledger/Plan/055..068.md`,
`Ledger/State.md`), затем P055 VAC-B1: `Kind::GlobalState` + детектор (отдельный файл рядом с
`languages/rust.rs`) + тесты в новом файле `languages/tests/` + decision D096; мид-бранч пин
stable.txt + bootstrap; VAC-B2 проводка (`catalog::ALL`, `lint.yaml`, навык `own-the-state`,
`assets.rs`); догфуд `just candidate lint` → 0; живая проба. Свободные ID: D096+, P069+, B065+.
