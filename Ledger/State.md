# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P133 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

P133 по B129 научила правило `superseded-form` брать `Vec` из возвращаемого типа и выполнена. Новая работа не
выбрана, Plan пуст.

## Workspace

Branch: `feature/superseded-return`, от master `560c308`, где влита P132.
Revision: `582fd9586fc7d2308036afb73ef00f9104b4ac70`

Установленный рантайм `582fd95`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- P133 выполнена в VAC 1–2 на ветке `feature/superseded-return`, итоги и приёмка в
  [Archive/Plan/133.md](Archive/Plan/133.md). Решение D178.
- Закрывающий VAC ставит пин D051 на `582fd95` и переносит P133 и B129 в Archive.

## Verification

- VAC 2 `582fd95` прошёл гейт с 440 тестами.
- После bootstrap `582fd95` lint рига чист, `lint-rules` называет оба новых случая.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/superseded-return` с сохранением ветки, затем отчёт пользователю.
Свободные ID: D179+, I022+, P134+, B130+ (B066 и D111 зарезервированы).
