# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P079 завершены и слиты в master, последняя — `680cbb4`. **P080** (B073 embedded-text) активна:
фаза 1 двухфазной поставки нового вида правила (D051). Дальше P081 (B069), P082 (B064), затем новые карточки Backlog
из `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md` по уже данному согласию пользователя.

## Workspace

Branch: `feature/embedded-text`
Revision: `680cbb459e0120e74f8275616a8c3941ce5c5280`

База — master `680cbb4`. Этот снимок входит в VAC фазы 1. Ветки `feature/file-fan-out`, `feature/boolean-expression`
и прежние, а также `feature/init-probe`, сохранены. macOS: в PATH `just write`/`just run`/`just check`/
`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- Фаза 1 P080, VAC в рабочем дереве: вид `EmbeddedText` в `rules/kind.rs` и дескриптор в `rules/descriptor.rs`
  (ошибка выше 20 строк, исключение тестов), вне `catalog::ALL` и `HANDLERS`; замер `literal` в
  `languages/rust/mod.rs`; тесты в `languages/tests/mod.rs` с общим помощником `line_actuals`; решение
  [D122](Decisions/122.md).
- Ради лимитов размера: `Parameters` вынесен в `rules/parameters.rs`, сигнатуры-кортежи — в
  `languages/rust/tuple_record.rs`.
- Замер корпуса r8 без bun: литералов длиннее 20 строк 149, все в выборке — данные; в AgentRig один литерал на 4 строки.

## Verification

- 2026-11-07, рабочее дерево фазы 1: `cargo test -- structural_lint::languages::tests` — 61 passed; clippy
  `-D warnings` чист; `just candidate lint --root .` без находок.

## Blockers

Нет.

## Next action

Коммит VAC фазы 1, затем мид-бранч пин D051 и фаза 2: активация в `catalog::ALL` и `HANDLERS`, навык
`extract-embedded-text`, правило в `.agentrig/lint.yaml`, `decided` в `test_config_check.py`, проверка AC1–AC2 на
потребителе, Delivery и ротация. Свободные ID: D123+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112 обещан
B069).
