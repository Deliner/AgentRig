# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [P037](Plan/037.md)
([B026](Backlog/026.md)): линты безопасности. 1.1 обоснование `unsafe` ссылкой, 1.2
Bandit-группа ruff `S` (влито), 1.3 скан хардкод-секретов.

## Workspace

Ветка: feature/security-lint (от master `f4f853d`). Коммиты: `474ff9b` VAC-A (ruff `S`, D064);
`2ca56c9` VAC-B1 (машинерия `unsafe-justification`, не активно). `tooling/distribution/stable.txt`
= `2ca56c9` (промежуточный пин VAC-B1, НЕ закоммичен; установленный бинарь уже знает Kind).

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

VAC-A (1.2) влит (`474ff9b`); VAC-B1 (enabling) влит (`2ca56c9`), запинен+забустрапен.
VAC-B2 (go-live) готов в рабочем дереве, не закоммичен: скилл-группа `references` (replace-comment
перемещён под неё + symlink-алиас; создан `justify-unsafe/` SKILL.md+architecture.yaml + алиас);
`Project/shared/skills/architecture.yaml` обновлён; `assets.rs` SKILLS = 22 (пути через
`references/`); `Kind::UnsafeJustification` в `catalog::ALL`; `unsafe-justification` в
`.agentrig/lint.yaml` (Level/error, `.rs`, `# DECISION: D065`); маркеры `// DECISION: D065` на
descriptor.rs/rust.rs/tests; `// DECISION: D066` над всеми 11 `unsafe` (macos.rs ×4, linux.rs ×2,
process.rs ×2, group.rs ×3); D065 (правило) и D066 (обоснование FFI) записаны в Decisions.

## Verification

VAC-B1: полный гейт зелёный (`2ca56c9`), бинарь переустановлен и знает Kind. VAC-B2: проверяется
гейтом этого коммита (build candidate с `catalog::ALL`+assets, `just lint` установленным бинарём по
live-`lint.yaml` — 0 находок, т.к. все 11 `unsafe` помечены; rust-тесты + pytest, включая I020
delivered==live по `skills`/`rule-kinds`).

## Blockers

Нет.

## Next action

1) Застейджить изменения VAC-B2 и закоммитить через хук (гейт — единственная проверка; не гонять
gate повторно до коммита). 2) VAC-C (1.3): оценить статическую осуществимость скана секретов;
правило ИЛИ norm-or-choice. 3) Финал P037: финальный пин (D051: head → stable.txt + bootstrap одним
scratch) + коммит пина, архив P037/B026, `just feature-merge`. Не пушить, ветку сохранить.
