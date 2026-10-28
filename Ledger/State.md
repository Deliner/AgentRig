# State

## Focus

На 2026-10-28 выполняется согласованный набор улучшений (одна выборка, автономно, D032)
в порядке: B3 → C1 → B023 → [B024](Backlog/024.md). Выполнено и влито: B023/B024 в
Backlog; B3 (D056); C1 ([Norms.md](Norms.md)). B023 завершён как [P032](Archive/Plan/032.md):
TC2/TC4 оставлены нормой (D057). Текущий шаг — финальный VAC B023 (архивация), затем B024.

## Workspace

Branch: feature/test-visibility-norm

Revision: a02dde4 (VAC1 — P032 + D057); этот VAC — архивация P032/B023.

Машина — macOS (Darwin 25.5.0). B023 — вывод-норма, Project не менялся, пин по D051 не
нужен. Merge запускать через `just scratch -- '... tooling/worker/run feature-merge'` с
PATH. Осталась ветка B024 (спайк по покрытию): меняет ли Project — решится при реализации.

## Progress

B023/P032 завершён и переносится в Archive: [P032](Archive/Plan/032.md) с Delivery и
сверкой K1–K3; карточка [B023](Archive/Backlog/023.md) — в Archive/Backlog; живые строки
Plan/Backlog сняты (живой Plan пуст, в Backlog остаётся только B024). Решение
[D057](Decisions/057.md): TC2/TC4 — норма рядом с TC1 (D055), без правила/инварианта.

## Verification

Коммит-гейт (build, lint, memory, rustfmt, clippy, rust-test, ruff/ruff-format/mypy,
pytest, typos, vulture); memory-check — соответствие индексов и деталей после переноса
карточек в Archive/Plan и Archive/Backlog.

## Blockers

Нет.

## Next action

Закоммитить архивный VAC B023 через гейт (git rm живых Plan/032.md и Backlog/023.md),
влить ветку (merge через scratch с PATH, без пина). Затем B024: промоутить в Plan,
спайк по сбору per-test покрытия (Rust llvm-cov, Python coverage.py) и проверке уровня
размещения; по итогам — правило либо норма с записью причины; пин+bootstrap, если
менялся Project; архивировать B024.
