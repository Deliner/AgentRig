# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P076 завершены и слиты в master `1d846be`. Активна **P077** (B068 ord-eq-mismatch), фаза 1.
Исследования вне Ledger завершены: `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`;
их итог станет новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/ord-eq-mismatch`
Revision: `1d846bec440cfec7ce996b72dbf9cdd4b70f4022`

База — master `1d846be`, слияние `feature/blocking-in-async`.

Этот снимок входит в коммит VAC 1 фазы 1.

Ветки `feature/blocking-in-async`, `feature/guard-across-await`, `feature/panic-surface`,
`feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`, `feature/backlog-research-cards`,
`feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe` сохранены. macOS: в PATH `just write`/
`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- VAC 1, фаза 1: решение [D119](Decisions/119.md). Инертный вид `OrdEqMismatch`, дескриптор `ord-eq-mismatch` с
  уровнем error, детектор `languages/rust/ord_equality.rs` вызывается один раз на файл у корня `source_file`; тесты
  `languages/tests/ord_equality.rs`, шесть функций в пределах 40 строк.
- Замер на корпусе r8: 125 Rust-файлов с ручным `Ord`/`PartialOrd`, 10 находок, все нарушают контракт std: 6 известных
  сайтов исследования и 4 новых, nushell `nu-experimental/src/lib.rs:229`, zed `user.rs:80`, `tab_stop.rs:52`,
  `syntax_map.rs:2056`. Пропуск uv `request.rs:843` ожидаем. Для замера вид временно подключался в каталог и
  обработчик, затем отключён.
- Осталось: коммит VAC 1, мид-бранч пин, фаза 2 — каталог `ALL`, `HANDLERS`, правило `.agentrig/lint.yaml`, навык
  `signature/align-order-with-equality` с симлинком и ассетом, список `decided`, Delivery, ротация P077/B068,
  финальный пин, `just feature-merge`.

## Verification

- 2026-11-07, рабочее дерево до коммита VAC 1: `just run rust-test -- ord_equality` 6 passed;
  `just run rust-test -- structural_lint` 108 passed до разбиения тестов; `just candidate lint` чист.

## Blockers

Нет.

## Next action

Закоммитить VAC 1 через pre-commit, затем мид-бранч пин и фаза 2 P077. Свободные ID: D120+, I022+, P083+, B075+
(B066 и D111 зарезервированы, D112 обещан B069).
