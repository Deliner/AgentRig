# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P081 завершены и слиты в master, последняя — `0bf9d8d`. **P082** (B064 ratchet-baseline)
доставлена VAC `af7906b`; в этом коммите пин D051, дальше слияние. После неё Plan и Backlog пусты; дальше новые
карточки Backlog из `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md` по уже данному
согласию пользователя.

## Workspace

Branch: `feature/ratchet-baseline`
Revision: `af7906b8f7f9abc1916eb782afda37602ab1107d`

База — master `0bf9d8d`. Ветки `feature/grammar-fork` и прежние, а также `feature/init-probe`, сохранены. macOS: в
PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- VAC `af7906b`: `structural_lint/baseline.rs` — запись и применение baseline; точка входа `agentrig-lint`
  переехала из `structural_lint/standalone.rs` в `command_line/lint.rs` рядом с `main.rs`, иначе в каталоге линта
  16 записей при лимите 15. Команда `lint-baseline` в `command_line/main.rs`, рецепт в `just/lint.just` и в
  шаблоне `init`, предложение в `init` и мастере, тесты `structural_lint/tests/test_lint_baseline.py` и в
  `initialization/tests/test_layout.py`, README и руководство по настройке, решение [D123](Decisions/123.md).
  Delivery в [Archive/Plan/082.md](Archive/Plan/082.md); P082 и B064 перенесены в архив.
- Пин D051, в этом коммите: `tooling/distribution/stable.txt` = `af7906b`, `just bootstrap` установил его.

## Verification

- 2026-11-07, гейт коммита `af7906b`: build, lint, memory, rustfmt, clippy, rust-test 151 passed, ruff-format, ruff,
  mypy, test 339 passed, typos, vulture — PASS; memory выдал ожидаемое WARNING о `grammar.js` под D112.
- Потребитель в scratchpad `p082/staged.sh`: `check --staged --only lint` — 1 без baseline и с baseline только в
  рабочем дереве, 0 после `git add` baseline.
- `just bootstrap` на `af7906b`: код 0, `agentrig 0.3.0`.

## Blockers

Нет.

## Next action

`just feature-merge` на чистой ветке. Затем черновики новых карточек Backlog из исследований clippy и ошибок
агентов, каждая показывается в чате при сохранении. Свободные ID: D124+, I022+, P083+, B075+ (B066 и D111
зарезервированы).
