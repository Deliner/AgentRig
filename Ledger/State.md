# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P074 завершены и слиты в master `2a52e3e`. Активна **P075** (B050 guard-across-await), фаза 1.
Исследования вне Ledger завершены: `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`;
их итог станет новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/guard-across-await`
Revision: `2a52e3efce4feda7398fd2c86ce67797aa2126fe`

База — master `2a52e3e`, слияние `feature/panic-surface`. Этот снимок входит в коммит VAC 1 фазы 1.

Ветки `feature/panic-surface`, `feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`,
`feature/backlog-research-cards`, `feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe`
сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с
cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- VAC 1, фаза 1: решение [D117](Decisions/117.md). Инертный вид `GuardAcrossAwait` в `rules/kind.rs`, дескриптор
  `guard-across-await` с уровнем по умолчанию warning: `Parameters::Level` получил поле `default`, у всех прежних
  видов оно `Error`. Детектор `structural_lint/languages/rust/guard_await.rs`, тесты
  `languages/tests/guard_await.rs`.
- Каталог `languages` превысил 15 записей и стал составным: диспетчер и детекторы Rust перенесены в часть
  `languages/rust/`, запись `Measurement` и общие для Rust и Python измерения — в часть `languages/measure/`.
  Вариант с `rust.rs` и дочерними модулями отвергнут: clippy `self_named_module_files` требует `mod.rs`. Бюджеты входящих связей подняты: `rules` 7 → 9, `diagnostics` 10 → 11. Ссылки индекса
  решений на перенесённые детекторы и ссылка D017 обновлены.
- Осталось: коммит VAC 1, мид-бранч пин, фаза 2 — каталог `ALL`, `HANDLERS`, правило `.agentrig/lint.yaml` с
  `level: warning`, навык `release-the-guard` и его ассет, список `decided` в `test_config_check.py`, Delivery,
  ротация P075/B050, финальный пин, `just feature-merge`.

## Verification

- 2026-11-07, рабочее дерево до коммита VAC 1: `just candidate lint` чист; `just run rust-test -- structural_lint`
  100 passed, включая оба теста `guard_await`.

## Blockers

Нет.

## Next action

Закоммитить VAC 1 через pre-commit, затем мид-бранч пин `stable.txt` и фаза 2 P075. Свободные ID: D118+, I022+,
P083+, B075+ (B066 и D111 зарезервированы, D112 обещан B069).
