# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055–P063 слиты (последний merge `95586fa`).
**P064** (B057, политика запрещённых путей и владелец `shared/fs`; [D105](Decisions/105.md)): VAC-A —
поле `disallowed` в настройках правила и проверка через резолвер — завершается этим коммитом; затем
мид-бранч пин D051, VAC-B (владелец `shared/fs`, миграция продакшн-сайтов), VAC-C (записи в
`lint.yaml`, проба, ротация, финальный пин, merge).

## Workspace

Ветка `feature/disallowed-paths` (база master `95586fa`), наблюдаемый HEAD `95586fa` (коммитов на ветке
ещё нет); VAC-A целиком в рабочем дереве. macOS: коммиты/cargo — через `just scratch` с восстановленным
PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; догфуд — `tooling/worker/run run
candidate -- lint` внутри `just scratch`; pytest одного файла — `tooling/worker/run run test -- <файл>`.

## Progress

- VAC-A (этот коммит): `rules/architecture.rs` — `Settings.disallowed: Vec<Disallowed>` (`path`, `reason`,
  `replacement`, `allow`; `deny_unknown_fields`), `Disallowed::allowed()` компилирует allow-глобы через
  `shared::paths::patterns::globs`, `validate_disallowed` (литеральный `::`-путь, непустые reason и
  replacement, валидные глобы), поле в `describe()`; `architecture::bounded` публичен, фасад реэкспортирует
  `Disallowed`; `runner/disallowed.rs::Policy` проверяет разрешённый внешний путь из
  `runner/analysis.rs::dependencies`; README правила (70 строк) описывает поле; `shared/paths` бюджет
  зависимых 12 → 13 (`rules/` валидирует глобы, D079); тесты `architecture/tests/test_architecture_disallowed.py`;
  [D105](Decisions/105.md) и строка индекса; Plan P064 → active.
- Отброшено: правка примера каталога (`catalog/mod.rs`) — поле необязательно и пусто по умолчанию, другие
  опциональные поля в примере тоже не значатся.
- Не сделано: мид-бранч пин D051 (стабильный бинарник должен принимать `disallowed` до записей в
  `lint.yaml`); VAC-B — владелец `Project/shared/fs` (D106: обёртки с путём и действием в ошибке и `kind()`,
  реэкспорты `File`/`OpenOptions`/`Permissions`/`PermissionsExt`/`OpenOptionsExt`/`symlink`,
  `remove_dir_all` сохраняет имя для D102), регистрация в `shared/mod.rs` и `shared/architecture.yaml`,
  миграция всех продакшн-сайтов `std::fs`/`std::os::unix::fs`; VAC-C — записи `disallowed:` для `std::fs` и
  `std::os::unix::fs` в `.agentrig/lint.yaml` (allow: `Project/shared/fs/**`, `**/tests/**`, `**/tests.rs`),
  догфуд 0, живая проба, ротация P064/B057 в Archive, финальный пин, `just feature-merge` (ветку сохранить,
  не пушить).

## Verification

- 2026-11-05 на рабочем дереве VAC-A: `cargo fmt` применён, `cargo clippy --all-targets -D warnings` OK,
  `cargo test --lib` — 126 passed; `just lint-config-check` OK, `just config-check` OK; догфуд кандидата —
  0 находок (exit 0; две находки бюджета зависимых устранены фасадом и бюджетом 13); `lint-rule
  directory-architecture` (кандидат) описывает `disallowed`; pytest `test_architecture_disallowed.py` на
  кандидате — 8 passed (находка `src/a/mod.rs:3` с reason/replacement, локальный модуль `fs` и файл под
  `allow` не отмечены; 7 отказов `lint-config-check`).
- Гейт этого VAC ещё не запускался (коммит — следующий шаг).

## Blockers

Нет.

## Next action

Закоммитить VAC-A через гейт, затем мид-бранч пин D051 (`git rev-parse HEAD > tooling/distribution/stable.txt`
+ bootstrap, коммит «P064 D051 pin…»). Далее VAC-B: прочитать `Ledger/Backlog/057.md` (AC2), создать
`Project/shared/fs` (mod.rs, tests.rs, architecture.yaml), записать D106, мигрировать сайты, держать догфуд 0.
Свободные ID: D106+, I022+, P069+, B065+.
