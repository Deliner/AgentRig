# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P073 завершены и слиты в master `75b2689`. **P074** (B048 panic-surface-extension) завершена
и в архиве; ветка ждёт финального пина и слияния. Следующая — P075.
Исследования вне Ledger завершены: `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`;
их итог станет новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/panic-surface`
Revision: `481eb3707ed631cde491b189b0e470b465ef0479`

База — master `75b2689`, слияние `feature/discarded-variants`. VAC 1 `a313bf2`, мид-бранч пин `f7b0246`, VAC 2
`481eb37`; этот снимок входит в коммит финального пина D051: `stable.txt` = `481eb37`, `just bootstrap` выполнен.

Ветки `feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`, `feature/backlog-research-cards`,
`feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe` сохранены. macOS: в PATH `just write`/
`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`; поставка —
`just candidate <команда> --root <каталог>`.

## Progress

- VAC 2 `481eb37`, фаза 2: вид подключён в каталог, обработчик Rust, правило `.agentrig/lint.yaml`, навык `justify-unwind` и
  его ассет; решение [D116](Decisions/116.md) исключает реестр `.agentrig/lint.yaml` из `file-size`, потому что
  он перерос 500 строк; Delivery в [Archive/Plan/074.md](Archive/Plan/074.md), P074 и B048 в архиве.
- VAC 1 `a313bf2`, фаза 1: решение [D115](Decisions/115.md). AC1 — `assertion_macro` в `structural_lint/languages/rust.rs`
  считает `assert*!` под `cfg(debug_assertions)` или в ветке `if cfg!(debug_assertions)` отладочной проверкой;
  обход `cfg` в `test_context.rs` параметризован флагом. AC2 — `panic_call` ловит `unwrap_err`, `expect_err` и
  `resume_unwind`. AC3 — инертный вид `UnwindJustification`, детектор `unwind_catch.rs` и дескриптор; навыки
  `firm-up-assertion` и `handle-panic` называют новые формы; тесты `languages/tests/panic_surface.rs`.

## Verification

- 2026-11-07: гейт VAC 1 `a313bf2` зелёный — rust-test 109 passed, test 288 passed, остальные проверки PASS.
- 2026-11-07: гейт VAC 2 `481eb37` зелёный — rust-test 134 passed, test 331 passed, остальные проверки PASS;
  потребитель после `just candidate init` получает находку `unwind-justification`.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/panic-surface`; далее P075 (B050). Свободные ID: D117+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112 обещан B069).
