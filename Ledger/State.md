# State

## Focus

На 2026-10-28 выполняется согласованный набор улучшений (одна выборка, автономно, D032)
в порядке: B3 → C1 → [B023](Backlog/023.md) → [B024](Backlog/024.md). Выполнено: запись
B023/B024 в Backlog (влито). Текущий шаг — B3: исключить `Ledger/Archive/Backlog` из
`directory-size`, решение D056.

## Workspace

Branch: feature/archive-backlog-size-exempt

Revision: 568c683 влит в master через 2df2b08-потомок; этот VAC — поверх master.

Машина — macOS (Darwin 25.5.0). B3 меняет только `.agentrig/lint.yaml` и Ledger —
пин по D051 не нужен. B023/B024 меняют Project — там перед merge потребуются пин
stable.txt и bootstrap (запускает пользователь из обычного терминала с PATH). Merge
запускать через `just scratch -- '... tooling/worker/run feature-merge'` с PATH (прямой
`just feature-merge` не видит cargo).

## Progress

B3: в `.agentrig/lint.yaml` добавлено исключение `Ledger/Archive/Backlog` из
`directory-size` с `# DECISION: D056`; создан [Decisions/056.md](Decisions/056.md);
добавлена строка D056 в [Decisions.md](Decisions.md). Архив Backlog сейчас 11 карточек
(>10 — предупреждение, растёт к ошибке 15). Живой Plan пуст.

## Verification

Коммит-гейт (build, lint, memory, rustfmt, clippy, rust-test, ruff/ruff-format/mypy,
pytest, typos, vulture). lint — directory-size по `Ledger/Archive/Backlog` больше не
предупреждает; memory-check — индекс решений и деталь D056 согласованы.

## Blockers

Нет.

## Next action

Закоммитить B3 через гейт и влить ветку (merge через scratch с PATH). Затем C1: индекс
норм «хорошего состояния» со ссылками на их записи (без пересказа), с проверкой, что
memory-check терпит новый файл в `Ledger/`. Затем B023, затем B024.
