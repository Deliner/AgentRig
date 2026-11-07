# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071 (B074) и P072 (B051) завершены и слиты. Активна **P073** ([B047](Backlog/047.md)
discarded-result-variants). Исследования вне Ledger завершены: `agentrig-research/clippy-triage.md` и
`agentrig-research/agent-errors.md`; их итог станет новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/discarded-variants`
Revision: `edd3715d5bea671df8597e5bf7fa5e8f3ddd9ae1`

База — master `edd3715`, слияние `feature/self-hygiene`. VAC 1 этой ветки — этот коммит.

Ветки `feature/self-hygiene`, `feature/feature-cfg`, `feature/backlog-research-cards`, `feature/delivery-repairs`,
`feature/state-refresh` и `feature/init-probe` сохранены. macOS: в PATH `just write`/`just run`/`just check`/
`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; поставка —
`just candidate <команда> --root <каталог>`.

## Progress

- VAC 1: B047 AC1–AC2 — решение [D114](Decisions/114.md); `discarded_method` в `structural_lint/languages/rust.rs`
  ловит голые инструкции с последним вызовом `.err()`, `.map`, `.and_then`, `.map_or`, `.map_or_else`; дескриптор,
  навык `handle-discarded` и назначения его каталогов называют новые формы; тест
  `rust_flags_discarded_option_combinator_and_err_statements`.

## Verification

- 2026-11-07: `just run rust-test -- discarded` — 2 passed; `just candidate lint` на репозитории — код 0.

## Blockers

Нет.

## Next action

После зелёного гейта VAC 1 — финальный VAC с Delivery и ротацией P073/B047, пин D051, `just feature-merge`; затем
P074 (B048). Свободные ID: D115+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112 обещан B069).
