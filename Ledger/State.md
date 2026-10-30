# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [P037](Plan/037.md)
([B026](Backlog/026.md)): линты безопасности. 1.1 обоснование `unsafe` ссылкой, 1.2
Bandit-группа ruff `S`, 1.3 скан хардкод-секретов. P036 (B029) завершён и влит.

## Workspace

Ветка: feature/security-lint (от master `f4f853d`). VAC-A (1.2 ruff `S`) в рабочем дереве, не
закоммичен; промоушен P037 (строка Plan.md + Plan/037.md) тоже в рабочем дереве.
`tooling/distribution/stable.txt` = `06ef95e` (актуальный, VAC-A не трогает `Project`).

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

P037 начат. VAC-A (1.2): в `tooling/pyproject.toml` `select` += `S`; S101 (assert), S603
(subprocess без shell=True), S607 (partial path) ограничены тест-кодом
(`**/tests/**`, `**/testing/**`, `**/conftest.py`) через `per-file-ignores`. Полная группа
`S` активна на продакшн-Python (shell=True/eval/exec/креды ловятся). Исходно 914 находок —
все три шумовых проверки в тест-коде; продакшн чист. Запись [D064](Decisions/064.md):
dev-гейт, не init-возможность (как D060, ruff в init не поставляется). Осталось закоммитить
VAC-A.

## Verification

`ruff check` с `S` — зелёный. Полный гейт на VAC-A ещё не прогонялся.

## Blockers

Нет.

## Next action

Застейджить VAC-A (pyproject.toml, Decisions.md, Decisions/064.md, Plan.md, Plan/037.md,
State) и закоммитить через гейт. Затем 1.1 обоснование `unsafe` — структурное правило (новый
`Kind`, двухкоммитная схема enabling→пин+бустрап→live): ~11 `unsafe` в `command_runner/jobs`
(identity/macos.rs, linux.rs, process.rs, group.rs) снабдить маркером `DECISION:`/`INVARIANT:`
(одно решение-обоснование FFI). Затем 1.3 скан секретов (оценить осуществимость; norm-or-choice).
По завершении P037 — финальный пин (D051) + архив + `just feature-merge`. Отчёт по-русски. Не
пушить, ветку сохранить.
