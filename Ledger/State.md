# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P065 слиты (последний merge `b224444`).
**P066** (B062, одноразовые ремонты собственного кода: AC2 fail-closed + `&Settings`, AC3 enum'ы,
AC4 именованные записи, AC5 `Ordering`, AC6 `--help`): VAC-A (AC2) `384936a`; VAC-B (AC6) `4d2e48a`;
VAC-C (AC3a `Level`) завершается этим коммитом; далее VAC по AC3 (JobState, Command, каталог
памяти), AC4, AC5 + ротация, пин D051, `just feature-merge`; затем P067 (B061), P068 (B063).

## Workspace

Ветка `feature/own-code-repairs` (база master `b224444`), наблюдаемый HEAD `4d2e48a` (VAC-B);
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
- VAC-C (этот коммит, AC3a): `enum Level { Warning, Error }` переехал к владельцу `Diagnostic` в
  `shared/diagnostics/lint.rs` (serde lowercase, `name()`, `Copy`, `Eq`); `structural_lint::config::Level`
  — реэкспорт (конфиг правил читает тот же тип); `Diagnostic.level: Level`; сравнения со строками
  `"error"`/`"warning"` исчезли из `structural_lint/mod.rs` (`finding`, `run_output`),
  `architecture/runner/mod.rs` и `registry/check.rs`; текст и JSON диагностик не изменились
  (`Guidance.level` получает `level.name()`).
- Не сделано: AC3b–AC3d, AC4, AC5, ротация P066/B062, пин D051, `just feature-merge`.

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
  `registry/tests/test_registry.py` — 22 passed. Гейт коммита — следующий шаг.

## Blockers

Нет.

## Next action

Закоммитить VAC-C через гейт («P066 VAC-C: type the diagnostic level (B062 AC3)»); затем VAC-D AC3b:
`enum JobState` в `command_runner/jobs/mod.rs` (`observe_once`/`grouped` возвращают типизированное
состояние, `observe` сравнивает с `JobState::Interrupted`; JSON-поле `state` и порядок ключей
сохраняются — serde_json с `preserve_order`). Свободные ID: D108+, I022+, P069+, B065+.
