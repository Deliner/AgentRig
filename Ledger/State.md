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

- VAC 1 `e4295d6`: выбор P071–P082 в Plan; завершённая история выбора P017–P070 заменена ссылкой на ревизию
  `c1935e4`, иначе Plan.md превышал 500 строк.
- VAC 2: B074 — `conditions.rs` принимает `feature = "…"` в том числе внутри `all`/`any`/`not`; тест
  `test_architecture_attributes.py` для модуля под фичей ожидает полный анализ и находку цикла.
- Проба на копии voxel-rust `voxel-lint2`: находки «conditional expansion» на `src/core/lib.rs:18` и `:21` исчезли,
  «incomplete» 4727 → 4705; корень `src/core/lib.rs` всё ещё не анализируется из-за
  `include!(concat!(env!("OUT_DIR"), …))` в `src/schema/mod.rs` — вне границ B074, кандидат в новую карточку.

## Verification

- 2026-11-07: `just feature-merge` ветки `feature/backlog-research-cards` — гейт зелёный, слито в master.

## Blockers

Нет.

## Next action

Закоммитить выбор P071–P082, затем реализовать B074 AC1–AC2. Свободные ID: D113+, I022+, P083+, B075+ (B066 и D111
зарезервированы, D112 обещан B069).
