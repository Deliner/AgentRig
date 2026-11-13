# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P132 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил карточку B129 о том, чтобы правило `superseded-form` видело `Vec` в возвращаемом
типе: «Сохрани и сразу сделай». Выбрана P133, она активна.

## Workspace

Branch: `feature/superseded-return`, от master `560c308`, где влита P132.
Revision: `560c3089b223f3037e2f2263fc285e2dbdc025a4`

Установленный рантайм `f57a156`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- VAC 1 P133 сохранил B129 и выбрал P133.

## Verification

- master `560c308`: `feature-merge` P132 прошёл со всеми 490 тестами.

## Blockers

Нет.

## Next action

VAC 2 P133: в детекторе `languages/rust/superseded_form.rs` тип `Vec` из возвращаемого типа функции или замыкания
для последнего выражения тела и для `return`, тесты, описание правила и навык `use-the-current-form`, решение D178
преемником предиката D177, место `structural_lint/architecture/tests.rs:504` на `Vec::from_fn`. Затем пин D051 и
`just feature-merge`.
Свободные ID: D178+, I022+, P134+, B130+ (B066 и D111 зарезервированы).
