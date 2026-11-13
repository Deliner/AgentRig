# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P133 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

Активна P134 по B130: ветки уходят в `remote` из `git.yaml` после коммита в feature-ветке и после feature-merge.
Пользователь поручил отправлять master и feature-ветки после каждого слияния.

## Workspace

Branch: `feature/branch-push`, от master `0dd124a`, где влита P133; `origin/master` на том же коммите.
Revision: `ced7abcd850af32782bf95f8f6ba608a884c02b3`

Установленный рантайм `582fd95`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты, `feature-merge` и
`git push` через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок
сохранены и отправлены в `origin`, `feature/init-probe` не трогать.

## Progress

- VAC 1 `ced7abc` сохранил B130 и выбрал P134.
- VAC 2 по D179 в рантайме: необязательный ключ `remote` в `git.yaml`, отправка базы и ветки после feature-merge,
  команда `feature-push` и хук `post-commit` от init, четыре теста в `change_delivery/git/tests/push.rs`.
- Дальше мид-бранч пин D051 на VAC 2, затем в риге `remote: origin` и `.agentrig/hooks/post-commit`.

## Verification

- До коммита VAC 2: 4 теста `push` и 93 теста групп `project_setup`, `cli_dispatch`, `shared::vcs`,
  `change_delivery`, `project_config` прошли на кандидате; lint кандидата чист.

## Blockers

Нет.

## Next action

Пин D051 на VAC 2 и bootstrap, затем включить отправку в риге.
Свободные ID: D180+, I022+, P135+, B131+ (B066 и D111 зарезервированы).
