# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P065 слиты (последний merge `b224444`).
**P066** (B062, одноразовые ремонты собственного кода: AC2 fail-closed + `&Settings`, AC3 enum'ы,
AC4 именованные записи, AC5 `Ordering`, AC6 `--help`): VAC-A (AC2) `384936a`; VAC-B (AC6)
завершается этим коммитом; далее VAC по AC3 (Level, JobState, Command, каталог памяти), AC4, AC5 +
ротация, пин D051, `just feature-merge`; затем P067 (B061), P068 (B063).

## Workspace

Ветка `feature/own-code-repairs` (база master `b224444`), наблюдаемый HEAD `384936a` (VAC-A);
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
- VAC-B (этот коммит): `command_line/main.rs` — ошибка неизвестной команды стала одной строкой
  `unknown command "<имя>"; run agentrig --help for the command table` (таблица команд живёт в
  `print_help`); usage-строка с частичным перечнем удалена.
- Не сделано: AC3, AC4, AC5, ротация P066/B062, пин D051, `just feature-merge`.

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
  таблицу. Гейт коммита — следующий шаг.

## Blockers

Нет.

## Next action

Закоммитить VAC-B через гейт («P066 VAC-B: point unknown commands at --help (B062 AC6)»); затем
VAC-C AC3a: `Level` переезжает в `shared/diagnostics/lint.rs` (владелец `Diagnostic`), `config::Level`
— реэкспорт, сравнения со строками уровней исчезают из `structural_lint/mod.rs`, `architecture/runner`,
`registry/check.rs`. Свободные ID: D108+, I022+, P069+, B065+.
