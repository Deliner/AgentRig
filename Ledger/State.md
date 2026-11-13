# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P140 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил показанную карточку B136 о флагах в команде clippy: «Сохрани и сразу сделай». P140
выполнена, решение D184; ветке остаются пин D051 и слияние. Plan пуст.

## Workspace

Branch: `feature/command-flags`, от master `df9e60a`, где влита P139.
Revision: `71e8cd6c7bb95f0210f7a077428c36710856389c`

Установленный рантайм `37e93cf` в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены и отправлены в
`origin`, `feature/init-probe` не трогать.

## Progress

- VAC 1 `71e8cd6`: карточка B136 и P140; `allow-justification` сообщает элемент после `--` в `argv` команды, чей
  первый элемент `cargo` и где до `--` стоит `clippy` или `rustc`, если он начинается с `-A`, `--allow`, `--cap-lints`
  или `--force-warn` и выше нет ссылки; шаблон `init` добавляет `.agentrig/commands.yaml` в include правила; навык
  `justify-allow` и текст ошибки описывают этот случай; решение D184.
- VAC 2, этот коммит: перенос P140 и B136 в архив с таблицей AC; итог в Archive/Plan/140.md.

## Verification

На `71e8cd6` гейт прошёл 464 затронутых теста. Lint кандидата `71e8cd6` на самом репозитории чист. Проба кандидата на
временном проекте после `init`: lint без находок в `commands.yaml`; `--cap-lints allow` в конце команды clippy дал
ошибку `allow-justification` с навыком `justify-allow`, со ссылкой выше lint прошёл.

## Blockers

Нет.

## Next action

Записать голову VAC 2 в `stable.txt` и выполнить `just bootstrap`, закоммитить пин со State, затем `just
feature-merge`; слияние отправит master и ветку в `origin`. Свободные ID: D185+, I022+, P141+, B137+.
