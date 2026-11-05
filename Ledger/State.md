# State

## Focus

**P069 / [B065](Backlog/065.md) — `shipped-contract`: поставка `init` и диагностика lint соответствуют записанным
решениям (находки раунда 8).** Авторизация — /goal пользователя 2026-11-05 (бриф D1, закрытый список AC1–AC7). Текущий
VAC 3: AC6 (`Project/README.md` перечисляет фактические поля JSON находки).

## Workspace

Branch: `feature/shipped-contract`
Revision: `37657248ad1c33f3dd91da249460f0a789df63b7`

База — master `72a7e3d`; VAC 1 — `ea5ba17`; VAC 2 — `3765724`; VAC 3 в рабочем дереве; stable.txt → `1fad15d`. macOS: cargo нет в PATH
`just write`/`just run` — коммиты и тесты с кандидатом через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH
&& …'`; поставка — `just candidate -- init --root <каталог> …`.

## Progress

- VAC 1 `ea5ba17`: B065 и строка индекса, P069 (active) и `Plan/069.md`, снимок State.
- VAC 2 `3765724`: `structural_lint/mod.rs::rerun_args` добавляет `--json`/`--warnings` вызова; обёртка
  `shared/fs::canonicalize` (D106: действие «resolve» и путь) в `shared/diagnostics/skills.rs`; тесты
  `test_lint.py::test_rerun_reproduces_findings_with_their_flags`, `::test_missing_skill_is_named_in_the_finding`,
  случай `resolve` в `shared/fs/tests.rs`.
- VAC 3 (рабочее дерево): `Project/README.md:29` — rule, path, line, symbol, actual, limit, level, message, skill, rerun.
- Не начато: AC1, AC3, AC2, AC7, финальный VAC (Delivery, ротация), пин D051, `just feature-merge`.

## Verification

- 2026-11-05, VAC 3: `grep` новой строки полей в `Project/README.md` — найдена (стр. 29); `git grep` той же строки в
  `72a7e3d` — exit 1; поля совпадают с `shared/diagnostics/lint.rs::Diagnostic`.
- VAC 2 `3765724`: два новых теста `test_lint.py` с `WORKER_BINARY` ревизии `1fad15d` — 2 failed (без изменения
  падают), с кандидатом — passed; гейт pre-commit зелёный (rust-test 116 passed, test 372 passed); `shared::fs::` гейт
  не выбрал (нет в карте влияния) — `just run rust-test -- shared::fs` вручную: 2 passed.
- VAC 1 `ea5ba17`: гейт pre-commit зелёный.

## Blockers

Нет.

## Next action

Закоммитить VAC 3 через хук; затем AC1 (исключения тестов девяти правил), AC3, AC2, AC7. Свободные ID: D111+,
I022+, P070+, B066+.
