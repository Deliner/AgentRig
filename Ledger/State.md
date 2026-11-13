# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P139 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил показанную карточку B136 о флагах в команде clippy: «Сохрани и сразу сделай». Выбрана
и выполняется P140, решение D184.

## Workspace

Branch: `feature/command-flags`, от master `df9e60a`, где влита P139.
Revision: `df9e60a427c3cbb55ca2b861e23d867534275ccb`

Установленный рантайм `37e93cf` в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены и отправлены в
`origin`, `feature/init-probe` не трогать.

## Progress

- VAC 1, этот коммит: карточка B136 и P140; `allow-justification` сообщает элемент после `--` в `argv` команды, чей
  первый элемент `cargo` и где до `--` стоит `clippy` или `rustc`, если он начинается с `-A`, `--allow`, `--cap-lints`
  или `--force-warn` и выше нет ссылки; шаблон `init` добавляет `.agentrig/commands.yaml` в include правила; навык
  `justify-allow` и текст ошибки описывают этот случай; решение D184.

## Verification

До коммита прошли 9 тестов по фильтрам `flag_list`, `shipped_rules_check_lowering_flags_of_the_clippy_command` и
`shipped_rules_keep_compiler_flags_in_the_rig_list`. Lint кандидата на самом репозитории чист. Проба кандидата на
временном проекте после `init`: lint без находок в `commands.yaml`; `--cap-lints allow` в конце команды clippy дал
ошибку `allow-justification` с навыком `justify-allow`, со ссылкой выше lint прошёл.

## Blockers

Нет.

## Next action

VAC 2: перенести P140 и B136 в архив с таблицей AC, затем пин D051 и `just feature-merge`. Свободные ID: D185+, I022+,
P141+, B137+.
