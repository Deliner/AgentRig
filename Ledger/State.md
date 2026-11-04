# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P063 слиты (последний merge `95586fa`).
**P064** (B057, политика запрещённых путей и владелец `shared/fs`; [D105](Decisions/105.md),
[D106](Decisions/106.md)): VAC-A, мид-бранч пин, VAC-B1, VAC-B2 и VAC-B3 закоммичены; VAC-C (записи в
`lint.yaml`, живая проба, ротация P064/B057 в Archive) завершается этим коммитом; далее финальный пин
D051 и `just feature-merge`; затем `just feature-start` для P065 (B060).

## Workspace

Ветка `feature/disallowed-paths` (база master `95586fa`), наблюдаемый HEAD `03d803c` (VAC-A `b68140d`,
пин `7020ea6`, VAC-B1 `324326e`, VAC-B2 `8731520`, VAC-B3 `03d803c`; stable.txt → `b68140d`, бинарник
принимает `disallowed`). macOS: коммиты/cargo — через
`just scratch` с восстановленным PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`;
догфуд — `tooling/worker/run run candidate -- lint` внутри `just scratch`; pytest одного файла —
`tooling/worker/run run test -- <файл>` (форматирование — `tooling/.venv/bin/ruff format --config
tooling/pyproject.toml`).

## Progress

- VAC-A (`b68140d`): поле `disallowed` в настройках `directory-architecture`, `Disallowed::allowed()`,
  валидация, описание, `runner/disallowed.rs::Policy` из `runner/analysis.rs`, README, бюджет
  `shared/paths` 13, pytest `test_architecture_disallowed.py`, D105. Пример каталога не трогался
  (поле необязательно). Пин `7020ea6`.
- VAC-B1 (`324326e`): лист `Project/shared/fs` (mod.rs: обёртки `read`, `read_to_string`, `write`,
  `open`, `open_with`, `create_dir_all`, `read_dir`, `metadata`, `symlink_metadata`, `read_link`,
  `set_permissions`, `symlink`, `remove_file`, `remove_dir_all`; `Error {action, path, source}` с
  `kind()`; реэкспорт `File`, `OpenOptions`, `OpenOptionsExt`, `Permissions`, `PermissionsExt`; tests.rs;
  architecture.yaml с `dependents: 9`), регистрация в `shared/mod.rs` и `shared/architecture.yaml`;
  мигрированы `shared/{artifacts,paths,vcs,diagnostics,yaml}` и `structural_lint/{mod.rs,
  architecture/policy.rs, architecture/inventory.rs, architecture/runner/analysis.rs, inventory}`;
  [D106](Decisions/106.md) и строка индекса.
- VAC-B2 (`8731520`): мигрированы `project_memory/{format,resume,source,mod,history}.rs`,
  `project_memory/evidence/{mod,content}.rs`, `project_config/settings/validation.rs`,
  `project_setup/{claude,doctor,mod}.rs`, `project_setup/initialization/{init,wizard}.rs`,
  `command_line/main.rs` (install пишет через `fs::open_with`; `source.rs` без дублирующего контекста
  пути); бюджет `shared/fs` поднят до 15.
- VAC-B3 (`03d803c`): мигрированы `command_runner/scratch.rs`, `command_runner/registry/{check,mod}.rs`,
  `command_runner/jobs/{background,stop,streams,mod,storage,logs}.rs`,
  `command_runner/jobs/identity/linux.rs` (cfg linux; формы вызовов сохранены, на macOS не
  компилируется), `agent_hooks/{transcript,reminder,input}.rs` (inline-тест `input.rs` пишет через
  `fs::write`); `OpenOptions…open` заменён на `fs::open_with`, `File::open` — на `fs::open`; бюджет
  `shared/fs` поднят до 20. Grep: вне `shared/fs/mod.rs` и тестовых файлов ссылок на `std::fs` и
  `std::os::unix::fs` не осталось.
- VAC-C (этот коммит): записи `disallowed` для `std::fs` и `std::os::unix::fs` в `.agentrig/lint.yaml`
  (allow `Project/shared/fs/**`, `**/tests/**`, `**/tests.rs`; flow-стиль — блочная форма дала 506
  непустых строк и находку file-size, теперь 494); ротация `Plan/064.md` → `Archive/Plan/064.md`
  (Delivery, ссылки `../../Decisions/`), `Backlog/057.md` → `Archive/Backlog/057.md`, строки индексов,
  заметка в Plan.md, ссылки `card` в строках D105/D106.
- Не сделано: финальный пин D051 (stable.txt → голова ветки + bootstrap, отдельный коммит) и
  `just feature-merge` (ветку сохранить, не пушить); затем P065 (B060) на новой ветке.

## Verification

- 2026-11-05 на рабочем дереве VAC-C: `just lint-config-check` OK (stable-бинарь принимает записи),
  догфуд кандидата — 0 находок (exit 0); живая проба в `project_setup/claude.rs` (`use
  std::os::unix::fs::PermissionsExt` + `std::fs::metadata`) → `ERROR [architecture]` 2 локации (строки 37,
  38: «… is disallowed here: filesystem errors must name the action and the path; use crate::shared::fs»),
  после отката — 0; `memory-check` после ротации OK.
- Гейты VAC-A (`b68140d`: 369 pytest, 106 rust-test), VAC-B1 (`324326e`: 369 pytest, 123 rust-test),
  VAC-B2 (`8731520`: 193 pytest, 128 rust-test) и VAC-B3 (`03d803c`: 208 pytest, 15 rust-test по выборке)
  прошли; гейт этого VAC ещё не запускался (коммит — следующий шаг).

## Blockers

Нет.

## Next action

Закоммитить VAC-C через гейт; финальный пин D051 (`git rev-parse HEAD > tooling/distribution/stable.txt`,
`bash tooling/distribution/bootstrap.sh`, коммит «P064 D051 pin…»); `just feature-merge` (ветку
сохранить, не пушить). Затем `just feature-start` для P065 (B060, C-профиль clippy в
`.agentrig/commands.yaml`): прочитать `Ledger/Plan/065.md` и `Ledger/Backlog/060.md`.
Свободные ID: D107+, I022+, P069+, B065+.
