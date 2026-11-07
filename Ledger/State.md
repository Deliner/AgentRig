# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P077 завершены и слиты в master, последняя — `2ddd7c8`. **P078** (B071 boolean-expression)
активна: фаза 1 в коммите VAC 1. Исследования вне Ledger завершены: `agentrig-research/clippy-triage.md` и
`agentrig-research/agent-errors.md`; их итог станет новыми карточками Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/boolean-expression`
Revision: `2ddd7c8ffcc0aab47b5c839a4394d893d6bf3ef0`

База — master `2ddd7c8`, слияние P077; этот снимок входит в коммит VAC 1.

Ветки `feature/ord-eq-mismatch`, `feature/blocking-in-async`, `feature/guard-across-await`, `feature/panic-surface`,
`feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`, `feature/backlog-research-cards`,
`feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe` сохранены. macOS: в PATH `just write`/
`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- P078 фаза 1 (VAC 1): решение [D120](Decisions/120.md), инертный пороговый вид `BooleanExpression`, ошибка выше 3;
  измерение `boolean_expression` в `languages/measure/mod.rs`, вызовы из Rust и Python; три теста в
  `languages/tests/mod.rs`.
- Замер на корпусе r8 без bun: 1 357 из 21 136 выражений длиннее трёх частей; в самом AgentRig 12 мест в Rust:
  `command_runner/registry/check.rs:99`, `project_memory/evidence/mod.rs:210` и `:215`, `shared/vcs/export.rs:43`,
  `shared/vcs/git.rs:15`, `architecture/resolution/rust/macros.rs:32` и `:47`, `resolution/rust/paths.rs:141`,
  `architecture/source/python.rs:48`, `structural_lint/config/mod.rs:145`, `languages/rust/mod.rs:227`,
  `rules/architecture.rs:147`.
- Осталось по P078: мид-бранч пин; фаза 2 — `ALL`, `HANDLERS` для Rust и Python, ремонт 12 мест, правило в
  `.agentrig/lint.yaml`, навык `control-flow/split-boolean-expression` с симлинком и ассетом, список `decided`,
  Delivery, ротация P078 и B071, финальный пин, `just feature-merge`.

## Verification

- 2026-11-07, гейт pre-commit VAC 2 P077 `4709a23`: все проверки PASS, rust-test 144 passed, test 331 passed;
  слияние `2ddd7c8`.
- Рабочее дерево VAC 1 P078 до коммита: `cargo test -- operands parentheses` — 3 passed. Образец
  `just candidate lint` с временной регистрацией вида даёт ожидаемые числа частей для Rust и Python.

## Blockers

Нет.

## Next action

Закоммитить VAC 1 P078 через pre-commit, затем мид-бранч пин с bootstrap. Свободные ID: D121+, I022+, P083+, B075+
(B066 и D111 зарезервированы, D112 обещан B069).
