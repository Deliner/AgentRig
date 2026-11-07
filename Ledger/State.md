# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P073 завершены и слиты в master `75b2689`. **P074** (B048 panic-surface-extension) активна:
VAC 1 — первая фаза двухфазной поставки нового вида `unwind-justification` вместе с расширениями AC1 и AC2.
Исследования вне Ledger завершены: `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`;
их итог станет новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/panic-surface`
Revision: `75b2689890862c556dc15588d3026a3eb47f9410`

База — master `75b2689`, слияние `feature/discarded-variants`. Этот снимок входит в VAC 1 первой фазы.

Ветки `feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`, `feature/backlog-research-cards`,
`feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe` сохранены. macOS: в PATH `just write`/
`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; поставка —
`just candidate <команда> --root <каталог>`.

## Progress

- VAC 1, фаза 1: решение [D115](Decisions/115.md). AC1 — `assertion_macro` в `structural_lint/languages/rust.rs`
  считает `assert*!` под `cfg(debug_assertions)` или в ветке `if cfg!(debug_assertions)` отладочной проверкой;
  обход `cfg` в `test_context.rs` параметризован флагом. AC2 — `panic_call` ловит `unwrap_err`, `expect_err` и
  `resume_unwind`. AC3 — инертный вид `UnwindJustification`, детектор `unwind_catch.rs` и дескриптор; навыки
  `firm-up-assertion` и `handle-panic` называют новые формы; тесты `languages/tests/panic_surface.rs`.

## Verification

- 2026-11-07: `just run rust-test -- structural_lint::languages` — 46 passed, из них `panic_surface` 3 passed;
  `just candidate lint` на репозитории — код 0.

## Blockers

Нет.

## Next action

Коммит VAC 1, мид-бранч пин D051 и `just bootstrap`; затем фаза 2: каталог, обработчик в реестре, правило
`.agentrig/lint.yaml`, навык `justify-unwind` и его поставочный ассет. Свободные ID: D116+, I022+, P083+, B075+
(B066 и D111 зарезервированы, D112 обещан B069).
