# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [P037](Plan/037.md)
([B026](Backlog/026.md)): линты безопасности. 1.1 обоснование `unsafe` ссылкой (влито,
D065/D066), 1.2 Bandit-группа ruff `S` (влито, D064), 1.3 скан хардкод-секретов по
паттернам (в работе; пользователь выбрал «правило по паттернам» 2026-10-30).

## Workspace

Ветка: feature/security-lint (от master `f4f853d`). Коммиты: `474ff9b` VAC-A (ruff `S`, D064);
`2ca56c9` VAC-B1 (машинерия `unsafe-justification`); `a4f1c80` VAC-B2 (go-live `unsafe-justification`,
D065/D066). `tooling/distribution/stable.txt` = `2ca56c9` (промежуточный пин, НЕ закоммичен).

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

VAC-A (1.2) влит (`474ff9b`); VAC-B1+B2 (1.1 `unsafe-justification`) влиты (`2ca56c9`, `a4f1c80`).
VAC-C (1.3) по двухкоммитной схеме. VAC-C1 (A1 enabling) готов в рабочем дереве, не закоммичен:
`Kind::HardcodedSecret` + дескриптор (skill `remove-secret`) + запись в оба handler'а (Rust+Python)
+ матчер `shared::syntax::secret` (фикс-набор: private-key PEM, AWS `AKIA`, GitHub `ghp_`/…/
`github_pat_`, Google `AIza`, Slack `xox[bpar]-`) + общий хелпер `languages::credential` + arm'ы на
`string_literal`/`raw_string_literal` (Rust) и `string` (Python) + поведенческий тест. НЕ в
`catalog::ALL`, НЕ в `.agentrig/lint.yaml`, без маркеров, скилл не создан — бинарь узнаёт Kind через
пин, прежде чем lint.yaml сошлётся. Тест строит образцы конкатенацией, чтобы ни один литерал в
репозитории не был полным паттерном (A2 остаётся зелёным).

## Verification

VAC-A/B влиты полным гейтом. VAC-C1: `cargo test structural_lint::languages` 10 passed (новый
`rust_flags_literals_matching_known_credential_formats`), `cargo clippy` чисто. Полный гейт — на
коммите C1.

## Blockers

Нет.

## Next action

1) Закоммитить VAC-C1 через хук. 2) Запинить head C1 в `tooling/distribution/stable.txt` + `just
bootstrap` (один scratch; пин НЕ коммитить). 3) VAC-C2 (A2 go-live): `Kind::HardcodedSecret` в
`catalog::ALL`; правило `hardcoded-secret` в `.agentrig/lint.yaml` (Level/error, `.rs`+`.py`/`.pyi`,
`# DECISION: D067`); сгруппировать `refactor-*` в `refactor/` (+symlink-алиасы) ради места под новый
скилл; создать скилл `remove-secret` (top-level); обновить `assets.rs` (пути refactor + remove-secret);
маркеры `// DECISION: D067`; записать D067; refresh State; коммит (догфуд: 0 находок). 4) Финал P037:
финальный пин (D051) + коммит пина, архив P037/B026, `just feature-merge`. Не пушить, ветку сохранить.
