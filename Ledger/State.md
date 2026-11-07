# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P076 завершены и слиты в master `1d846be`. **P077** (B068 ord-eq-mismatch) доставлена на ветке,
фаза 2 в коммите VAC 2; осталось финальный пин и слияние. Исследования вне Ledger завершены:
`agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`; их итог станет новыми карточками Backlog
по уже данному согласию пользователя.

## Workspace

Branch: `feature/ord-eq-mismatch`
Revision: `f31cb50c6cfcdc9265e73494091e16bb28fae469`

База — master `1d846be`. VAC 1 `37cf4f0`, мид-бранч пин `f31cb50`; этот снимок входит в коммит VAC 2.

Ветки `feature/blocking-in-async`, `feature/guard-across-await`, `feature/panic-surface`,
`feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`, `feature/backlog-research-cards`,
`feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe` сохранены. macOS: в PATH `just write`/
`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- P077 фаза 1 (VAC 1 `37cf4f0`): решение [D119](Decisions/119.md), инертный вид `OrdEqMismatch` уровня error,
  детектор `languages/rust/ord_equality.rs`, тесты `languages/tests/ord_equality.rs`.
- P077 фаза 2 (VAC 2): вид в каталоге `ALL` и `HANDLERS`, правило `ord-eq-mismatch` в `.agentrig/lint.yaml`, навык
  `signature/align-order-with-equality` с симлинком и ассетом, назначение группы `signature` расширено, список
  `decided`. Delivery в [Archive/Plan/077.md](Archive/Plan/077.md); P077 и B068 перенесены в архив, ссылки D119
  обновлены.
- Осталось по P077: финальный пин stable.txt на голову VAC 2 с bootstrap, `just feature-merge`.

## Verification

- 2026-11-07, гейт pre-commit VAC 1 `37cf4f0`: все проверки PASS, rust-test 119 passed, test 216 passed.
- Рабочее дерево VAC 2 до коммита: `just config-check` и `just lint-config-check` валидны; `just candidate lint`
  чист. Потребитель после `just candidate init --language rust` получает правило уровня error и навык;
  `just candidate lint` даёт `ERROR [ord-eq-mismatch] Project/src/lib.rs:6 (Dependency: equality-only version)`.

## Blockers

Нет.

## Next action

Закоммитить VAC 2 через pre-commit, затем финальный пин с bootstrap и `just feature-merge`; дальше P078 (B071
boolean-expression) на новой ветке. Свободные ID: D120+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112
обещан B069).
