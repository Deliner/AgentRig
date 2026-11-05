# State

## Focus

**P069 / [B065](Backlog/065.md) — `shipped-contract`: поставка `init` и диагностика lint соответствуют записанным
решениям (находки раунда 8).** Авторизация — /goal пользователя 2026-11-05 (бриф D1, закрытый список AC1–AC7). Текущий
VAC 6: AC2 (`init --language rust` выводит корни, крейты и манифест тестов из `Cargo.toml`).

## Workspace

Branch: `feature/shipped-contract`
Revision: `c4f7d2f27e3575eea174d21c3ebda08efab55de9`

База — master `72a7e3d`; VAC 1 — `ea5ba17`, VAC 2 — `3765724`, VAC 3 — `5f6fda8`, VAC 4 — `bf23c83`, VAC 5 —
`c4f7d2f`; VAC 6 в рабочем дереве; stable.txt → `1fad15d`. macOS: в PATH `just write`/`just run` нет cargo — коммиты и
тесты с кандидатом через `just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; поставка —
`just candidate -- init --root <каталог>`.

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
- VAC 6 (рабочее дерево, AC2): новый `project_setup/cargo.rs` (`Layout`: манифест `<source>/Cargo.toml`, иначе
  `Cargo.toml`; корни `[lib]`/`src/lib.rs` и `[[bin]]`/`src/main.rs` пакета и членов workspace с глобами; крейт
  `имя` с `-`→`_`; без манифеста — прежние формулы); `template::scaffold` передаёт его в команду тестов и через
  `bundle` → `lint::template` в правило architecture (`rust_crates` — только непустой); `init.rs`, `wizard.rs`;
  тесты `initialization/tests/test_layout.py::test_rust_init_*` (бинарный крейт, workspace ×2, раскладка AgentRig).
- Не начато: AC7, финальный VAC (Delivery, ротация), пин D051, `just feature-merge`.

## Verification

- 2026-11-06, рабочее дерево VAC 6: `just run test -- Project/project_setup/initialization/tests
  Project/project_setup/tests` — 45 passed; с `WORKER_BINARY` ревизии `1fad15d` четыре теста `rust_init` — 4 failed;
  `just lint` (кандидат) — exit 0; `just check --only` rustfmt, clippy, ruff-format, ruff, mypy, typos, vulture — PASS.
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

Закоммитить VAC 6 через хук; затем AC7 (навыки ремонта), финальный VAC, пин D051, `just feature-merge`. Свободные ID:
D111+, I022+, P070+, B066+.
