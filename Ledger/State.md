# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [P037](Plan/037.md)
([B026](Backlog/026.md)): линты безопасности. 1.1 обоснование `unsafe` ссылкой (влито,
D065/D066), 1.2 Bandit-группа ruff `S` (влито, D064), 1.3 скан хардкод-секретов по
паттернам (в работе; пользователь выбрал «правило по паттернам» 2026-10-30).

## Workspace

Ветка: feature/security-lint (от master `f4f853d`). Коммиты: `474ff9b` VAC-A (ruff `S`, D064);
`2ca56c9` VAC-B1; `a4f1c80` VAC-B2 (1.1 `unsafe-justification`, D065/D066); `fc804a2` VAC-C1
(машинерия `hardcoded-secret`). `tooling/distribution/stable.txt` = `fc804a2` (промежуточный пин,
НЕ закоммичен; установленный бинарь знает `HardcodedSecret`).

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

VAC-A (1.2) влит; VAC-B1+B2 (1.1) влиты; VAC-C1 (A1 `hardcoded-secret`) влит+запинен+забустрапен.
VAC-C2 (A2 go-live) готов в рабочем дереве, не закоммичен: `Kind::HardcodedSecret` в `catalog::ALL`;
правило `hardcoded-secret` в `.agentrig/lint.yaml` (Level/error, `.rs`+`.py`/`.pyi`, `# DECISION:
D067`); `refactor-*` сгруппированы под `refactor/` (3 `git mv` + symlink-алиасы + `refactor/
architecture.yaml`); скилл `remove-secret` создан top-level (SKILL.md + architecture.yaml); parent
`architecture.yaml` обновлён (`refactor`, `remove-secret`); `assets.rs` — пути refactor + запись
remove-secret; маркеры `// DECISION: D067` на descriptor.rs/shared syntax/languages mod.rs/tests;
D067 записан в Decisions (+ карточка 067.md). Скилл-директория: 12 реальных top-level + architecture
+ remove-secret = 14 ≤ 15.

## Verification

VAC-A/B/C1 влиты полным гейтом. VAC-C2 — проверяется гейтом этого коммита: build candidate
(`catalog::ALL`+assets), `just lint` установленным (C1) бинарём по live-`lint.yaml` — 0 находок
`hardcoded-secret` (в коде нет таких литералов; тест строит образцы конкатенацией), rust+pytest,
I020 delivered==live по `skills`/`rule-kinds` (remove-secret + hardcoded-secret с обеих сторон).

## Blockers

Нет.

## Next action

1) Застейджить VAC-C2 (без stable.txt) и закоммитить через хук. 2) Финал P037: финальный пин (D051:
head → stable.txt + bootstrap одним scratch) + коммит пина; архив P037/B026 (Plan/037 → Archive,
Backlog/026 → Archive, индексы, ссылки D064–D067 на Archive/Backlog/026.md); `just feature-merge`;
отчёт по-русски. Не пушить, ветку сохранить. 3) Дальше — карточки B028, B030–B033 по догфудингу.
