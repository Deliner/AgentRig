# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Первая — B027
([P035](Archive/Plan/035.md)) — завершена; идёт её финальный коммит и влитие.

## Workspace

Branch: feature/external-profiles (от master). Revision: VAC-A `a9875fa`, VAC-B `72299fa`,
VAC-C `0a50b68`, VAC-D `ace5511` committed; финальный коммит (D060 + архивация + пин) —
текущий. Пин `tooling/distribution/stable.txt` = `ace5511`, бустрап выполнен (D051).

Машина — macOS. Сборка/clippy/коммит — через `just scratch -- '...'` с PATH; `cargo`
добавлен в `~/.zshenv` (`~/.cargo/bin`), поэтому `just bootstrap`/`!`-шелл его видят.

## Progress

B027 доставлен (dev-гейт репозитория):

- clippy (`.agentrig/commands.yaml`) — `-D warnings` + паник-риск-deny
  (`panic`/`todo`/`unimplemented`/`dbg_macro`/`unreachable`/`expect_used`/`unwrap_used`);
  `print_stdout`/`print_stderr` исключены осознанно (CLI-вывод — продукт).
- ruff — добавлен `PERF` (0 правок); `mypy --strict` был. Код зелёный (VAC-A..D).
- [D060](Decisions/060.md) записан: clippy-профиль без print-линтов; `init` внешних тулов
  не поставляет, профиль — улучшение dev-гейта, не init-возможность.
- P035 → [Archive/Plan](Archive/Plan.md), B027 → [Archive/Backlog](Archive/Backlog.md),
  Delivery заполнена; оригиналы `Plan/035.md`/`Backlog/027.md` удалены.

## Verification

VAC-D коммит-гейт целиком зелёный: build/lint/memory/rustfmt/clippy (полный профиль)/
rust-test 77/ruff/ruff-format/mypy/pytest 346/typos/vulture — все PASS. Бустрап `ace5511`
успешен (`agentrig 0.3.0`).

## Blockers

Нет.

## Next action

Закоммитить финал (Ledger-архивация + `stable.txt`). Затем `just feature-merge` (прогонит
affected тесты всей ветки); ветка сохраняется. Затем — следующая карточка набора
(B026, B028–B033) на новой ветке через `feature-start`. Не пушить без просьбы.
