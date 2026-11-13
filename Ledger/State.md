# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P133 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

Активна P134 по B130: ветки уходят в `remote` из `git.yaml` после коммита в feature-ветке и после feature-merge.
Пользователь поручил отправлять master и feature-ветки после каждого слияния.

## Workspace

Branch: `feature/branch-push`, от master `0dd124a`, где влита P133; `origin/master` на том же коммите.
Revision: `028f1136aaaaffa7664442039edb8c07bb7f0086`

Установленный рантайм `028f113`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты, `feature-merge` и
`git push` через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок
сохранены и отправлены в `origin`, `feature/init-probe` не трогать.

## Progress

- VAC 1 `ced7abc` сохранил B130 и выбрал P134.
- VAC 2 `028f113` по D179 в рантайме: необязательный ключ `remote` в `git.yaml`, отправка базы и ветки после
  feature-merge, команда `feature-push` и хук `post-commit` от init, четыре теста в
  `change_delivery/git/tests/push.rs`.
- Мид-бранч пин D051 на `028f113` и bootstrap сделаны.
- VAC 3 включает отправку в риге: `remote: origin` в `.agentrig/git.yaml` и `.agentrig/hooks/post-commit`.

## Verification

- Гейт VAC 2 `028f113` прошёл с 496 тестами.
- После bootstrap `028f113` установленный рантайм принял конфигурацию с `remote: origin`.

## Blockers

Нет.

## Next action

Проверить, что коммит VAC 3 отправил ветку в `origin`, затем закрыть P134: приёмка, архив, пин, feature-merge.
Свободные ID: D180+, I022+, P135+, B131+ (B066 и D111 зарезервированы).
