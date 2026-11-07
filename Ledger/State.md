# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog — B074, B051, B047, B048,
B050, B070, B068, B071, B072, B073, B069, B064 — выполняются автономно по одной за ветку. **P071** (B074
feature-cfg) завершена и перенесена в архив вместе с карточкой; следующая — P072. Параллельно вне Ledger идут
исследования: разбор clippy и разбор реальных ошибок агентов; их итог станет новыми карточками Backlog по уже данному
согласию пользователя.

## Workspace

Branch: `feature/feature-cfg`
Revision: `804cc0d285b59aaf22bf80b79bdad17264f4b091`

База — master `c1935e4`. Этот снимок входит в финальный VAC ветки (Delivery P071, ротация P071/B074).

Ветки `feature/backlog-research-cards`, `feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe`
сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с
cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
поставка — `just candidate <команда> --root <каталог>`.

## Progress

- VAC 1 `e4295d6`: выбор P071–P082 в Plan; завершённая история выбора P017–P070 заменена ссылкой на ревизию
  `c1935e4`, иначе Plan.md превышал 500 строк.
- VAC 2 `804cc0d`: B074 AC1–AC2 — `conditions.rs` принимает `feature = "…"` в том числе внутри `all`/`any`/`not`.
- Финальный VAC: Delivery в [Archive/Plan/071.md](Archive/Plan/071.md), P071 и B074 перенесены в архив.
- Проба на копии voxel-rust: корень `src/core/lib.rs` всё ещё не анализируется из-за
  `include!(concat!(env!("OUT_DIR"), …))` в `src/schema/mod.rs` — вне границ B074, кандидат в новую карточку.

## Verification

- 2026-11-07: гейт VAC 2 `804cc0d` зелёный — rust-test 105 passed, test 216 passed, остальные проверки PASS.

## Blockers

Нет.

## Next action

Закоммитить финальный VAC, записать голову ветки в `tooling/distribution/stable.txt` с `just bootstrap`, закоммитить
пин D051, `just feature-merge`; затем ветка `feature/self-hygiene` для P072 (B051). Свободные ID: D113+, I022+,
P083+, B075+ (B066 и D111 зарезервированы, D112 обещан B069).
