# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. В работе первая — B027
(строгие профили clippy/ruff), [P035](Plan/035.md).

## Workspace

Branch: feature/external-profiles (от master после влития feature/quality-backlog).
Revision: VAC-A committed `a9875fa`; VAC-B (unreachable) застейджен, ещё не закоммичен.

Машина — macOS (Darwin 25.5.0). Сборка/clippy/fmt/коммит — через `just scratch -- '...'`
с PATH (`/Users/admin/.cargo/bin:/Users/admin/.local/bin:/opt/homebrew/bin:/usr/bin:/bin`).

## Progress

B027 реализуется по одному clippy-линту за VAC, гейт зелёным. Замер: ruff `PERF` — 0
правок; clippy паник-риск — 37 сайтов (`unwrap` 17 / `expect` 14 / `unreachable!` 6);
print-линты (73) исключены осознанно (CLI-вывод). mypy уже strict; ruff уже с B/SIM/UP.

- VAC-A (committed `a9875fa`): `PERF` в ruff; нулевые clippy-линты
  (`panic`/`todo`/`unimplemented`/`dbg_macro`) в `.agentrig/commands.yaml`; B027→P035.
- VAC-B (готов к коммиту): 6 `unreachable!` устранены — `settings::files` и
  `structural_lint` два места → `anyhow::bail!`; `format::columns`/`detail_sections` →
  `Result` (вызовы в `parse_table`, `project_memory::mod`, `assets::memory`→`Result`,
  `bundle`); `vcs::export` match→if (Submodule уже отсечён `ensure!`). Включён
  `-D clippy::unreachable`. fmt+clippy локально зелёные.

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

Закоммитить VAC-B. Затем VAC-C: 14 `expect` → `-D clippy::expect_used`; VAC-D: 17 `unwrap`
→ `-D clippy::unwrap_used`. Затем D060 (профиль без print-линтов + dev-гейт-scoping). Правки
трогают Project/ → перед `feature-merge` один раз пин stable.txt + бустрап (D051), коммит
пина, затем merge и архивация P035/B027. Сайты 37 паник-рисков — в scratchpad/b027/clippy.txt.
Не пушить без просьбы.
