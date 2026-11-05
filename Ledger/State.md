# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P065 слиты (последний merge `b224444`).
**P066** (B062, одноразовые ремонты собственного кода: AC2 fail-closed + `&Settings`, AC3 enum'ы,
AC4 именованные записи, AC5 `Ordering`, AC6 `--help`): VAC-A (AC2) `384936a`; VAC-B (AC6) `4d2e48a`;
VAC-C (AC3a `Level`) `a983c85`; VAC-D (AC3b `JobState`) `8059224`; VAC-E (AC3c `Command`)
`029eed3`; VAC-F (AC3d `Catalog`) `96a4a57`; VAC-G (AC4 записи) завершается этим коммитом; далее
VAC-H: AC5 (D108 + маркеры) + ротация P066/B062, пин D051, `just feature-merge`; затем P067 (B061),
P068 (B063).

## Workspace

Ветка `feature/own-code-repairs` (база master `b224444`), наблюдаемый HEAD `96a4a57` (VAC-F);
stable.txt → `42f028a`. macOS: коммиты/cargo — через `just scratch` с восстановленным PATH
(`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; профиль clippy гейта —
`tooling/worker/run run clippy`; догфуд — `tooling/worker/run run candidate -- lint`; pytest одного
файла — `tooling/worker/run run test -- <файл>`.

## Progress

- Разбор карточки: fail-open AC2 — `agent_hooks/guard.rs:95` (`enters_check`:
  `shell_words::split(word).unwrap_or_default()` — нечитаемое слово тихо считалось «не вход в
  check/commit», и маршрут VAC-коммита (D047) обходился; воспроизводимый обход — ANSI-C-кавычки
  `just scratch -- "git commit -q -m $'it\'s'"`: shell_words даёт «missing closing quote», а `sh -c`
  выполняет commit). `guard_reference_with` уже пробрасывает ошибки (AC1 в P056) — менялась только
  сигнатура.
- VAC-A (`384936a`): `enters_check` → `Result<bool>`; ошибка разбора слова — отказ хука с причиной
  (слово и причина) через общий путь отказов `shell_event` (`agent_hooks/mod.rs`);
  `guard_reference_with(root, phase, &Settings)` как у `guard_commit`; вызов в `command_line/mod.rs`;
  негативный тест `test_shell_guard_refuses_unreadable_check_entry` в `agent_hooks/tests/test_hooks.py`;
  строка P066 в Plan.md — active. Побочный эффект: слово с непарной кавычкой в обычном аргументе
  (например, `just read -- grep "don't"`) теперь отклоняется с указанием слова — охрана не отличает
  shell-текст от аргумента; писать `don.t`.
- VAC-B (`4d2e48a`): `command_line/main.rs` — ошибка неизвестной команды стала одной строкой
  `unknown command "<имя>"; run agentrig --help for the command table` (таблица команд живёт в
  `print_help`); usage-строка с частичным перечнем удалена.
- VAC-C (`a983c85`, AC3a): `enum Level { Warning, Error }` переехал к владельцу `Diagnostic` в
  `shared/diagnostics/lint.rs` (serde lowercase, `name()`, `Copy`, `Eq`); `structural_lint::config::Level`
  — реэкспорт (конфиг правил читает тот же тип); `Diagnostic.level: Level`; сравнения со строками
  `"error"`/`"warning"` исчезли из `structural_lint/mod.rs` (`finding`, `run_output`),
  `architecture/runner/mod.rs` и `registry/check.rs`; текст и JSON диагностик не изменились
  (`Guidance.level` получает `level.name()`).
- VAC-D (`8059224`, AC3b): `enum JobState` (Completed, Running, Orphaned, StartingOrFinishing,
  Interrupted, Stopping, Cancelled; serde kebab-case) и запись `Observation { state, value }` в
  `command_runner/jobs/mod.rs`; `observe_once`/`grouped` возвращают типизированное состояние, `observe`
  сравнивает `JobState::Interrupted` вместо строки в JSON; поле `state` пишется той же вставкой
  (`json!(state)`), порядок ключей сохранён (serde_json `preserve_order`). Строки состояний живут только
  в этом файле (grep по Project).
- VAC-E (`029eed3`, AC3c): `command_line::Command` (16 вариантов + `Jobs(jobs::Command)`) с одной
  таблицей имён `NAMES` (`parse` для `main.rs` вместо `owns()` + особых случаев guard-*, `name()` для
  сообщений `takes no arguments`/`unknown scaffold command`); `run`/`validate_arguments`/
  `configured_command` сопоставляют варианты; `jobs::Command { List, Status, Logs, Stop, Cleanup }`,
  `jobs::cli` принимает его, ветка `unknown jobs command` исчезла (исчерпывающий match). Строки команд
  остались только на границе разбора (`main.rs`: hook/lint-*, `project_root` для `init`) и в
  `structural_lint::cli` — вне карточки.
- VAC-F (`96a4a57`, AC3d): `enum Catalog { Plan, Backlog, Decisions, Invariants }` в
  `project_memory/format.rs` с `directory()`, `prefix()`, `columns()`, `detail_sections() -> Sections
  { required, optional }`; свободные `columns(char)`/`detail_sections(&str)` с ветками `bail!` исчезли;
  `table`/`parse_table`/`parse_row` принимают каталог; потребители — `project_memory/mod.rs`
  (`details`, `reject_unindexed`, все вызовы `table`), `history.rs`, `project_setup/assets.rs memory()`.
  Догфуд поймал кортеж в новой сигнатуре `detail_sections` (tuple-as-record; прежняя была скрыта в
  `Result<(..)>` — правило не заглядывает в `Result`, побочное наблюдение для отчёта) — заменён записью
  `Sections`.
- VAC-G (этот коммит, AC4): семь пар стали записями у владельцев — `format::Link { name, path }`
  (`parse_row`, `invariants`), `project_config::settings::ModuleFile { path, source }` (`files`,
  потребитель `project_setup::bundle`), `registry/check.rs Finding { path, message }` (`Findings`),
  `architecture::policy::Description { purpose, role }` (`describe`, потребитель `registry::role`),
  `template::Justfile { path, source }` (`justfiles`, потребитель `add_runtime`),
  `streams::Logs { stdout, stderr }` (`Readers::start`, `Job::open_logs`); импорт `File` в `jobs/mod.rs`
  стал лишним и удалён. Эти пары прятались за `Result<(..)>`, `Option<(..)>` и псевдонимом типа —
  правило tuple-as-record их не видело (наблюдение для отчёта).
- Не сделано: AC5 (D108 + маркеры), ротация P066/B062, пин D051, `just feature-merge`.

## Verification

- 2026-11-05, рабочее дерево VAC-A: `cargo fmt` применён; `tooling/worker/run run clippy` exit 0;
  `--all-targets` чисто; догфуд `candidate -- lint` — 0 находок; `cargo test --lib` 128 passed;
  pytest `agent_hooks/tests/test_hooks.py` + `change_delivery/git/tests/test_git.py` — 44 passed
  (первая редакция теста с `# don't` не падала: shell_words, как sh, считает `#` комментарием —
  заменена на ANSI-C-кавычки; `sh -c "printf %s $'it\'s'"` печатает `it's`). Гейт `384936a` прошёл
  (11 rust-test, 215 pytest).
- 2026-11-05, рабочее дерево VAC-B: `tooling/worker/run run candidate -- bogus-command` → exit 2, одна
  строка `ERROR [bogus-command] …: unknown command "bogus-command"; run agentrig --help for the command
  table. ACTION: … RERUN: …`; без команды — `unknown command ""` с той же отсылкой; `--help` печатает
  таблицу. Гейт `4d2e48a` прошёл (128 rust-test, 27 pytest).
- 2026-11-05, рабочее дерево VAC-C: `cargo fmt` применён; профиль clippy exit 0; `--all-targets` чисто;
  догфуд 0; `cargo test --lib` 128 passed; `candidate -- lint --json --warnings` печатает
  `"level":"warning"` как прежде; pytest `structural_lint/tests/test_lint.py` +
  `registry/tests/test_registry.py` — 22 passed. Гейт `a983c85` прошёл (116 rust-test, 370 pytest).
- 2026-11-05, рабочее дерево VAC-D: `cargo fmt` применён; профиль clippy exit 0; `--all-targets` чисто;
  догфуд 0; `cargo test --lib` 128 passed; pytest `command_runner/tests/test_jobs.py` — 13 passed
  (утверждения по `state`: running, completed, orphaned, interrupted, cancelled, `group_observation`);
  `candidate -- job-status run-BTBLbq` печатает `…"background":false,"state":"completed","age_seconds":…`
  — позиция и значение ключа прежние. Гейт `8059224` прошёл (15 rust-test, 209 pytest).
- 2026-11-05, рабочее дерево VAC-E: `cargo fmt` применён; профиль clippy exit 0; `--all-targets` чисто;
  догфуд 0 (с `--warnings` — 16 прежних file-size, среди них `jobs/mod.rs` 308 > 300 после VAC-D, и
  directory-size `.`/Ledger; ошибок нет); `cargo test --lib` 128 passed; пробы кандидата: `doctor extra`
  → `doctor takes no arguments`, `job-status` → `job-status RUN_ID`, `feature-start` → `feature-start
  NAME`, `jobs extra` → `jobs takes no arguments`, `nope` → строка AC6, `commands` печатает каталог —
  exit 2/0 как прежде; pytest `command_runner/tests/test_commands.py` + `test_jobs.py` — 18 passed.
  Гейт `029eed3` прошёл (128 rust-test, 209 pytest).
- 2026-11-05, рабочее дерево VAC-F: `cargo fmt` применён; профиль clippy exit 0; `--all-targets` чисто;
  догфуд 0 (первая редакция дала tuple-as-record `format.rs:48` — исправлено); `cargo test --lib` 128
  passed; `candidate -- memory-check` на репозитории exit 0; pytest `project_memory/tests` +
  `project_setup/tests` — 90 passed. Гейт `96a4a57` прошёл (11 rust-test, 211 pytest).
- 2026-11-05, рабочее дерево VAC-G: `cargo fmt` применён (первый прогон clippy упал на неиспользуемом
  импорте `File` — удалён); профиль clippy exit 0; `--all-targets` чисто; догфуд 0; `cargo test --lib` 128
  passed; `candidate -- memory-check` exit 0; `candidate -- commands` печатает строки ролей как прежде
  (логика `describe` не менялась); pytest `command_runner/tests` + `registry/tests` +
  `project_memory/tests/test_memory.py` + `project_setup/tests` — 42 passed. Гейт коммита — следующий шаг.

## Blockers

Нет.

## Next action

Закоммитить VAC-G через гейт («P066 VAC-G: name the record pairs (B062 AC4)»); затем VAC-H: карточка
D108 (Release в `mark_exited` / Acquire в `wait_readable`, расширяет D095) + строка в Decisions.md +
`// DECISION: D108` над `streams.rs` 69 и 146; ротация P066/B062 в Archive; пин D051; `just feature-merge`.
Свободные ID: D108+, I022+, P069+, B065+.
