# State

## Focus

Пользователь 2026-11-10 задал стандарт рига, записан в AGENTS.md и D151. P110–P114 по карточкам B102–B106 слиты в
master и перенесены в Archive. B107 о сверке имён узлов с грамматикой сохранена, но в работу не взята: она требует
дополнительного согласования, пользователь вернётся к ней сам.

2026-11-11 пользователь поручил после P114 автономно перенести тесты рига с Python на Rust: быстрее, только через
публичные интерфейсы, тесты внутри проверяемых фич; где хватает Rust-теста, Python-тест не нужен. Ок дан заранее на
всё, что для этого нужно. По этому согласию сохранена карточка [B108](Backlog/108.md) и выбрана
[P115](Plan/115.md), она active.

## Workspace

Branch: `feature/rust-tests`, от master `41c227f`.
Revision: `48c592ea8a0e3b4af5d52fca5a30011ec540a467`, перенос `test_git.py`; этот снимок входит в VAC переноса
`test_delivery.py`.

Установленный рантайм `cd46b81`. macOS: cargo, кандидат, коммиты и `feature-merge` через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; Python-тесты кандидата с абсолютным
`WORKER_BINARY=/Users/admin/projects/AgentRig/.cache/worker/release/agentrig`: с относительным путём тесты, которые
меняют каталог, падают. Ветки прежних поставок сохранены, `feature/init-probe` не трогать.

## Progress

- P114 слита в master `41c227f`. Открытые находки P114: `Option<Option<T>>` ловят и `ambiguous-option`, и Clippy
  `option_option`, вопреки границе B106 «у каждой формы один владелец»; ветку песочницы Linux в `145e1e0` не
  компилировали.
- Списки запретов в `~/projects/junk/rust-ast-rules` переразмечены по итогам P114: в списке 2 применено 18 из 30, в
  списке 3 — 17 из 30.
- P115: исходный замер — 47 файлов Python-тестов, 233 функции, 527 случаев; полный прогон pytest 2026-11-11: 527
  прошли за 178 с, по часам 187 с. Больше всего времени у `test_git.py` 62 с и `test_delivery.py` 19 с. 202 Rust-теста
  за 2 с; они уже вызывают публичные функции фич в процессе, например `evaluate`, `dispatch`, `init`.
- D157 в `cd46b81`, пин `05c2e39`, I021 на Rust в `23e4149`, весь `test_git.py` в `48c592e`: 12 Rust-тестов в
  `Project/change_delivery/git/tests/`, I006 и I010 на оракулах cargo. Ход переноса, соответствие тестов, проверки
  разрыва и замеры ведутся в Delivery карточки [P115](Plan/115.md).
- Этот VAC переносит `shared/testing/test_delivery.py`: сценарий доставки в `project_setup/tests/delivery.rs`,
  коллизия `init` и I020 в `project_setup/initialization/tests/`, проверки оракулов и I002 в
  `project_memory/tests/oracles.rs`. Поддержка `shared/testing/tests` даёт тестам Python инструментов через uv.
- Главная цена процессных тестов на macOS: первый запуск каждого нового исполняемого файла, около 0,37 с, по одному
  на всю систему. Её снимает только настройка машины, Developer Tools для терминала; это решение пользователя.

## Verification

- В рабочем дереве этого VAC: весь `--lib` 221 прошли за 23,2 с с готовым кандидатом и за 32,5 с со сборкой;
  `project_setup` 4 прошли за 9,2 с, оракулы 3 за 2,9 с. Кандидат: `memory-check` проходит с прежним предупреждением
  о `grammar.js`, D112; `config-check`, `lint-config-check`, lint, rustfmt и Clippy чистые. Проверка разрыва I002 и
  I020 записана в P115; после отката поломок тесты проходят.

## Blockers

Нет.

## Next action

Перенести `project_setup/tests/test_runtime.py`, 9 с; дальше `change_delivery/gate/tests/test_gate.py` 7 с и
остальные по выигранным секундам. Каждый VAC дописывает Delivery в P115. Свободные ID: D158+, I022+, P116+, B109+ (B066 и D111
зарезервированы).
