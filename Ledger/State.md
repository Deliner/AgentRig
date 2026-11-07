# State

## Focus

Программа **P071 → P082** (/goal пользователя 2026-11-07): все открытые карточки Backlog выполняются автономно по
одной за ветку. P071–P078 завершены и слиты в master, последняя — `25a5a97`. **P079** (B072 file-fan-out) доставлена
на ветке одним VAC `33a255b` без нового вида правила; финальный пин в этом коммите, осталось слияние. Дальше P080
(B073), P081 (B069), P082 (B064), затем новые карточки Backlog из `agentrig-research/clippy-triage.md` и
`agentrig-research/agent-errors.md` по уже данному согласию пользователя.

## Workspace

Branch: `feature/file-fan-out`
Revision: `33a255b5e761b6e53ee4a6df795f688b3b927d59`

База — master `25a5a97`. VAC `33a255b`; stable.txt указывает на `33a255b`, установлен через `just bootstrap`. Этот
снимок входит в коммит финального пина. Ветки
`feature/boolean-expression`, `feature/ord-eq-mismatch`, `feature/blocking-in-async`, `feature/guard-across-await`,
`feature/panic-surface`, `feature/discarded-variants`, `feature/self-hygiene`, `feature/feature-cfg`,
`feature/backlog-research-cards`, `feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe`
сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — коммиты и вызовы с
cargo через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && cd /Users/admin/projects/AgentRig && …'`;
`cargo fmt --manifest-path Project/Cargo.toml`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- P079 VAC: проверка `fan_out` в `structural_lint/architecture/graph.rs`, вызов в `runner/mod.rs`, настройка в
  `rules/architecture.rs`, решение [D121](Decisions/121.md), тесты `test_architecture_fan_out.py`, ремонт широкого
  файла в навыке `refactor-large-directory`, строка в README архитектуры.
- Свои три широких файла разделены: `change_delivery/gate/check.rs`, `structural_lint/evaluation.rs`,
  `command_line/project.rs`, `config_check` перенесён в `project_setup/doctor.rs`; ссылки D036 и D072 перенесены в
  `evaluation.rs`.
- Delivery в [Archive/Plan/079.md](Archive/Plan/079.md); P079 и B072 перенесены в архив.

## Verification

- 2026-11-07, гейт pre-commit VAC `33a255b`: все проверки PASS, rust-test 122 passed, test 394 passed.
- Рабочее дерево VAC до коммита: `just candidate lint --root .` — код 0; `test_architecture_fan_out.py` — 5 passed;
  `just config-check` и `just lint-config-check` валидны.
- Пин: stable.txt = `33a255b`, `just bootstrap` установил `agentrig 0.3.0`.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/file-fan-out`; дальше P080 (B073 embedded-text) на новой ветке. Свободные ID: D122+, I022+, P083+, B075+ (B066 и D111 зарезервированы, D112
обещан B069).
