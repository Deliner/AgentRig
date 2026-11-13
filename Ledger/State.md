# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P129 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B126 о блокирующих правилах на числах
`lint-structure` сохранена, в работу не выбрана.

2026-11-13 пользователь одобрил B125, B126 и B127: «Сохраняй, выполняй B125, потом B127». Активна P130 по B125:
`just lint-structure` и навык `clarify-structure` из навыка v5 P129 ставятся в Rust-проекты через `init` как
подсказка, без блокировки. Затем P131 по B127: структура Project по эталону v36 из P128.

## Workspace

Branch: `feature/structure-skill-delivery`, от master `3dddae9`.
Revision: `3dddae97a4162343e66de97b7b978c9a8921f675`

Установленный рантайм `c17b785`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- P129 влита слиянием `1d69014`, поправка к её итогу слиянием `3dddae9`. Выбраны `lint-structure` `b89fb25` и навык
  `Ledger/Assets/P129/skill-v5.md`: Sonnet 66,67 % против 51,6 % у исходного дерева.
- VAC 1 P130 сохраняет B125, B126 и B127 и выбирает P130 и P131.

## Verification

- Шлюзы коммитов `c17b785` и `43654d8` прошли; bootstrap установил `c17b785`, `agentrig 0.3.0`.
- `feature-merge` влил `feature/structure-signal` слиянием `1d69014` и `feature/structure-signal-note` слиянием
  `3dddae9`.

## Blockers

Нет.

## Next action

После VAC 1 найти, как `init` пишет рецепты `just` и навыки, и добавить рецепт `lint-structure`, пропуск в защите
оболочки и канонический навык `clarify-structure`. Свободные ID: D173+, I022+, P132+, B128+ (B066 и D111
зарезервированы).
