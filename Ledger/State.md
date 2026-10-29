# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Сейчас в работе первая —
B027 (строгие профили clippy/ruff), промоутнута в [P035](Plan/035.md).

## Workspace

Branch: feature/external-profiles (от master после влития feature/quality-backlog).
Revision: HEAD ветки до первого коммита VAC-A.

Машина — macOS (Darwin 25.5.0). Merge/бустрап/сборка — через `just scratch -- '...'` с
PATH (`/Users/admin/.cargo/bin:/Users/admin/.local/bin:/opt/homebrew/bin:/usr/bin:/bin`).

## Progress

Замер догфуд-поверхности B027 (clippy restriction как warnings + ruff PERF):
- ruff `PERF` — 0 правок (наш код уже чист).
- clippy restriction — 113 срабатываний: `println!`/`eprintln!`/`print!` = 73 (CLI-вывод,
  легитимно); `unwrap` 17 / `expect` 14 / `unreachable!` 6 = 37 (паник-риск, чиним);
  `panic!`/`todo!`/`unimplemented!`/`dbg!` = 0.

Открытие: шаблон `init` (`Project/project_setup/template.rs`) внешних тулов
(clippy/ruff/mypy) НЕ поставляет — отдаёт встроенный структурный линтер (`CheckKind::Lint`)
и `read`/`write`/`test`. Значит clippy/ruff/mypy — собственный dev-гейт репозитория, не
init-возможность. B027 сужен до улучшения dev-гейта (зафиксировать решением D060); «из
коробки для пользователя» дают карточки-правила движка, не B027.

VAC-A (готов к коммиту, код не меняется → пин не нужен): `PERF` добавлен в
`tooling/pyproject.toml`; нулевые clippy-линты (`panic`/`todo`/`unimplemented`/`dbg_macro`)
добавлены как `-D` в `.agentrig/commands.yaml`; B027→P035 (active) в Plan.

mypy уже `strict = true`; ruff уже содержит `B`/`SIM`/`UP` — менять не нужно.

## Verification

Коммит-гейт (build, lint, memory, rustfmt, clippy, rust-test, ruff/ruff-format/mypy,
pytest, typos, vulture). VAC-A: clippy с новыми нулевыми линтами и ruff с PERF должны быть
зелёными без правок кода; memory-check после промоута P035.

## Blockers

Нет.

## Next action

Закоммитить VAC-A через хук. Затем по одному линту за VAC, гейт зелёным:
VAC-B чинит 6 `unreachable!` → `-D clippy::unreachable`; VAC-C чинит 14 `expect` →
`-D clippy::expect_used`; VAC-D чинит 17 `unwrap` → `-D clippy::unwrap_used`. Затем D060
(профиль без print-линтов + dev-гейт-scoping). Правки трогают Project/-исходники → перед
`feature-merge` один раз пин stable.txt + бустрап (D051), коммит пина, затем merge и
архивация P035/B027. Сайты 37 паник-рисков — в
scratchpad/b027/clippy.txt. Не пушить без просьбы.
