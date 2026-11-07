# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P079 завершены и слиты в master, последняя — `680cbb4`. **P080** (B073 embedded-text) доставлена
на ветке двумя VAC; этот снимок входит в коммит финального пина, осталось слияние. Дальше P081 (B069), P082 (B064),
затем новые карточки Backlog из `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md` по уже
данному согласию пользователя.

## Workspace

Branch: `feature/embedded-text`
Revision: `c38457be036af3a37bf50ef68c290273d04d1cd5`

База — master `680cbb4`. VAC 1 `c0ffe2e`, мид-бранч пин `6af7dff`, VAC 2 `c38457b`; stable.txt указывает на
`c38457b`, установлен через `just bootstrap`. Ветки
`feature/file-fan-out`, `feature/boolean-expression` и прежние, а также `feature/init-probe`, сохранены. macOS: в
PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- VAC 1 `c0ffe2e`: инертный вид `EmbeddedText`, замер `literal` в `languages/rust/mod.rs`, тесты, решение
  [D122](Decisions/122.md); ради лимитов `rules/parameters.rs` и `languages/rust/tuple_record.rs`.
- VAC 2 `c38457b`: вид в `catalog::ALL` и `HANDLERS`, правило `embedded-text` в `.agentrig/lint.yaml`
  (error 20), навык `refactor/extract-embedded-text` с ассетом, `decided` в `test_config_check.py`. Delivery в
  [Archive/Plan/080.md](Archive/Plan/080.md); P080 и B073 перенесены в архив.

## Verification

- 2026-11-07, гейт pre-commit VAC 1 `c0ffe2e`: все проверки PASS, rust-test 124 passed, test 221 passed.
- Гейт pre-commit VAC 2 `c38457b`: все проверки PASS, rust-test 149 passed, test 336 passed.
- Рабочее дерево VAC 2 до коммита: `just candidate lint --root .` — код 0; `just candidate config-check` и
  `just candidate lint-config-check` — валидно; `cargo test --lib` — 149 passed; pytest `config/tests`,
  `project_setup/tests`, `languages/tests` — 53 passed. Потребитель `init --language rust`: находка на литерале в
  21 строку, литерал в 20 строк и литералы в тестах без находок.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/embedded-text`; затем P081
(B069 grammar-fork) на новой ветке. Свободные ID: D123+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112
обещан B069).
