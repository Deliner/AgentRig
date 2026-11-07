# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog — B074, B051, B047, B048,
B050, B070, B068, B071, B072, B073, B069, B064 — выполняются автономно по одной за ветку. Активна **P071**
([B074](Backlog/074.md) feature-cfg). Параллельно вне Ledger идут два исследования, разбор clippy и разбор реальных
ошибок агентов; их итог станет новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/feature-cfg`
Revision: `c1935e47a3fe0c1b556692aed4f9c81d1a979fbb`

База — master `c1935e4`, слияние `feature/backlog-research-cards`; VAC 1 этой ветки ещё не закоммичен.

Ветки `feature/backlog-research-cards`, `feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe`
сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с
cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
поставка — `just candidate <команда> --root <каталог>`.

## Progress

- VAC 1 этой ветки: выбор P071–P082 в Plan.
- Следующий VAC: B074 — `conditions.rs` принимает `feature = "…"`; тест `test_architecture_attributes.py` ожидает для
  модуля под фичей полный анализ и находку цикла.

## Verification

- 2026-11-07: `just feature-merge` ветки `feature/backlog-research-cards` — гейт зелёный, слито в master.

## Blockers

Нет.

## Next action

Закоммитить выбор P071–P082, затем реализовать B074 AC1–AC2. Свободные ID: D113+, I022+, P083+, B075+ (B066 и D111
зарезервированы, D112 обещан B069).
