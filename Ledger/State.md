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
Revision: `ae95b20bf9b2889a431752daa9ca294ec90a3345`, перенос `test_capabilities`; этот снимок входит в VAC переноса
`test_configuration`.

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
- Перенесены, последний в `ae95b20`: `test_git`, `test_delivery`, `test_runtime`, `test_gate`, `test_revision`, все
  тесты архитектуры lint, `test_jobs`, `test_commands`, `test_registry`, `test_layout`, `test_hooks`,
  `test_lint_defaults`, `test_revision_history`, `test_wizard`, `test_evidence`, `test_lint`, `test_config_check`,
  `test_memory`, `test_lint_baseline`, `test_lint_explain`, `test_lint_portable`, `test_reminders`, `test_resume`,
  `test_doctor`, `test_capabilities`. На оракулах cargo: I001–I008, I010, I012, I014, I016–I021.
- Заготовки: в `shared::testing` проект, память, Git, проверяемый репозиторий, снимок файлов, запуск рантайма,
  RERUN и `replay`; `Consumer` у входа `structural_lint` линтует потребителя в процессе через закрытый `analyze`;
  `structural_lint/tests/consumer.rs` готовит потребителя с правилом размера; `project_memory/tests/consumer.rs`
  готовит память с решением D001 и коммитом. Из других фич lint вызывают только процессом.
- Этот VAC переносит `test_configuration.py` в `project_config/tests/settings.rs`: 2 теста через процесс
  `config-check`, ссылка D038 указывает на `settings.rs`. В `Project/project_config` Python не осталось: его цели
  pytest и две группы карты pytest убраны, каталог убран из аргументов mypy и vulture.
- Главная цена процессных тестов на macOS: первый запуск каждого нового исполняемого файла, около 0,37 с, по одному
  на всю систему. Её снимает только настройка машины, Developer Tools для терминала; это решение пользователя.

## Verification

- Полный Rust-прогон длится около 30 с: 12 тестов `change_delivery::git::tests` подряд идут 42,7 с против 61,5 с у
  Python, а параллельно самый долгий растягивается с 10 до 25 с. Реальное время там в 5–10 раз больше процессорного:
  macOS проверяет первый запуск каждого нового хука и лаунчера, которые создаёт `init`, и параллельные тесты ждут одну
  системную службу. Это та же причина, что снимается только настройкой Developer Tools.
- Гейт коммита `ae95b20` прошёл: 408 Rust-тестов за 35,01 с, pytest 102 за 7,05 с.
- В рабочем дереве этого VAC: 5 тестов `project_config::tests::settings` прошли за 0,07 с; удалённый файл на снимке:
  13 случаев за 0,10 с.

## Blockers

Нет.

## Next action

Перенести остальные Python-тесты по выигранным секундам: `test_language`, `test_plan`, `test_backlog` и прочие; I009,
I011 и I015 пока на pytest. Потом замеры AC5 полного прогона и гейта против master `41c227f`. Каждый VAC
дописывает строки в таблицы Delivery P115. Свободные ID: D158+, I022+, P116+, B109+ (B066 и D111
зарезервированы).
