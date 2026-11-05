# State

## Focus

**P069 / [B065](Backlog/065.md) — `shipped-contract`: поставка `init` и диагностика lint соответствуют записанным
решениям (находки раунда 8).** Авторизация — /goal пользователя 2026-11-05 (бриф D1, закрытый список AC1–AC7). Текущий
VAC 2: AC4 (`rerun` сохраняет `--json`/`--warnings`) и AC5 (находка `configuration` называет путь отсутствующего
навыка).

## Workspace

Branch: `feature/shipped-contract`
Revision: `ea5ba170911f489ab1fefe47d232f6c9c2b7324d`

База — master `72a7e3d`; VAC 1 — `ea5ba17`; VAC 2 в рабочем дереве; stable.txt → `1fad15d`. macOS: cargo нет в PATH
`just write`/`just run` — коммиты и тесты с кандидатом через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH
&& …'`; поставка — `just candidate -- init --root <каталог> …`.

## Progress

- VAC 1 `ea5ba17`: B065 и строка индекса, P069 (active) и `Plan/069.md`, снимок State.
- VAC 2 (рабочее дерево): `structural_lint/mod.rs::rerun_args` добавляет `--json`/`--warnings` вызова; обёртка
  `shared/fs::canonicalize` (D106: действие «resolve» и путь) в `shared/diagnostics/skills.rs`; тесты
  `test_lint.py::test_rerun_reproduces_findings_with_their_flags`, `::test_missing_skill_is_named_in_the_finding`,
  случай `resolve` в `shared/fs/tests.rs`.
- Не начато: AC6, AC1, AC3, AC2, AC7, финальный VAC (Delivery, ротация), пин D051, `just feature-merge`.

## Verification

- 2026-11-05, рабочее дерево VAC 2: `just run test -- Project/structural_lint/tests/test_lint.py` — 22 passed; те же два
  новых теста с `WORKER_BINARY` установленной ревизии `1fad15d` — 2 failed (без изменения падают);
  `just run rust-test -- shared::fs` — 2 passed; `just lint` (кандидат) — без находок.
- VAC 1 `ea5ba17`: гейт pre-commit зелёный (build, lint, memory, rustfmt, clippy, ruff, mypy, test, typos, vulture).

## Blockers

Нет.

## Next action

Закоммитить VAC 2 через хук; затем AC6 (поля JSON в `Project/README.md`), AC1, AC3, AC2, AC7. Свободные ID: D111+,
I022+, P070+, B066+.
