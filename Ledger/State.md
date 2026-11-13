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
Revision: `76774c1efe40c86ef52a43ae5208ed04deed4e83`

Установленный рантайм `d8e186b`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- VAC 1 P131 `502d2f6` записал снимки «до» в `Ledger/Assets/P131/` и сделал P131 активной.
- VAC 2 P131 `76774c1` перенёс и поделил части `Project/shared/` по карте; `total` упал с 24.0 до 21.7.
- VAC 3 P131 переносит корень `structural_lint` в `targeting`, `rule_set`, `rule_identities` и `reporting`; два
  отклонения записаны в Delivery P131: `rule_identities` вне `rule_set` из-за цикла и бюджеты `dependents` +1 у пяти
  каталогов. Бюджеты пользователь ещё не видел, в отчёте их нужно вынести на решение.
- Порядок дальше: `structural_lint/architecture` и `languages`, затем остальные верхние части и `command_line` →
  `cli_dispatch`. Ссылки: `.agentrig/checks.yaml`, `.agentrig/lint.yaml`, `Project/Cargo.toml`,
  `tooling/pyproject.toml`, ссылки применения в `Ledger/Decisions.md` и `Ledger/Invariants.md`, README. Детали
  решений неизменны (D020).

## Verification

- Снимки сняты установленным рантаймом `d8e186b` на `fb5090e`: `total 24.0 units 192`.
- На изменениях VAC 3 прошли сборка всех целей, `just lint`, `memory-check`, `config-check`, `lint-config-check` и
  333 теста `structural_lint::` и `project_setup::`.

## Blockers

Нет.

## Next action

VAC 4: по карте `runner` → `contract_check`, `source` → `extraction`, деление python `resolution`, переименования
rust-резолвера, `languages/rust/context` → `exemptions`, `languages/measure` → `neutral_metrics`. Свободные ID: D173+,
I022+, P132+, B128+ (B066 и D111 зарезервированы).
