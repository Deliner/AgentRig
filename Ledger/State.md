# State

## Focus

**P069 / [B065](Backlog/065.md) — `shipped-contract`: поставка `init` и диагностика lint соответствуют записанным
решениям (находки раунда 8).** Авторизация — /goal пользователя 2026-11-05 (бриф D1, закрытый список AC1–AC7). Текущий
VAC 7: AC7 (канонические навыки ремонта совпадают с реализацией).

## Workspace

Branch: `feature/shipped-contract`
Revision: `57ef5c2d657568d74431b28bd44c71a8818a5a44`

База — master `72a7e3d`; VAC 1 — `ea5ba17`, VAC 2 — `3765724`, VAC 3 — `5f6fda8`, VAC 4 — `bf23c83`, VAC 5 —
`c4f7d2f`, VAC 6 — `57ef5c2`; VAC 7 в рабочем дереве; stable.txt → `1fad15d`. macOS: в PATH `just write`/`just run`
нет cargo — коммиты и тесты с кандидатом через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`;
поставка — `just candidate -- init --root <каталог>`.

## Progress

- VAC 1 `ea5ba17`: B065 и строка индекса, P069 (active) и `Plan/069.md`, снимок State.
- VAC 2 `3765724` (AC4, AC5): `structural_lint/mod.rs::rerun_args` повторяет `--json`/`--warnings`; обёртка
  `shared/fs::canonicalize` (D106, действие «resolve» и путь) в `shared/diagnostics/skills.rs`; тесты
  `test_lint.py::test_rerun_reproduces_findings_with_their_flags`, `::test_missing_skill_is_named_in_the_finding`.
- VAC 3 `5f6fda8` (AC6): `Project/README.md:29` — rule, path, line, symbol, actual, limit, level, message, skill, rerun.
- VAC 4 `bf23c83` (AC1): поле `Descriptor::exclude` (`TEST_CODE` у девяти правил, у остальных 20 — пусто),
  `catalog::example` выдаёт `exclude` только непустой; тесты
  `test_config_check.py::test_rule_examples_exclude_test_files_only_where_decided`,
  `test_lint_defaults.py::test_shipped_rules_skip_test_files_only_where_decided[python|rust]`.
- VAC 5 `c4f7d2f` (AC3): `languages/unsafe_site.rs::attribute_opener`; метрика unsafe-justification и навык
  justify-unsafe называют исключение; тест
  `languages/tests/unsafe_site.rs::rust_skips_unsafe_attribute_openers_in_macro_arguments`.
- VAC 6 `57ef5c2` (AC2): новый `project_setup/cargo.rs` (`Layout`: манифест `<source>/Cargo.toml`, иначе
  `Cargo.toml`; корни `[lib]`/`src/lib.rs` и `[[bin]]`/`src/main.rs` пакета и членов workspace с глобами; крейт
  `имя` с `-`→`_`; без манифеста — прежние формулы); `template::scaffold` передаёт его в команду тестов и через
  `bundle` → `lint::template` в правило architecture (`rust_crates` — только непустой); `init.rs`, `wizard.rs`;
  тесты `initialization/tests/test_layout.py::test_rust_init_*` (бинарный крейт, workspace ×2, раскладка AgentRig).
- VAC 7 (рабочее дерево, AC7): девять канонических навыков в `Project/shared/skills` — remove-debug (`dbg!(expr);`
  с эффектом → `expr;`, удаляется только безэффектное; фраза D081 сохранена), anchor-at-crate
  (`pub(in super::super)`, исключение `**/tests/**`/`**/tests.rs`), justify-unsafe/-allow/-skip (единственная
  форма — ссылка строкой выше; `// SAFETY:`, `# Safety`, `reason = ".."`, `#[ignore = ".."]` правило не читает по
  соглашению проекта), split-boolean-parameter (bool-факт флагуется так же, ремонт — именованный тип),
  own-the-resource (`.keep()`/`.into_path()` по имени без проверки получателя), name-the-record (unit `()` как в
  `rust.rs::record_signature`), remove-secret (точные форматы из `shared/syntax/mod.rs`).
- Не начато: финальный VAC (Delivery, ротация P069/B065), пин D051, `just feature-merge`.

## Verification

- 2026-11-06, рабочее дерево VAC 7: `git grep -F` 19 формулировок AC7 — в `72a7e3d` и `57ef5c2` найдено 0 из 19, в
  рабочем дереве 19 из 19; проба кандидата (Rust-код `57ef5c2`, `init` + `lint --json` на фикстуре) подтверждает
  описанные предикаты: флагуются `pub(in super::super)`, `.into_path()` на получателе без очистки, `Vec<((), u8, Foo)>`,
  `#[ignore = ".."]`, `reason = ".."`, `// SAFETY:` и `# Safety` без ссылки, `verbose: bool`, `dbg!`, девять
  литералов-секретов; не флагуются `-> ()`, `Result<(), E>`, `Vec<((), (), ())>`, токены короче порога на символ,
  `PUBLIC KEY`; `just check --only` typos, lint — PASS.
- VAC 6 `57ef5c2`: гейт зелёный (rust-test 11, test 189 passed); четыре теста `rust_init` на `1fad15d` — 4 failed.
- VAC 5 `c4f7d2f`: гейт зелёный (rust-test 104, test 280 passed); фикстура через `init`+`lint --json`: `1fad15d`
  флагует строки [2, 5, 7, 9], кандидат — [7, 9].
- VAC 4 `bf23c83`: гейт зелёный (rust-test 103, test 280 passed); три новых теста на `1fad15d` — 3 failed.
- VAC 3 `5f6fda8`: гейт зелёный (test 27 passed); строка полей есть в README, в `72a7e3d` — нет (`git grep` exit 1).
- VAC 2 `3765724`: гейт зелёный (rust-test 116, test 372 passed); новые тесты на `1fad15d` — 2 failed; `shared::fs::`
  гейт не выбрал (нет в карте влияния) — `just run rust-test -- shared::fs` вручную: 2 passed.
- VAC 1 `ea5ba17`: гейт зелёный.

## Blockers

Нет.

## Next action

Закоммитить VAC 7 через хук (гейт выбирает `test_delivery.py`, I020); затем финальный VAC (Delivery `Plan/069.md`,
ротация P069/B065 в Archive), пин D051, `just feature-merge`. Свободные ID: D111+, I022+, P070+, B066+.
