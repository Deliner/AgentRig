# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P130 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B126 о блокирующих правилах на числах
`lint-structure` сохранена, в работу не выбрана.

2026-11-13 пользователь одобрил B125, B126 и B127: «Сохраняй, выполняй B125, потом B127». P130 по B125 влита слиянием
`fb5090e`. Активна P131 по B127: структура Project по эталону v36 из P128, карта
`Ledger/Assets/P128/v36-plainnames.map`, целевое дерево `Ledger/Assets/P128/v36-plainnames-tree.txt`.

## Workspace

Branch: `feature/structure-v36`, от master `fb5090e`.
Revision: `21243fd67535e19475c9b1a8f7b50949cc5402d7`

Установленный рантайм `d8e186b`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- VAC 1 P131 `502d2f6` записал снимки «до» в `Ledger/Assets/P131/` и сделал P131 активной.
- VAC 2 P131 `76774c1` перенёс и поделил части `Project/shared/` по карте; `total` упал с 24.0 до 21.7.
- VAC 3 P131 `312f44e` перенёс корень `structural_lint` в `targeting`, `rule_set`, `rule_identities` и `reporting`;
  два отклонения записаны в Delivery P131: `rule_identities` вне `rule_set` из-за цикла и бюджеты `dependents` +1 у
  пяти каталогов. Бюджеты пользователь ещё не видел, в отчёте их нужно вынести на решение.
- VAC 4 P131 `d7b21fd` перенёс `structural_lint/architecture`: `contract_check`, `extraction/references`,
  переименования rust-резолвера и деление python-резолвера.
- VAC 5 P131 `21243fd` перенёс `languages/rust/context` в `exemptions` и `languages/measure` в `neutral_metrics`.
- VAC 6 P131 переименовывает файлы `agent_hooks`, `change_delivery/gate`, `initialization` и
  `project_config/settings` по карте.
- Порядок дальше: `command_runner`, `project_memory/evidence`, `command_line` → `cli_dispatch`. Ссылки:
  `.agentrig/checks.yaml`, `.agentrig/lint.yaml`, `Project/Cargo.toml`, `tooling/pyproject.toml`, ссылки применения
  в `Ledger/Decisions.md` и `Ledger/Invariants.md`, README. Детали решений неизменны (D020).

## Verification

- Снимки сняты установленным рантаймом `d8e186b` на `fb5090e`: `total 24.0 units 192`.
- Гейт коммита VAC 5 прошёл: lint, rustfmt, clippy, 429 тестов и проверки Python.
- На изменениях VAC 6 прошли сборка всех целей, `just lint`, `memory-check`, `config-check` и 98 тестов
  `agent_hooks::`, `change_delivery::`, `project_setup::` и `project_config::`.

## Blockers

Нет.

## Next action

VAC 7: по карте `command_runner/registry` → `tool_registry` с делением `check.rs` на `placement_check.rs` и
`removal_records.rs`, `jobs/identity` → `jobs/pid_reuse_guard`, `project_memory/evidence` → `check_evidence` с
`content.rs` → `input_fingerprint.rs`; затем `command_line` → `cli_dispatch`. Свободные ID: D173+, I022+, P132+,
B128+ (B066 и D111 зарезервированы).
