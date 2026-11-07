# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P074 завершены и слиты в master `2a52e3e`. **P075** (B050 guard-across-await) завершена и в
архиве; ветка ждёт финального пина и слияния. Следующая — P076 (B070 blocking-in-async).
Исследования вне Ledger завершены: `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`;
их итог станет новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/guard-across-await`
Revision: `1f023eab22257246c38e99a0b353b17cf6644ffe`

База — master `2a52e3e`. VAC 1 `1f023ea`, затем мид-бранч пин `stable.txt` = `1f023ea` с выполненным
`just bootstrap`; этот снимок входит в коммит VAC 2.

Ветки `feature/panic-surface`, `feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`,
`feature/backlog-research-cards`, `feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe`
сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с
cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- VAC 2, фаза 2: вид `GuardAcrossAwait` подключён в `catalog::ALL` и обработчик Rust, правило
  `guard-across-await` с `level: warning` в `.agentrig/lint.yaml`, навык `ownership/release-the-guard` и его ассет,
  список `decided` в `test_config_check.py`; Delivery в [Archive/Plan/075.md](Archive/Plan/075.md), P075 и B050 в
  архиве.
- VAC 1 `1f023ea`, фаза 1: решение [D117](Decisions/117.md), дескриптор с уровнем по умолчанию, детектор
  `languages/rust/guard_await.rs`, тесты; `languages` стал составным с частями `rust/` и `measure/`.

## Verification

- 2026-11-07: гейт VAC 1 `1f023ea` зелёный — rust-test 124 passed, test 383 passed, остальные проверки PASS.
- 2026-11-07, рабочее дерево VAC 2 до коммита: `just candidate lint` — код 0; `just config-check` и
  `just lint-config-check` — валидно; `just run rust-test -- structural_lint` — 100 passed; потребитель после
  `just candidate init` получает `WARNING [guard-across-await]` с `--warnings`.

## Blockers

Нет.

## Next action

Закоммитить VAC 2, затем финальный пин D051 и `just feature-merge` ветки `feature/guard-across-await`; далее P076
(B070). Свободные ID: D118+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112 обещан B069).
