# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P133 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

Активна P134 по B130: ветки уходят в `remote` из `git.yaml` после коммита в feature-ветке и после feature-merge.
Пользователь поручил отправлять master и feature-ветки после каждого слияния.

## Workspace

Branch: `feature/branch-push`, от master `0dd124a`, где влита P133; `origin/master` на том же коммите.
Revision: `0dd124aa1b57af5c34943a7e0dc33d83d0a11624`

Установленный рантайм `582fd95`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты, `feature-merge` и
`git push` через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок
сохранены и отправлены в `origin`, `feature/init-probe` не трогать.

## Progress

- VAC 1 сохраняет B130 и выбирает P134.
- Дальше рантайм: ключ `remote`, отправка в feature-merge, команда для хука post-commit и его генерация в init,
  тесты с временным голым репозиторием, решение D179. Потом мид-бранч пин D051, затем в риге `remote: origin` и
  `.agentrig/hooks/post-commit`.

## Verification

- На master `0dd124a` гейт P133 прошёл с 440 тестами, lint рига чист.

## Blockers

Нет.

## Next action

Реализовать в рантайме ключ `remote` и отправку при feature-merge и после коммита, с тестами AC2–AC4.
Свободные ID: D179+, I022+, P135+, B131+ (B066 и D111 зарезервированы).
