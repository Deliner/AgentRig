# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P066 слиты (последний merge `806c2a1`).
**P067** (B061, владелец консольного вывода): VAC-A (AC1) — лист `Project/shared/console` с функциями
`stdout`/`stderr`/`prompt`, 73 продакшн-сайта печатают через него, вывод не меняется — завершается этим
коммитом; далее VAC-B (AC2: `-D clippy::print_stdout`/`print_stderr` в профиле, `#![allow]` с D109 у
владельца, проба вне владельца → красный гейт; AC3: решение D109, уточняющее D060; ротация P067/B061),
пин D051, `just feature-merge`; затем P068 (B063).

## Workspace

Ветка `feature/console-owner` (база master `806c2a1`), наблюдаемый HEAD `806c2a1` (коммитов на ветке
ещё нет); stable.txt → `9d03e5a`. macOS: коммиты/cargo — через `just scratch` с восстановленным PATH
(`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; профиль clippy гейта —
`tooling/worker/run run clippy`; догфуд — `tooling/worker/run run candidate -- lint`; pytest одного
файла — `tooling/worker/run run test -- <файл>`.

## Progress

- Обмер: 73 продакшн-сайта в 23 файлах (55 `println!`, 17 `eprintln!`, 1 `print!` + `flush` в
  `wizard.rs::ask`); строка в `structural_lint/rules/architecture.rs:68` — текст контракта, не сайт;
  два сайта в тестах — вне AC.
- Проектный выбор в VAC-A: макросы `out!`/`err!` отвергнуты — догфуд-анализатор архитектуры по
  построению не анализирует атрибут `#[macro_export]` и любой вызов макроса своего крейта («local crate
  macro expansion is not analyzed»), а без экспорта макрос недостижим из бинарей `main.rs`/
  `standalone.rs`. Владелец отдаёт функции: `stdout(impl Display)`, `stderr(impl Display)`,
  `prompt(impl Display) -> io::Result<()>` (`print!` + `flush`). Формы сайтов: `console::stdout("literal")`,
  `console::stdout(value)` (аргумент `{}`/`{name}` с Display-значением), `console::stdout(format!(…))`;
  бинари импортируют `agentrig::shared::console`; `ask` зовёт `console::prompt(format!(…))?`.
- Файлы VAC-A: `shared/console/mod.rs` + `architecture.yaml` (`type: flat`, `dependents: 13` — реальный
  радиус: без строки lint сообщает «depended on by 13 other directories (budget 6)»), `shared/mod.rs`
  (`pub mod console;`), `shared/architecture.yaml` (запись `console`), 23 файла с сайтами; строка P067 в
  Plan.md — active. Миграция — скриптом со сканером скобок (сессионный scratchpad): 7 литералов, 11
  одиночных значений, 54 `format!`; затем 3 `format!("{s}")` со строковым значением (clippy
  `useless_format`) и 4 `format!("{x}")` с Display-значением заменены значением — итог 7/18/47.
- Не сделано: VAC-B (AC2/AC3, D109, ротация), пин D051, `just feature-merge`.

## Verification

- 2026-11-05, рабочее дерево VAC-A: `cargo fmt --check` чисто; `tooling/worker/run run clippy` exit 0;
  `--all-targets` чисто; догфуд `candidate -- lint` — 0 находок (с удалённой строкой `dependents` —
  единственная находка 13 > 6, строка восстановлена); на итоговом дереве `cargo test --lib` 128 passed и
  полный pytest `run test -- Project -q` — 435 passed (job `run-JgfYFU`; CLI/hook/lint-наборы сравнивают
  текст stdout/stderr — вывод не изменился); `candidate -- memory-check` exit 0. Гейт коммита VAC-A —
  следующий шаг.

## Blockers

Нет.

## Next action

Закоммитить VAC-A через гейт («P067 VAC-A: route console output through one owner (B061 AC1)»);
затем VAC-B: флаги в `clippy` профиля
`.agentrig/commands.yaml`, `// DECISION: D109` + `#![allow(clippy::print_stdout, clippy::print_stderr)]`
в `shared/console/mod.rs`, `Ledger/Decisions/109.md` + строка индекса, проба `println!` вне владельца →
`run clippy` красный, ротация P067/B061, State; пин D051 + bootstrap отдельным коммитом;
`just feature-merge` (ветку сохранить); затем `just feature-start` для P068 (B063, Plan/068.md).
Свободные ID: D109+, I022+, P069+, B065+.
