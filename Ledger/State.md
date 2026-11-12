# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P126 выполнены, влиты в master и
перенесены в Archive. B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует
дополнительного согласования, пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине.

2026-11-12 пользователь выбрал P126 → P127 и поручил выполнять их по порядку без новых вопросов. P126 о пяти гонках
«проверить, потом действовать» влита. P127 активна: варианты правил IV-20, IV-22 и IV-30 по B122.

## Workspace

Branch: `feature/rule-variants`, от master `e006e9b`.
Revision: `e006e9b052c22d1a4c087e54bd95d3bf93136aed`

Установленный рантайм `1254ce0`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- P126 влита слиянием `e006e9b`: `fs::present` и пять мест B121, перенос в Archive `1254ce0`, пин `8861397`.
- VAC 1 `431b9f7` сделал P127 активной. VAC 2 записывает в Delivery P127 протокол новой выборки и предикаты трёх
  вариантов до заморозки.

## Verification

- `feature-merge` P126 прошёл с кодом 0: 464 теста, все проверки PASS, очистка после слияния без ошибок.
- VAC 1 `431b9f7`: хук коммита прошёл.

## Blockers

Нет.

## Next action

Заморозить по записанным предикатам инертные виды: `hidden-exit` и `stale-check` в
`languages/rust/control_flow`, `untyped-conversion` в `languages/rust/signature`; тесты там же в `languages/tests`.
Пределы у краёв: в `languages/rust` и `languages/tests` по 15 записей из 15, в `references` 15 из 15, зависимых у
`measure` 7 из 7 и у `rules` 13 из 13. Свободные ID: D171+, I022+, P128+, B123+ (B066 и D111 зарезервированы).
