# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P075 завершены и слиты в master `edc76fa`. Активна **P076** (B070 blocking-in-async), фаза 1.
Исследования вне Ledger завершены: `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`;
их итог станет новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/blocking-in-async`
Revision: `edc76fac39d902170ab26c4a315bb85c8eb6da0c`

База — master `edc76fa`, слияние `feature/guard-across-await`. Этот снимок входит в коммит VAC 1 фазы 1.

Ветки `feature/guard-across-await`, `feature/panic-surface`, `feature/discarded-variants`, `feature/self-hygiene`,
`feature/feature-cfg`, `feature/backlog-research-cards`, `feature/delivery-repairs`, `feature/state-refresh` и
`feature/init-probe` сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- VAC 1, фаза 1: решение [D118](Decisions/118.md). Инертный вид `BlockingInAsync`, дескриптор
  `blocking-in-async` с уровнем warning, детектор `languages/rust/blocking_async.rs`; общий модуль
  `languages/rust/async_context.rs` задаёт границу области для обоих async-правил, `guard_await.rs` его использует.
  Тесты обоих правил объединены в `languages/tests/async_code.rs`, каталог тестов остаётся в лимите 15 записей.
- Замер на корпусе r8: 565 Rust-файлов с `async` и `sleep` или `block_on`, 6 находок, все по критерию AC1; 4 похожи
  на ошибки (rustdesk `ipc.rs:2272`, zed `predict.rs:792`, `repair.rs:511`, `server.rs:1174`), 2 намеренные
  (zed `db.rs:254` тестовая поддержка, `hang_detection.rs:50` отладочное зависание). Для замера вид временно
  подключался в каталог и обработчик, затем отключён.
- Осталось: коммит VAC 1, мид-бранч пин, фаза 2 — каталог `ALL`, `HANDLERS`, правило `.agentrig/lint.yaml` с
  `level: warning`, навык `await-not-block` и его ассет, список `decided`, Delivery, ротация P076/B070, финальный
  пин, `just feature-merge`.

## Verification

- 2026-11-07, рабочее дерево до коммита VAC 1: `just candidate lint` чист; `just run rust-test -- structural_lint`
  102 passed, включая 4 теста `async_code`.

## Blockers

Нет.

## Next action

Закоммитить VAC 1 через pre-commit, затем мид-бранч пин и фаза 2 P076. Свободные ID: D119+, I022+, P083+, B075+
(B066 и D111 зарезервированы, D112 обещан B069).
