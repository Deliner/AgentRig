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
Revision: `e5699fc8f0c45b578ffc020bbb5184e85ca502ac`, перенос тестов привязок, макросов и атрибутов архитектуры;
этот снимок входит в VAC переноса тестов крейтов, ресурсов и описей архитектуры.

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
- `test_delivery.py` перенесён в `395dd03`: сценарий доставки в `project_setup/tests/delivery.rs`, коллизия `init` и
  I020 в `project_setup/initialization/tests/`, проверки оракулов и I002 в `project_memory/tests/oracles.rs`.
  Поддержка `shared/testing/tests` даёт тестам Python инструментов через uv.
- `test_runtime.py` перенесён в `6015498`: 3 Rust-теста в `project_setup/tests/runtime.rs`, ссылка D150 на них.
- `test_gate.py` перенесён в `e2c8780`: 9 тестов в `change_delivery/gate/tests/`, 2 в `project_config/tests/`, 2 в
  `command_line/tests/`. Минимальный проект и запуск RERUN есть в `shared::testing`. `test_revision.py` перенесён в
  `b67d901`, память проекта тоже в `shared::testing`.
- `test_architecture_bindings.py`, `_macros.py`, `_attributes.py` и `_incomplete.py` перенесены в `e5699fc` в
  `structural_lint/tests/architecture/`: заготовка `Consumer` линтует проект-потребитель в процессе через `analyze`.
  Из `architecture/tests/` такой вызов давал цикл каталогов, поэтому тесты лежат у входа `structural_lint`.
- Этот VAC переносит туда же `_resources.py`, `_crates.py`, `_inventory.py` и `_vcs_directories.py` и удаляет
  `_behavior.py`. Случаи с репозиторием Git идут процессом через `shared::testing`.
- Остались Python-тесты архитектуры: `test_architecture.py` с общими заготовками и сверкой двух бинарников,
  `_coupling.py`, `_disallowed.py`, `_module_wiring.py`.
- Главная цена процессных тестов на macOS: первый запуск каждого нового исполняемого файла, около 0,37 с, по одному
  на всю систему. Её снимает только настройка машины, Developer Tools для терминала; это решение пользователя.

## Verification

- Гейт коммита `e5699fc`: 204 Rust-теста, pytest 203, все проверки прошли.
- В рабочем дереве этого VAC: 11 новых тестов прошли за 0,43 с, весь `structural_lint::` 190 за 0,7 с; lint кандидата
  чистый. Четыре удалённых файла с тестами на снимке HEAD: 31 случай за 6,6 с.

## Blockers

Нет.

## Next action

Перенести остальные тесты архитектуры на заготовку `Consumer`, затем `test_architecture.py` с одним процессным тестом
сверки `agentrig` и `agentrig-lint`; дальше остальные Python-тесты по выигранным секундам. Каждый VAC дописывает
Delivery в P115. Свободные ID: D158+, I022+, P116+, B109+ (B066 и D111 зарезервированы).
