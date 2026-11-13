# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P130 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B126 о блокирующих правилах на числах
`lint-structure` сохранена, в работу не выбрана.

2026-11-13 пользователь одобрил B125, B126 и B127: «Сохраняй, выполняй B125, потом B127». P130 по B125 выполнена:
`just lint-structure` и навык `clarify-structure` ставятся через `init` как подсказка, без блокировки. Дальше P131 по
B127: структура Project по эталону v36 из P128.

## Workspace

Branch: `feature/structure-skill-delivery`, от master `3dddae9`.
Revision: `d8e186b23748e790954ac6f250b98e08ff5dd60d`

Установленный рантайм `d8e186b`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- VAC 1 P130 сохранил B125, B126 и B127 и выбрал P130 и P131.
- VAC 2 P130 `d8e186b` добавил рецепт `lint-structure` в шаблон `init` и в рецепты рига, пропуск в защите оболочки,
  навык `clarify-structure` из навыка v5 P129, строку README и проверки, перенёс P130 и B125 в Archive.
- Коммит пина D051 записывает `d8e186b` в `stable.txt`.

## Verification

- Шлюз коммита `d8e186b` прошёл: lint, memory, rustfmt, clippy, 165 выбранных тестов Rust, ruff, mypy, typos, vulture.
- `just bootstrap` установил `d8e186b`, `agentrig 0.3.0`. Установленный хук пропустил `just lint-structure
  Project/agent_hooks`, вывод начался с `total 10.1 units 7`.

## Blockers

Нет.

## Next action

Влить ветку через `just feature-merge`. Затем ветка P131 от master: сверить карту v36 с текущим деревом Project. Свободные ID: D173+, I022+, P132+, B128+ (B066 и D111
зарезервированы).
