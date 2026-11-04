# State

## Focus

**Программа реализации находок исследовательской дуги (семь раундов) через догфудинг.**
Пользователь поручил (/goal, 2026-11-04) реализовать все находки последовательно и для каждой
убедиться, что она работает. Карточки B052–B064 (B064 — идея, не выбрана), дополнение B049 (AC2);
выбраны в Plan **P055–P068** (порядок — в Plan.md). P055 (`33477fd`), P056 (`8c3142c`), P057 (`fbb0a06`),
P058 (`3d23588`), P059 (`a2dc929`), P060 (`f33d418`) слиты. **P061** (B055, новый `kind: recursive-deletion`,
[D102](Decisions/102.md)) идёт двухфазно: этот VAC-B1 — источник; далее мид-бранч пин и VAC-B2 (проводка, навык,
догфуд, ротация).

## Workspace

Ветка `feature/recursive-deletion` (база master `f33d418`), наблюдаемый HEAD `f33d418` (коммитов в ветке ещё нет);
весь VAC-B1 в рабочем дереве и коммитится этим VAC. macOS: коммиты/cargo — через `just scratch` с восстановленным
PATH (`/Users/admin/.cargo/bin`), манифест — `Project/Cargo.toml`; догфуд — `tooling/worker/run run candidate -- lint`
внутри `just scratch`.

## Progress

- VAC-B1 (этот коммит): `Kind::RecursiveDeletion`, дескриптор `RECURSIVEDELETION` (навык `contain-the-deletion`),
  детектор `languages/recursive_deletion.rs` (имя вызываемого: поле метода, последний сегмент пути или голый
  идентификатор `remove_dir_all`; исключения `justified`/`in_test_context`) на диспетче `call_expression` в
  `rust.rs` и в обзоре тел макросов `scan_fragment` (D094); тест `languages/tests/recursive_deletion.rs`;
  контракты `languages`/`languages/tests`; [D102](Decisions/102.md) + строка индекса; P061 → active.
- Не сделано: мид-бранч пин D051 (stable.txt → голова VAC-B1 + bootstrap, отдельный коммит); VAC-B2 — `catalog::ALL`,
  правила Rust в `registry.rs`, правило `recursive-deletion` в `.agentrig/lint.yaml` (исключения `tests`, только
  после пина), навык `contain-the-deletion` в новой группе `skills/ownership` (туда же `own-the-resource` и
  `own-the-state`: `references` уже на пределе 15 записей; символические ссылки, `assets.rs`, контракты, ссылки
  D092/D096), обоснования `// DECISION: D102` над двумя сайтами `command_runner/scratch.rs:42,53`, догфуд 0, живая
  проба, ротация P061/B055 в Archive, State; затем финальный пин и `just feature-merge` (ветку сохранить, не пушить).

## Verification

- 2026-11-04 на рабочем дереве VAC-B1: `cargo clippy --all-targets -D warnings` OK, `cargo test --lib` — 122 passed
  (AC1: находки строк 2, 3, 4, 7, 10, 11 фикстуры — `std::fs::`, `fs::`, голый вызов, метод `self.store.`,
  `tokio::fs::…await`, внутри `assert!`; не-находки — строка с DECISION выше, `remove_file`, `remove_dir`, тестовый
  модуль); `cargo fmt --check` потребовал одну перестановку в тесте — исправлена, повторно не запускался (гейт
  проверит).
- Гейт VAC-B1 ещё не запускался (коммит — следующий шаг).

## Blockers

Нет.

## Next action

Закоммитить VAC-B1 через гейт (`git add Ledger Project`), затем мид-бранч пин D051 (`git rev-parse HEAD >
tooling/distribution/stable.txt` + bootstrap, коммит «P061 D051 pin…»), затем VAC-B2 по списку выше.
Свободные ID: D103+, I022+, P069+, B065+.
