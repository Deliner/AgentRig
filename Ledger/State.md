# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P126 выполнены, влиты в master и
перенесены в Archive. B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует
дополнительного согласования, пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине.

2026-11-12 пользователь выбрал P126 → P127 и поручил выполнять их по порядку без новых вопросов. P126 о пяти гонках
«проверить, потом действовать» влита. P127 активна: варианты правил IV-20, IV-22 и IV-30 по B122.

## Workspace

Branch: `feature/rule-variants`, от master `e006e9b`.
Revision: `28f494b9e65bfb7f29187ff3d70b755db43fb74e`

Установленный рантайм `28f494b`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- P126 влита слиянием `e006e9b`: `fs::present` и пять мест B121, перенос в Archive `1254ce0`, пин `8861397`.
- VAC 1 `431b9f7` сделал P127 активной. VAC 2 `4179ce3` записал в Delivery P127 протокол новой выборки и предикаты
  трёх вариантов до заморозки.
- VAC 3 `9e9af8b` заморозил предикаты инертными видами: `hidden-exit` и `stale-check` в
  `languages/rust/control_flow`, `untyped-conversion` в `languages/rust/signature`, без записи в реестр.
- VAC 4 `28f494b` записал в Delivery P127 замер: все три варианта прошли порог AC3. Новая выборка корпуса: IV-20
  верна на 23 из 23, IV-22 на 47 из 48, IV-30 на 15 из 16.
- VAC 5 закрепляет рантайм на `28f494b` и включает три вида уровнем error: реестры, `.agentrig/lint.yaml`, навыки
  `expose-the-exit`, `act-and-match-the-error` и `name-the-target`, исправления 10 находок `untyped-conversion` в
  Project аннотацией типа.

## Verification

- `feature-merge` P126 прошёл с кодом 0: 464 теста, все проверки PASS, очистка после слияния без ошибок.
- VAC 1 `431b9f7` и VAC 2 `4179ce3`: хук коммита прошёл.
- VAC 3 `9e9af8b`: хук коммита прошёл, 422 теста. VAC 4 `28f494b`: хук прошёл.
- Пин `28f494b`: bootstrap завершился с кодом 0.
- Рабочее дерево VAC 5: `just lint` кандидата без находок, `lint-config-check` валиден, 56 тестов discovery,
  project_setup и трёх видов прошли. `init --language rust` кандидата во временный корень записал три правила уровнем
  error и три навыка.

## Blockers

Нет.

## Next action

Закоммитить VAC 5, затем D171 о поставке трёх видов с метками `DECISION: D171`, приёмка AC1–AC6 в Delivery P127,
перенос P127 и B122 в Archive, финальный пин и `just feature-merge`. Пределы у краёв: в `languages/rust` и
`languages/tests` по 15 записей из 15, в `references` 15 из 15, зависимых у `measure` 7 из 7 и у `rules` 13 из 13.
Свободные ID: D171+, I022+, P128+, B123+ (B066 и D111 зарезервированы).
