# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P076 завершены и слиты в master `1d846be`. **P077** (B068 ord-eq-mismatch) доставлена на ветке,
VAC 2 `4709a23` закоммичен, финальный пин в этом коммите; осталось слияние. Исследования вне Ledger завершены:
`agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`; их итог станет новыми карточками Backlog
по уже данному согласию пользователя.

## Workspace

Branch: `feature/ord-eq-mismatch`
Revision: `4709a23fec2d4359130dfea6ee05c17af06abc61`

База — master `1d846be`. VAC 1 `37cf4f0`, мид-бранч пин `f31cb50`, VAC 2 `4709a23`; stable.txt указывает на
`4709a23`, установлен через `just bootstrap`. Этот снимок входит в коммит финального пина.

Ветки `feature/blocking-in-async`, `feature/guard-across-await`, `feature/panic-surface`,
`feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`, `feature/backlog-research-cards`,
`feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe` сохранены. macOS: в PATH `just write`/
`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- P077 фаза 1 (VAC 1 `37cf4f0`): решение [D119](Decisions/119.md), инертный вид `OrdEqMismatch` уровня error,
  детектор `languages/rust/ord_equality.rs`, тесты `languages/tests/ord_equality.rs`.
- P077 фаза 2 (VAC 2 `4709a23`): вид в каталоге `ALL` и `HANDLERS`, правило `ord-eq-mismatch` в `.agentrig/lint.yaml`, навык
  `signature/align-order-with-equality` с симлинком и ассетом, назначение группы `signature` расширено, список
  `decided`. Delivery в [Archive/Plan/077.md](Archive/Plan/077.md); P077 и B068 перенесены в архив, ссылки D119
  обновлены.
- Финальный пин: stable.txt = `4709a23`, bootstrap выполнен, `agentrig 0.3.0`.
- Осталось по P077: `just feature-merge`.

## Verification

- 2026-11-07, гейт pre-commit VAC 1 `37cf4f0`: все проверки PASS, rust-test 119 passed, test 216 passed.
- 2026-11-07, гейт pre-commit VAC 2 `4709a23`: все проверки PASS, rust-test 144 passed, test 331 passed.
- Рабочее дерево VAC 2 до коммита: `just config-check` и `just lint-config-check` валидны; `just candidate lint`
  чист. Потребитель после `just candidate init --language rust` получает правило уровня error и навык;
  `just candidate lint` даёт `ERROR [ord-eq-mismatch] Project/src/lib.rs:6 (Dependency: equality-only version)`.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/ord-eq-mismatch`; дальше P078 (B071
boolean-expression) на новой ветке. Свободные ID: D120+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112
обещан B069).
