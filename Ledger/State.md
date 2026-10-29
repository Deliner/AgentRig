# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Завершается первая —
B027 (строгие профили clippy/ruff как dev-гейт), [P035](Plan/035.md).

## Workspace

Branch: feature/external-profiles (от master после влития feature/quality-backlog).
Revision: VAC-A `a9875fa`, VAC-B `72299fa`, VAC-C `0a50b68` committed; VAC-D (unwrap) —
текущий коммит.

Машина — macOS (Darwin 25.5.0). Сборка/clippy/fmt/коммит — через `just scratch -- '...'`
с PATH (`/Users/admin/.cargo/bin:/Users/admin/.local/bin:/opt/homebrew/bin:/usr/bin:/bin`).

## Progress

B027 реализуется по одному clippy-линту за VAC, гейт зелёным. mypy уже strict; ruff уже
с B/SIM/UP; VAC-A добавил PERF (0 правок). Профиль clippy теперь полный паник-риск-набор:
`panic`/`todo`/`unimplemented`/`dbg_macro`/`unreachable`/`expect_used`/`unwrap_used`.
Print-линты (73 сайта) исключены осознанно (CLI-вывод — продукт).

- VAC-A `a9875fa`: `PERF` в ruff; нулевые clippy-линты в `.agentrig/commands.yaml`; B027→P035.
- VAC-B `72299fa`: 6 `unreachable!` → `bail!`/Result; `vcs::export` match→именованный if;
  извлечён `rule_message`. `-D clippy::unreachable`.
- VAC-C `0a50b68`: 14 `expect` → `let … else`/`context`/`ok_or_else`; `runner::diagnostic`
  → `Result`. `-D clippy::expect_used`.
- VAC-D (текущий): 17 `unwrap()` → `ok_or_else`/`let-else`/`match` в `agent_hooks::reminder`,
  `command_runner::jobs`, `project_memory`, `project_setup::{assets,doctor,initialization}`,
  `structural_lint::{architecture::{graph,resolution::rust::macros,source::rust::attributes},
  inventory}`; поведение сохранено. `-D clippy::unwrap_used`.

Открытие (зафиксировать в D060 на завершении): шаблон `init` (`template.rs`) внешних тулов
не поставляет — только структурный линтер + read/write/test; поэтому B027 — улучшение
dev-гейта репозитория, не init-возможность.

## Verification

Локально зелёные: `cargo +1.98.1 fmt`; clippy с полным паник-риск-профилем — exit 0,
компиляция чистая; структурный линт (`check --root . --only lint`) — PASS. Коммит-гейт
прогонит affected rust-test и pytest.

## Blockers

Нет.

## Next action

Закоммитить VAC-D. Затем записать [D060] (clippy паник-риск-профиль как dev-гейт, без
print-линтов; init внешних тулов не поставляет), заполнить Delivery P035, заархивировать
P035→Archive/Plan и B027→Archive/Backlog (edit-plan). Правки трогают Project/ → перед
`feature-merge` один раз пин stable.txt + бустрап (D051), коммит пина, затем merge. Далее —
следующая карточка набора (B026, B028–B033). Не пушить без просьбы.
