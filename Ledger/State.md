# State

## Focus

На 2026-10-28 P031 — тестовая дисциплина ([B022](Archive/Backlog/022.md), TC1–TC7) —
выполнен. Дисциплина доставлена навыком-нормой `testing-discipline`; оба статических
кандидата (TC1 размещение, TC2 публичная поверхность) проверены на осуществимость при
реализации и отклонены, правил линтера и инвариантов не добавлено. Карточки P031 и
B022 перенесены в Archive/. Осталась интеграция ветки.

## Workspace

Branch: feature/test-discipline

Revision: 75d3a81 (HEAD; VAC2 — норма TC1 зафиксирована, D055)

Машина — macOS (Darwin 25.5.0). Ветка меняет Project (навык в Project/shared/skills,
SKILLS и инструкции в project_setup/assets.rs), поэтому перед merge нужны пин
stable.txt и bootstrap по D051. Bootstrap запускает пользователь из обычного
терминала с PATH — смена пина выключает хуки сессии.

## Progress

VAC1 (d208b78): навык `testing-discipline` (TC1–TC6), отдан потребителю (assets.rs
SKILLS + инструкции) и репозиторию (AGENTS.md); политика D054; I020 18/18.
VAC2 (75d3a81): осуществимость TC1 отклонена при реализации (ребро импорта не
отличает предмет проверки от общей ссылки; пример languages/tests → rules/kind.rs) —
D055 фиксирует TC1 как норму рядом с TC2 и отменяет следствие D054 о правиле; раздел
Enforcement навыка приведён в соответствие. Финальный VAC (в рабочем дереве, не
закоммичен): P031/B022 перенесены в Archive/Plan и Archive/Backlog с Delivery и
сверкой TC1–TC7; Plan и Backlog пусты.

## Verification

Оба VAC прошли коммит-гейт (build, lint, memory, rustfmt, clippy, rust-test,
ruff/ruff-format/mypy, pytest, typos, vulture). Архивный VAC проверяется гейтом при
коммите (memory-check — соответствие индексов и деталей после переноса).

## Blockers

Нет.

## Next action

Закоммитить архивный VAC через гейт. Затем пин по D051: записать принятый гейтом
HEAD ветки в tooling/distribution/stable.txt; пользователь запускает
`cd /Users/admin/projects/AgentRig && PATH="/Users/admin/.cargo/bin:/Users/admin/.local/bin:/opt/homebrew/bin:$PATH" just bootstrap`;
закоммитить пин. Затем `just feature-merge` (прогоняет тесты, затронутые всей
веткой), ветку сохранить.
