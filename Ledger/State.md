# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P139 выполнены и перенесены в Archive.
B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует дополнительного согласования,
пользователь вернётся к ней сам. B112 ждёт доступа к Linux-машине. B109 и B126 сохранены, в работу не выбраны.

2026-11-13 пользователь одобрил показанную карточку B135 о флаге `--force-warn`: «Сохрани и сразу сделай». P139
выполнена, решение D183; ветка запинена и ждёт слияния. Plan пуст.

## Workspace

Branch: `feature/force-warn`, от master `f38284e`, где влиты P137 и P138.
Revision: `37e93cf24a1610d5e060341e01bea9fead2e6a02`

Установленный рантайм `37e93cf` — голова VAC 2 этой ветки, он же в `stable.txt`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены и отправлены в
`origin`, `feature/init-probe` не трогать.

## Progress

- VAC 1 `21ba37a`: карточка B135 и P139; `allow-justification` сообщает `--force-warn` в списке `rustflags` без
  ссылки; текст ошибки и навыки `justify-allow` и `move-rustflags` называют четыре флага; решение D183.
- VAC 2 `37e93cf`: перенос P139 и B135 в архив с таблицей AC; итог в Archive/Plan/139.md.
- Пин: `stable.txt` называет `37e93cf`, `just bootstrap` его установил.

## Verification

На `21ba37a` гейт прошёл 461 затронутый тест, на `37e93cf` — 12 затронутых тестов. Проба кандидата `21ba37a` на временном проекте: `init` перенёс
`--force-warn` из `build.rustflags`, lint дал ошибку `allow-justification` с навыком `justify-allow`; в правленом
списке флаг без ссылки дал находку, со ссылкой нет.

## Blockers

Нет.

## Next action

Влить ветку `just feature-merge`; слияние отправит master и ветку в `origin`. Свободные ID: D184+, I022+, P140+, B136+.
