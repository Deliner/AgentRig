# State

## Focus

На 2026-10-28 выполняется согласованный набор улучшений (одна выборка, автономно, D032)
в порядке: B3 (исключить `Ledger/Archive/Backlog` из `directory-size`, решение D056) →
C1 (индекс норм «хорошего состояния») → [B023](Backlog/023.md) (статическая проверка
публичной поверхности тестов) → [B024](Backlog/024.md) (размещение теста по покрытию).
Текущий шаг — запись идей B023/B024 в Backlog.

## Workspace

Branch: feature/record-improvement-ideas

Revision: 2df2b08 (родитель; этот VAC добавляет запись B023/B024)

Машина — macOS (Darwin 25.5.0). Этот VAC меняет только Ledger, пин по D051 не нужен.
B023/B024 меняют Project — там перед merge потребуются пин stable.txt и bootstrap
(запускает пользователь из обычного терминала с PATH, иначе хуки сессии выключаются).

## Progress

B023 и B024 записаны в Backlog ([Backlog.md](Backlog.md) + карточки
[023](Backlog/023.md)/[024](Backlog/024.md)) в показанном и согласованном виде. B3 и C1
проводятся как обслуживание/память (через Backlog не идут). Живой Plan пуст; промоушен
B023/B024 в Plan — на их ветках доставки.

## Verification

Этот VAC проверяется коммит-гейтом (build, lint, memory, rustfmt, clippy, rust-test,
ruff/ruff-format/mypy, pytest, typos, vulture); memory-check — соответствие индекса
Backlog и карточек.

## Blockers

Нет.

## Next action

Закоммитить запись B023/B024 через гейт и влить ветку (`just feature-merge`). Затем
ветка B3: добавить `Ledger/Archive/Backlog` в `exclude` правила `directory-size` в
`.agentrig/lint.yaml` с `# DECISION: D056`, создать `Ledger/Decisions/056.md`, добавить
строку и заметку о преемственности (D048/D030) в `Decisions.md`. Затем C1, затем B023,
затем B024.
