# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P138 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил показанную карточку B135 о флаге `--force-warn`: «Сохрани и сразу сделай». Выбрана
P139, решение D183.

## Workspace

Branch: `feature/force-warn`, от master `f38284e`, где влиты P137 и P138.
Revision: `f38284e670e33a9bd8b55c080cc37672916f8036`

Установленный рантайм `54999fc`, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены и отправлены в
`origin`, `feature/init-probe` не трогать.

## Progress

- VAC 1: карточка B135 и P139; `allow-justification` сообщает `--force-warn` в списке `rustflags` без ссылки; текст
  ошибки и навыки `justify-allow` и `move-rustflags` называют четыре флага; решение D183. Вид правила и правила рига
  не меняются, мид-бранч пин не нужен.

## Verification

До коммита VAC 1: тесты `flag_list` и `shipped_rules_keep_compiler_flags_in_the_rig_list` прошли, 6 из 6. Проба
кандидата на временном проекте: `init` перенёс `--force-warn` из `build.rustflags`, lint дал ошибку
`allow-justification` с навыком `justify-allow`; в правленом списке флаг без ссылки дал находку, со ссылкой нет.

## Blockers

Нет.

## Next action

Закоммитить VAC 1, затем перенести P139 и B135 в архив с таблицей AC, пин головы по D051 и `just feature-merge`.
Свободные ID: D184+, I022+, P140+, B136+.
