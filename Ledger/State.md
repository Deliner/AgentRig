# State

## Focus

Ветка `feature/backlog-research-cards` сохраняет в Backlog карточки исследовательского реестра
[B068](Backlog/068.md)–[B074](Backlog/074.md); Plan пуст. 2026-11-07 пользователь поручил автономно: слить эту ветку,
взять открытые карточки Backlog в Plan и выполнить их, затем искать новые правила в разборе clippy и в реальных
ошибках агентов в AgentRig и voxel-rust; поиск правил по спискам остановлен. Последняя поставка продукта — P070
([B067](Archive/Backlog/067.md)), пин D051 `a1743fc`.

## Workspace

Branch: `feature/backlog-research-cards`
Revision: `465b231b248dc20495fb0cb5a15b5c970c12c71e`

База master `8b9ffdd`. Ветка меняет только Ledger: Backlog.md и карточки B068–B074, плюс этот снимок. Project не
меняется, пин D051 не нужен. Ветки `feature/delivery-repairs`, `feature/state-refresh` и `feature/init-probe`
сохранены. macOS: в PATH `just write`/`just run`/`just check`/`just feature-merge` нет cargo — вызовы с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; поставка — `just candidate <команда> --root
<каталог>`.

## Progress

- Карточки B068 `d717041`, B069 `d050e8d`, B070 `2bdb465`, B071–B073 `759a2eb`, B074 `465b231` сохранены по
  согласию пользователя.
- Исследование 20 кандидатов в правила вне Ledger: ни один не прошёл; отчёт
  `/Users/admin/projects/agentrig-research/rules-20-report.md`.
- Открытые карточки Backlog: B047, B048, B050, B051, B064, B068–B074.

## Verification

- 2026-11-07: коммиты карточек прошли pre-commit гейт; `just feature-merge` этой ветки ещё не выполнен.

## Blockers

Нет.

## Next action

Закоммитить этот снимок, выполнить `just feature-merge`, затем на новой ветке выбрать открытые карточки в Plan по
edit-plan и начать первую. Свободные ID: D113+, I022+, P071+, B075+ (B066 и D111 зарезервированы, D112 обещан
B069).
