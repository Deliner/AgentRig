# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P075 завершены и слиты в master `edc76fa`. **P076** (B070 blocking-in-async) доставлена на
ветке, фаза 2 в коммите VAC 2; осталось финальный пин и слияние. Исследования вне Ledger завершены:
`agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`; их итог станет новыми карточками Backlog
по уже данному согласию пользователя.

## Workspace

Branch: `feature/blocking-in-async`
Revision: `5ad4e6832c4f9f4a415b435ecd5c6faa1ae68fcc`

База — master `edc76fa`. VAC 1 `2f9be20`, мид-бранч пин `5ad4e68`; этот снимок входит в коммит VAC 2.

Ветки `feature/guard-across-await`, `feature/panic-surface`, `feature/discarded-variants`, `feature/self-hygiene`,
`feature/feature-cfg`, `feature/backlog-research-cards`, `feature/delivery-repairs`, `feature/state-refresh` и
`feature/init-probe` сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- P076 фаза 1 (VAC 1 `2f9be20`): решение [D118](Decisions/118.md), инертный вид `BlockingInAsync`, детектор
  `languages/rust/blocking_async.rs`, общий модуль `languages/rust/async_context.rs`, тесты в
  `languages/tests/async_code.rs`.
- P076 фаза 2 (VAC 2): вид в каталоге `ALL` и `HANDLERS`, правило `blocking-in-async` в `.agentrig/lint.yaml` с
  `level: warning`, навык `references/await-not-block` с симлинком и ассетом, список `decided`. Delivery в
  [Archive/Plan/076.md](Archive/Plan/076.md); P076 и B070 перенесены в архив, ссылки D118 обновлены.
- Осталось по P076: финальный пин stable.txt на голову VAC 2 с bootstrap, `just feature-merge`.

## Verification

- 2026-11-07, рабочее дерево VAC 2 до коммита: `just config-check` и `just lint-config-check` валидны;
  `just candidate lint` чист. Потребитель после `just candidate init --language rust` получает правило и навык;
  `just candidate lint --warnings` даёт `WARNING [blocking-in-async]` на `lib.rs:4 (sleep)` и `lib.rs:8 (block_on)`,
  вызов в замыкании не отмечен, без `--warnings` находки скрыты.

## Blockers

Нет.

## Next action

Закоммитить VAC 2 через pre-commit, затем финальный пин с bootstrap и `just feature-merge`; дальше P077 (B068
ord-eq-mismatch) на новой ветке. Свободные ID: D119+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112 обещан
B069).
