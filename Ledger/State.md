# State

## Focus

На 2026-10-28 исполняется P031 — тестовая дисциплина ([B022](Backlog/022.md),
критерии TC1–TC7). Доставка автономная на ветке feature/test-discipline.
Итог по enforcement: ни один из TC1–TC6 не проверяется статически без ложных
срабатываний при текущих возможностях движка, поэтому все они доставлены
навыком-нормой `testing-discipline`; правил линтера и инвариантов не добавлено.

## Workspace

Branch: feature/test-discipline

Revision: d208b78 (HEAD; VAC1 — навык + D054 отданы)

Машина — macOS (Darwin 25.5.0). Ветка меняет Project (навык в
Project/shared/skills, SKILLS в project_setup/assets.rs), поэтому перед merge
нужны пин и bootstrap по D051. Навыки — канонически в Project/shared/skills,
отдаются через .agents/skills и поставку `init`; стражи докфудинга —
Project/shared/testing/test_delivery.py (I020).

## Progress

VAC1 (коммит d208b78): навык `testing-discipline` (TC1–TC6), отдан потребителю
(assets.rs SKILLS + инструкции) и репозиторию (AGENTS.md); политика — D054;
страж I020 считает навык с обеих сторон (18/18). VAC2 (в рабочем дереве, не
закоммичен): осуществимость TC1 проверена при реализации и отклонена — ребро
импорта не отличает предмет проверки от попутной общей ссылки (пример:
structural_lint/languages/tests/mod.rs использует общий тип
structural_lint/rules/kind.rs); правило дало бы ложные срабатывания на общей
оснастке и словаре. Решение D055 фиксирует TC1 как норму рядом с TC2 и отменяет
следствие D054 о правиле+инварианте; раздел Enforcement навыка приведён в
соответствие.

## Verification

VAC1 прошёл коммит-гейт (build, lint, memory, rustfmt, clippy, rust-test,
ruff/mypy, pytest 181, typos, vulture). VAC2 ещё не проходил гейт. Осуществимость
TC1 отклонена по живым рёбрам зависимостей репозитория (все кросс-поддеревные
зависимости тестов идут в оснастку `tests`/`testing` или в общий тип).

## Blockers

Нет.

## Next action

Закоммитить VAC2 (D055 + правка Enforcement навыка + State) через гейт. Затем
архивировать P031 и B022 (edit-plan: строка и карточка Plan/031 и Backlog/022 в
Archive/), сверить приёмку TC1–TC7 в Delivery. Перед merge — пин в stable.txt и
bootstrap (D051; bootstrap запускает пользователь из обычного терминала, т.к.
смена пина выключает хуки), коммит пина, затем `just feature-merge`, ветку
сохранить.
