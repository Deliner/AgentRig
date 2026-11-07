# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071 (B074) завершена и слита. **P072** (B051 self-hygiene) завершена, в архиве. Разбор clippy
вне Ledger завершён (`agentrig-research/clippy-triage.md`), разбор реальных ошибок агентов ещё идёт; их итог станет
новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/self-hygiene`
Revision: `61dd2a28d3802cef0885762acf846e3389465612`

База — master `ff23ad9`, слияние `feature/feature-cfg`. VAC 1 `c7e6842`, финальный VAC `61dd2a2`; этот снимок входит
в коммит пина D051: `stable.txt` = `61dd2a2`, `just bootstrap` выполнен.

Ветки `feature/feature-cfg`, `feature/backlog-research-cards`, `feature/delivery-repairs`, `feature/state-refresh` и
`feature/init-probe` сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; поставка —
`just candidate <команда> --root <каталог>`.

## Progress

- Финальный VAC `61dd2a2`: Delivery в [Archive/Plan/072.md](Archive/Plan/072.md), P072 и B051 в архиве.
- VAC 1 `c7e6842`: B051 AC1–AC3 — решение-преемник [D113](Decisions/113.md); три best-effort сайта `libc::signal`/`libc::kill`
  в `jobs/process.rs` ссылаются на D113 вместо D066/D095, `restore_default_signals` сохраняет ссылку D095;
  оба преобразования PID там идут через `i32::try_from`; у zero-fill в `identity/macos.rs::info<T>` ссылка D113 с
  предусловием на `T`.

## Verification

- 2026-11-07: `just feature-merge` ветки `feature/feature-cfg` — гейт зелёный, слито в master `ff23ad9`.
- 2026-11-07: гейт VAC 1 `c7e6842` зелёный — rust-test 15 passed, test 219 passed, остальные проверки PASS.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/self-hygiene`; затем
P073 (B047). Свободные ID: D114+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112 обещан B069).
