# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P081 завершены и слиты в master, последняя — `0bf9d8d`. **P082** (B064 ratchet-baseline)
доставлена одним VAC в этом коммите; осталось записать пин и слить. После неё Plan и Backlog пусты; дальше новые
карточки Backlog из `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md` по уже данному
согласию пользователя.

## Workspace

Branch: `feature/ratchet-baseline`
Revision: `0bf9d8d6be86c5142af915c629a222fa5cada6cb`

База — master `0bf9d8d`. Ветки `feature/grammar-fork` и прежние, а также `feature/init-probe`, сохранены. macOS: в
PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- VAC, в этом коммите: `structural_lint/baseline.rs` — запись и применение baseline; точка входа `agentrig-lint`
  переехала из `structural_lint/standalone.rs` в `command_line/lint.rs` рядом с `main.rs`, иначе в каталоге линта
  16 записей при лимите 15. Команда `lint-baseline` в `command_line/main.rs`, рецепт в `just/lint.just` и в шаблоне `init`, предложение в
  `init` и мастере, тесты `structural_lint/tests/test_lint_baseline.py` и в `initialization/tests/test_layout.py`,
  README и руководство по настройке, решение [D123](Decisions/123.md). Delivery в
  [Archive/Plan/082.md](Archive/Plan/082.md); P082 и B064 перенесены в архив.

## Verification

- 2026-11-07, рабочее дерево до коммита: clippy `--all-targets -D warnings` чист; pytest
  `test_lint_baseline.py` и `project_setup/initialization/tests` — 38 passed.
- Потребитель в scratchpad `p082/staged.sh`: `check --staged --only lint` — 1 без baseline и с baseline только в
  рабочем дереве, 0 после `git add` baseline.

## Blockers

Нет.

## Next action

Коммит VAC через гейт; пин D051 — `git rev-parse HEAD > tooling/distribution/stable.txt && just bootstrap`, коммит
пина со State; `just feature-merge`. Затем черновики новых карточек Backlog из исследований clippy и ошибок агентов,
каждая показывается в чате при сохранении. Свободные ID: D124+, I022+, P083+, B075+ (B066 и D111 зарезервированы).
