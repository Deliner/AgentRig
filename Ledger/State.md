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
Revision: `a2742f1d921b10a9f20e86f695e47aa6d8cf52e3`, перенос `test_state`; этот снимок входит в VAC переноса
`test_plan`.

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
- D157 в `cd46b81` привязал оракулы cargo к тестам вложенных модулей, пин `05c2e39`. Delivery карточки [P115](Plan/115.md)
  ведётся по критериям: соответствие AC3 со временем AC5 по файлам, поломки AC4, карта AC6; ход VAC — в коммитах.
- Перенесены, последний в `a2742f1`: `test_git`, `test_delivery`, `test_runtime`, `test_gate`, `test_revision`, все
  тесты архитектуры lint, `test_jobs`, `test_commands`, `test_registry`, `test_layout`, `test_hooks`,
  `test_lint_defaults`, `test_revision_history`, `test_wizard`, `test_evidence`, `test_lint`, `test_config_check`,
  `test_memory`, `test_lint_baseline`, `test_lint_explain`, `test_lint_portable`, `test_reminders`, `test_resume`,
  `test_doctor`, `test_capabilities`, `test_configuration`, `test_state`. На оракулах cargo после этого VAC:
  I001–I012, I014, I016–I021.
- Заготовки: в `shared::testing` проект, память, Git, проверяемый репозиторий, снимок файлов, запуск рантайма,
  RERUN и `replay`; `Consumer` у входа `structural_lint` линтует потребителя в процессе через закрытый `analyze`;
  `structural_lint/tests/consumer.rs` готовит потребителя с правилом размера; `project_memory/tests/consumer.rs`
  готовит память с решением D001 и коммитом. Из других фич lint вызывают только процессом.
- Этот VAC переносит `test_plan.py` в `project_memory/tests/plan.rs`: 10 тестов через процесс `memory-check`, и
  подсказку по Archive в `agent_hooks/tests/memory.rs` через `dispatch` в процессе. I009 переходит на оракул cargo,
  ссылки D012 и D031 указывают на `plan.rs`. Повтор цикла Plan из `validation.rs` убран, общий `edit` и `rejects`
  перенесены в `consumer.rs`. Карта уже выбирает новые тесты на путях прежнего файла; pytest-цель каталога остаётся
  ради `test_backlog.py`.
- Главная цена процессных тестов на macOS: первый запуск каждого нового исполняемого файла, около 0,37 с, по одному
  на всю систему. Её снимает только настройка машины, Developer Tools для терминала; это решение пользователя.

## Verification

- Полный Rust-прогон длится около 30 с: 12 тестов `change_delivery::git::tests` подряд идут 42,7 с против 61,5 с у
  Python, а параллельно самый долгий растягивается с 10 до 25 с. Реальное время там в 5–10 раз больше процессорного:
  macOS проверяет первый запуск каждого нового хука и лаунчера, которые создаёт `init`, и параллельные тесты ждут одну
  системную службу. Это та же причина, что снимается только настройкой Developer Tools.
- Гейт коммита `a2742f1` прошёл: 411 Rust-тестов за 34,22 с, pytest 51 за 1,22 с.
- В рабочем дереве этого VAC: `plan::` 10 тестов за 0,16 с, новый тест хука за 0,25 с; удалённый файл на снимке: 40
  случаев за 0,79 с. Допуск двух active роняет оракул I009 на `consumer.rs:49`, после отката тесты проходят.

## Blockers

Нет.

## Next action

Перенести остальные Python-тесты: `test_backlog`, `test_language` с I015, `test_language_delivery`,
`test_inventory_vcs`, `test_coverage`, `test_clippy_profile`; `test_launcher` проверяет tooling и остаётся с причиной. Потом замеры AC5 полного прогона и гейта против master `41c227f`. Каждый VAC
дописывает строки в таблицы Delivery P115. Свободные ID: D158+, I022+, P116+, B109+ (B066 и D111
зарезервированы).
