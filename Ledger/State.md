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
Revision: `2666c1c0be883d881c6a71e4256ba915dc6d2586`

Установленный рантайм `c17b785`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- VAC 1 P130 сохранил B125, B126 и B127 и выбрал P130 и P131.
- VAC 2 P130 добавляет рецепт `lint-structure` в шаблон `init` и в рецепты рига, пропуск в защите оболочки, навык
  `clarify-structure` из навыка v5 P129, строку README и проверки, переносит P130 и B125 в Archive.

## Verification

- На кандидате прошли `rust_project_is_delivered_in_an_independent_layout`,
  `python_project_is_delivered_in_an_independent_layout`, `shell_catalog_and_recipe_listing` и
  `shell_guard_rejects_bypass`.
- `just lint`, `just config-check`, `just lint-config-check` и `memory-check` прошли на изменениях VAC 2.

## Blockers

Нет.

## Next action

После шлюза VAC 2 записать голову ветки в `stable.txt`, выполнить `just bootstrap`, закоммитить пин и влить ветку
через `just feature-merge`. Затем ветка P131 от master. Свободные ID: D173+, I022+, P132+, B128+ (B066 и D111
зарезервированы).
