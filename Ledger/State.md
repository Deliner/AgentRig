# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P079 завершены и слиты в master, последняя — `680cbb4`. **P080** (B073 embedded-text) активна:
фаза 1 двухфазной поставки нового вида правила (D051). Дальше P081 (B069), P082 (B064), затем новые карточки Backlog
из `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md` по уже данному согласию пользователя.

## Workspace

Branch: `feature/embedded-text`
Revision: `c0ffe2e88344ccf485e2043481efd1e550e37e76`

База — master `680cbb4`. VAC фазы 1 `c0ffe2e`; stable.txt указывает на `c0ffe2e`, установлен через `just bootstrap`.
Этот снимок входит в коммит мид-бранч пина. Ветки `feature/file-fan-out`, `feature/boolean-expression`
и прежние, а также `feature/init-probe`, сохранены. macOS: в PATH `just write`/`just run`/`just check`/
`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- Фаза 1 P080, VAC `c0ffe2e`: вид `EmbeddedText` в `rules/kind.rs` и дескриптор в `rules/descriptor.rs`
  (ошибка выше 20 строк, исключение тестов), вне `catalog::ALL` и `HANDLERS`; замер `literal` в
  `languages/rust/mod.rs`; тесты в `languages/tests/mod.rs` с общим помощником `line_actuals`; решение
  [D122](Decisions/122.md).
- Ради лимитов размера: `Parameters` вынесен в `rules/parameters.rs`, сигнатуры-кортежи — в
  `languages/rust/tuple_record.rs`.
- Замер корпуса r8 без bun: литералов длиннее 20 строк 149, все в выборке — данные; в AgentRig один литерал на 4 строки.

## Verification

- 2026-11-07, гейт pre-commit VAC `c0ffe2e`: все проверки PASS, rust-test 124 passed, test 221 passed. Ссылки
  D088 и D101 перенесены на `rust/tuple_record.rs` после замечания проверки memory.
- Пин: stable.txt = `c0ffe2e`, `just bootstrap` установил `agentrig 0.3.0`.

## Blockers

Нет.

## Next action

Фаза 2: активация в `catalog::ALL` и `HANDLERS`, навык
`extract-embedded-text`, правило в `.agentrig/lint.yaml`, `decided` в `test_config_check.py`, проверка AC1–AC2 на
потребителе, Delivery и ротация. Свободные ID: D123+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112 обещан
B069).
