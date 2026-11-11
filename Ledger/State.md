# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P116 по карточкам B102–B106, B108 и
B110 слиты в master и перенесены в Archive. B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята:
она требует дополнительного согласования, пользователь вернётся к ней сам.

2026-11-11 пользователь выбрал B110 и B111 сразу после P115. [P117](Archive/Plan/117.md) по
[B111](Archive/Backlog/111.md) выполнена и перенесена в Archive на своей ветке; осталось закрепить пин и слить ветку.
Других выбранных карточек нет. B112 ждёт доступа к Linux-машине.

## Workspace

Branch: `feature/option-option-owner`, от master `44c9004`.
Revision: `ebbabe2eb5aede1eb32fbc7990d62bd3c7276498`
Завершение P117; этот снимок входит в коммит пина поверх неё.

Установленный рантайм `ebbabe2`, его пин — коммит этого снимка. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`. Ветки прежних поставок сохранены,
`feature/init-probe` не трогать.

## Progress

- P116 слита в master `44c9004`, ветка `feature/gate-rebuild` сохранена.
- P117 выполнена: VAC 1 `8cd02d3` по D162 оставляет `Option<Option<T>>` линту Clippy `option_option`, правило
  `ambiguous-option` ловит только `Option<bool>`. Delivery с AC1, AC2 и пробой Clippy — в архивной карточке P117.
- Найдено: `type`-псевдоним и аннотацию `let` с вложенным `Option` теперь не ловит ни одна проверка рига; записано в
  D162.
- Главная цена процессных тестов на macOS: первый запуск каждого нового исполняемого файла, около 0,37 с. Пользователь
  включит Developer Tools для терминала и перезапустит приложение после отчёта; затем замерить скорость заново.

## Verification

- AC1: коммит фрагмента с `Option<Option<u32>>` в копии рига на `8cd02d3` прошёл `lint` и остановлен одной ошибкой
  `clippy`; на master `44c9004` тот же коммит останавливал `lint` с `ambiguous-option`.
- AC2: тест `rust_flags_an_option_over_a_bool_wherever_it_stands_and_leaves_a_nested_option_to_clippy` прошёл.
- VAC 1 прошёл гейт коммита: все проверки PASS, 397 тестов. Завершение `ebbabe2` прошло гейт коммита: все проверки
  PASS. `just bootstrap` установил `ebbabe2`.

## Blockers

Нет.

## Next action

`just feature-merge` ветки `feature/option-option-owner` с сохранением ветки. Затем отчёт пользователю. Свободные ID: D163+, I022+, P118+,
B113+ (B066 и D111 зарезервированы).
