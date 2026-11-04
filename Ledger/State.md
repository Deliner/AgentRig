# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P063 слиты (последний merge `95586fa`).
**P064** (B057, политика запрещённых путей и владелец `shared/fs`; [D105](Decisions/105.md),
[D106](Decisions/106.md)): VAC-A и мид-бранч пин закоммичены; VAC-B1 (владелец `shared/fs` и миграция
`shared/*`, `structural_lint/*`) завершается этим коммитом; далее VAC-B2/B3 (остальные продакшн-сайты),
VAC-C (записи в `lint.yaml`, проба, ротация, финальный пин, merge).

## Workspace

Ветка `feature/disallowed-paths` (база master `95586fa`), наблюдаемый HEAD `7020ea6` (VAC-A `b68140d`,
пин `7020ea6`; stable.txt → `b68140d`, бинарник принимает `disallowed`). macOS: коммиты/cargo — через
`just scratch` с восстановленным PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`;
догфуд — `tooling/worker/run run candidate -- lint` внутри `just scratch`; pytest одного файла —
`tooling/worker/run run test -- <файл>` (форматирование — `tooling/.venv/bin/ruff format --config
tooling/pyproject.toml`).

## Progress

- VAC-A (`b68140d`): поле `disallowed` в настройках `directory-architecture`, `Disallowed::allowed()`,
  валидация, описание, `runner/disallowed.rs::Policy` из `runner/analysis.rs`, README, бюджет
  `shared/paths` 13, pytest `test_architecture_disallowed.py`, D105. Пример каталога не трогался
  (поле необязательно). Пин `7020ea6`.
- VAC-B1 (этот коммит): лист `Project/shared/fs` (mod.rs: обёртки `read`, `read_to_string`, `write`,
  `open`, `open_with`, `create_dir_all`, `read_dir`, `metadata`, `symlink_metadata`, `read_link`,
  `set_permissions`, `symlink`, `remove_file`, `remove_dir_all`; `Error {action, path, source}` с
  `kind()`; реэкспорт `File`, `OpenOptions`, `OpenOptionsExt`, `Permissions`, `PermissionsExt`; tests.rs;
  architecture.yaml с `dependents: 9`), регистрация в `shared/mod.rs` и `shared/architecture.yaml`;
  мигрированы `shared/{artifacts,paths,vcs,diagnostics,yaml}` и `structural_lint/{mod.rs,
  architecture/policy.rs, architecture/inventory.rs, architecture/runner/analysis.rs, inventory}`;
  [D106](Decisions/106.md) и строка индекса.
- Не сделано: VAC-B2 — `project_memory/*`, `project_config/settings/validation.rs`, `project_setup/*`,
  `command_line/main.rs`; VAC-B3 — `command_runner/*` (scratch, registry, jobs), `agent_hooks/*`
  (включая inline-тест в `input.rs`); бюджет `shared/fs` поднимать до фактического числа зависимых;
  VAC-C — записи `disallowed:` для `std::fs` и `std::os::unix::fs` в `.agentrig/lint.yaml`
  (allow: `Project/shared/fs/**`, `**/tests/**`, `**/tests.rs`), догфуд 0, живая проба, ротация
  P064/B057 в Archive, финальный пин, `just feature-merge` (ветку сохранить, не пушить).

## Verification

- 2026-11-05 на рабочем дереве VAC-B1: `cargo fmt` применён, `cargo clippy --all-targets -D warnings` OK,
  `cargo test --lib` — 128 passed (2 новых в `shared/fs/tests.rs`: формулировка «<действие> <путь>»,
  `kind()`, цепочка source, поведение обёрток); догфуд кандидата — 0 находок (exit 0).
- Гейт VAC-A прошёл (`b68140d`: 369 pytest, 106 rust-test по выборке); гейт этого VAC ещё не
  запускался (коммит — следующий шаг).

## Blockers

Нет.

## Next action

Закоммитить VAC-B1 через гейт, затем VAC-B2: прочитать и мигрировать `project_memory/{format,resume,
source,mod,history}.rs`, `project_memory/evidence/{mod,content}.rs`, `project_config/settings/validation.rs`,
`project_setup/{claude,doctor,mod}.rs`, `project_setup/initialization/{init,wizard}.rs`,
`command_line/main.rs`; поднять `dependents` в `shared/fs/architecture.yaml`; проверки; коммит.
Свободные ID: D107+, I022+, P069+, B065+.
