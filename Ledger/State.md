# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07) завершена: все открытые карточки Backlog выполнены по одной
за ветку и слиты в master, последняя — P082 (B064 ratchet-baseline), merge `3a5bcf2`. Пункт 3 цели — новые правила из
разбора clippy и из реальных ошибок агентов: в этом коммите сохранены карточки B075–B078 по уже данному согласию
пользователя, каждая показана в чате при сохранении. Plan пуст; карточки ждут выбора пользователя.

## Workspace

Branch: `feature/rule-cards`
Revision: `3a5bcf239c753c6fc9b41f7d084c2ae94df46f4b`

База — master `3a5bcf2`. Ветки `feature/ratchet-baseline`, `feature/grammar-fork` и прежние, а также
`feature/init-probe`, сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo —
коммиты и вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`.

## Progress

- P082 слита: VAC `af7906b`, пин D051 `562ecfe` (`stable.txt` = `af7906b`, bootstrap прошёл), merge `3a5bcf2`.
- В этом коммите: Backlog B075 (пять линтов clippy с типами для dev-гейта), B076 (код выхода процесса), B077
  (покрытие карты влияния в `config-check`), B078 (методы ФС в политике `disallowed`). Источники —
  `agentrig-research/clippy-triage.md` и `agentrig-research/agent-errors.md`. Не взяты: устаревший размерный литерал
  (самый слабый кандидат разбора, близкий отброшенному `repeatlit`) и формы ниже порога двух исправлений.

## Verification

- `just feature-merge` P082 на `562ecfe`: build, lint, memory, tools, rustfmt, clippy, rust-test 151 passed, ruff,
  mypy, test 339 passed, typos, vulture — PASS.
- 2026-11-07, индекс с карточками до коммита: `check --staged --only memory` — PASS, ожидаемое WARNING о
  `grammar.js` под D112.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/rule-cards`. Дальше — выбор пользователя: какие из B075–B078 брать в Plan и в каком
порядке. Свободные ID: D124+, I022+, P083+, B079+ (B066 и D111 зарезервированы).
