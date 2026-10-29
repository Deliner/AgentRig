# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. В работе первая — B027
(строгие профили clippy/ruff), [P035](Plan/035.md).

## Workspace

Branch: feature/external-profiles (от master после влития feature/quality-backlog).
Revision: VAC-A `a9875fa`, VAC-B `72299fa` committed; VAC-C (expect) застейджен.

Машина — macOS (Darwin 25.5.0). Сборка/clippy/fmt/коммит — через `just scratch -- '...'`
с PATH (`/Users/admin/.cargo/bin:/Users/admin/.local/bin:/opt/homebrew/bin:/usr/bin:/bin`).

## Progress

B027 реализуется по одному clippy-линту за VAC, гейт зелёным. Замер: ruff `PERF` — 0
правок; clippy паник-риск — 37 сайтов (`unwrap` 17 / `expect` 14 / `unreachable!` 6);
print-линты (73) исключены осознанно (CLI-вывод). mypy уже strict; ruff уже с B/SIM/UP.

- VAC-A (committed `a9875fa`): `PERF` в ruff; нулевые clippy-линты
  (`panic`/`todo`/`unimplemented`/`dbg_macro`) в `.agentrig/commands.yaml`; B027→P035.
- VAC-B (committed `72299fa`): 6 `unreachable!` устранены → `bail!`/Result; `vcs::export`
  match→именованный if; извлечён `rule_message` (function-lines). `-D clippy::unreachable`.
- VAC-C (готов к коммиту): 14 `expect` устранены — грамматические инварианты tree-sitter
  в `languages::{rust,python}::inspect` → `let … else { return }`; Result-функции
  (`oracles`, `gate::run_check`, `evidence::save`, `architecture::runner`+`scope`,
  `settings::files`, `vcs::export`) → `context`/`ok_or_else`; `runner::diagnostic` →
  `Result` (единственный вызов — map/collect). `-D clippy::expect_used`. fmt+clippy+lint
  локально зелёные.

Открытие (зафиксировать в D060 в финальном VAC): шаблон `init` (`template.rs`) внешних
тулов не поставляет — только структурный линтер + read/write/test; поэтому B027 — улучшение
dev-гейта репозитория, не init-возможность.

## Verification

Локально: `cargo fmt` + clippy с полным профилем (`panic/todo/unimplemented/dbg_macro/
unreachable`) — зелёные, компиляция чистая. Коммит-гейт прогонит affected rust-test и
pytest. VAC-A прошёл гейт целиком (77 Rust, 109 pytest).

## Blockers

Нет.

## Next action

Закоммитить VAC-C. Затем VAC-D: 17 `unwrap` → `-D clippy::unwrap_used`. Затем D060
(профиль без print-линтов + dev-гейт-scoping). Правки
трогают Project/ → перед `feature-merge` один раз пин stable.txt + бустрап (D051), коммит
пина, затем merge и архивация P035/B027. Сайты 37 паник-рисков — в scratchpad/b027/clippy.txt.
Не пушить без просьбы.
