# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P077 завершены и слиты в master, последняя — `2ddd7c8`. **P078** (B071 boolean-expression)
доставлена на ветке: VAC 2 `8f674a1`, финальный пин в этом коммите; осталось слияние. Исследования вне Ledger завершены:
`agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`; их итог станет новыми карточками
Backlog по уже данному согласию пользователя.

## Workspace

Branch: `feature/boolean-expression`
Revision: `8f674a181ad25793c7c4fcb7d794da6022fc185a`

База — master `2ddd7c8`. VAC 1 `073c9cf`, мид-бранч пин `9319114`, VAC 2 `8f674a1`; stable.txt указывает на
`8f674a1`, установлен через `just bootstrap`. Этот снимок входит в коммит финального пина.

Ветки `feature/ord-eq-mismatch`, `feature/blocking-in-async`, `feature/guard-across-await`, `feature/panic-surface`,
`feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`, `feature/backlog-research-cards`,
`feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe` сохранены. macOS: в PATH `just write`/
`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- P078 фаза 1 (VAC 1 `073c9cf`): решение [D120](Decisions/120.md), инертный пороговый вид `BooleanExpression`,
  измерение в `languages/measure/mod.rs`, три теста в `languages/tests/mod.rs`.
- P078 фаза 2 (VAC 2 `8f674a1`): вид в `ALL` и `HANDLERS` Rust и Python, правило `boolean-expression` с `error: 3` в
  `.agentrig/lint.yaml`, навык `control-flow/split-boolean-expression` с симлинком и ассетом, ремонт 12
  своих выражений именами групп; тесты не исключены, поэтому правила нет в списке `decided`. Delivery в [Archive/Plan/078.md](Archive/Plan/078.md); P078 и B071
  перенесены в архив, ссылки D120 обновлены.
- Финальный пин: stable.txt = `8f674a1`, bootstrap выполнен, `agentrig 0.3.0`.
- Осталось по P078: `just feature-merge`.

## Verification

- 2026-11-07, гейт pre-commit VAC 1 `073c9cf`: все проверки PASS, rust-test 122 passed, test 216 passed.
- 2026-11-07, гейт pre-commit VAC 2 `8f674a1`: все проверки PASS, rust-test 147 passed, test 384 passed.
- Рабочее дерево VAC 2 до коммита: `just config-check` и `just lint-config-check` валидны; `just candidate lint`
  чист. Потребители после `just candidate init` с `--language rust` и `--language python` получают правило и навык;
  `just candidate lint` даёт `boolean-expression measures 4, exceeds 3` на `Project/src/lib.rs:3` и
  `Project/app.py:3`, выражения из двух и трёх частей молчат.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/boolean-expression`; дальше P079 (B072
file-fan-out) на новой ветке. Свободные ID: D121+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112 обещан
B069).
