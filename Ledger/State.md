# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P065 слиты (последний merge `b224444`).
**P066** (B062, одноразовые ремонты собственного кода: AC2 fail-closed + `&Settings`, AC3 enum'ы,
AC4 именованные записи, AC5 `Ordering`, AC6 `--help`): VAC-A (AC2) завершается этим коммитом; далее
VAC по AC6, AC3 (Level, JobState, Command, каталог памяти), AC4, AC5 + ротация, пин D051,
`just feature-merge`; затем P067 (B061), P068 (B063).

## Workspace

Ветка `feature/own-code-repairs` (база master `b224444`), HEAD `b224444` (коммитов ветки ещё нет);
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
- VAC-A (этот коммит): `enters_check` → `Result<bool>`; ошибка разбора слова — отказ хука с причиной
  (слово и причина) через общий путь отказов `shell_event` (`agent_hooks/mod.rs`);
  `guard_reference_with(root, phase, &Settings)` как у `guard_commit`; вызов в `command_line/mod.rs`;
  негативный тест `test_shell_guard_refuses_unreadable_check_entry` в `agent_hooks/tests/test_hooks.py`;
  строка P066 в Plan.md — active. Побочный эффект: слово с непарной кавычкой в обычном аргументе
  (например, `just read -- grep "don't"`) теперь отклоняется с указанием слова — охрана не отличает
  shell-текст от аргумента; писать `don.t`.
- Не сделано: AC6, AC3, AC4, AC5, ротация P066/B062, пин D051, `just feature-merge`.

## Verification

- 2026-11-05, рабочее дерево VAC-A: `cargo fmt` применён; `tooling/worker/run run clippy` exit 0;
  `--all-targets` чисто; догфуд `candidate -- lint` — 0 находок; `cargo test --lib` 128 passed;
  pytest `agent_hooks/tests/test_hooks.py` + `change_delivery/git/tests/test_git.py` — 44 passed
  (первая редакция теста с `# don't` не падала: shell_words, как sh, считает `#` комментарием —
  заменена на ANSI-C-кавычки; `sh -c "printf %s $'it\'s'"` печатает `it's`). Гейт коммита — следующий
  шаг.

## Blockers

Нет.

## Next action

Закоммитить VAC-A через гейт («P066 VAC-A: fail-closed check entry and Settings for guard-reference
(B062 AC2)»); затем VAC-B AC6 (`command_line/main.rs`: одна строка с указанием на `--help`).
Свободные ID: D108+, I022+, P069+, B065+.
