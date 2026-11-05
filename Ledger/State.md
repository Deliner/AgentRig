# State

## Focus

**P069 / [B065](Backlog/065.md) — `shipped-contract`: поставка `init` и диагностика lint соответствуют записанным
решениям (находки раунда 8).** Авторизация — /goal пользователя 2026-11-05 (бриф D1, закрытый список AC1–AC7). Текущий
VAC 4: AC1 (поставляемые исключения `**/tests/**`, `**/tests.rs` ровно у девяти правил с обещанием в решении).

## Workspace

Branch: `feature/shipped-contract`
Revision: `5f6fda84632e77ad1db36d45bb1e0373837bebe3`

База — master `72a7e3d`; VAC 1 — `ea5ba17`, VAC 2 — `3765724`, VAC 3 — `5f6fda8`; VAC 4 в рабочем дереве; stable.txt →
`1fad15d`. macOS: в PATH `just write`/`just run` нет cargo — коммиты и тесты с кандидатом через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; поставка — `just candidate -- init --root <каталог>`.

## Progress

- VAC 1 `ea5ba17`: B065 и строка индекса, P069 (active) и `Plan/069.md`, снимок State.
- VAC 2 `3765724` (AC4, AC5): `structural_lint/mod.rs::rerun_args` повторяет `--json`/`--warnings`; обёртка
  `shared/fs::canonicalize` (D106, действие «resolve» и путь) в `shared/diagnostics/skills.rs`; тесты
  `test_lint.py::test_rerun_reproduces_findings_with_their_flags`, `::test_missing_skill_is_named_in_the_finding`.
- VAC 3 `5f6fda8` (AC6): `Project/README.md:29` — rule, path, line, symbol, actual, limit, level, message, skill, rerun.
- VAC 4 (рабочее дерево, AC1): поле `Descriptor::exclude` (`TEST_CODE` у panic-path, debug-assertion, wildcard-import,
  tuple-as-record, eager-fallback, discarded-result, disabled-raii, global-state, super-super-reach; у остальных 20 —
  пусто), `catalog::example` выдаёт `exclude` только непустой; тесты
  `test_config_check.py::test_rule_examples_exclude_test_files_only_where_decided`,
  `test_lint_defaults.py::test_shipped_rules_skip_test_files_only_where_decided[python|rust]`.
- Не начато: AC3, AC2, AC7, финальный VAC (Delivery, ротация), пин D051, `just feature-merge`.

## Verification

- 2026-11-05, рабочее дерево VAC 4: `just run test -- …/test_config_check.py …/test_lint_defaults.py` — 36 passed; три
  новых теста с `WORKER_BINARY` ревизии `1fad15d` — 3 failed; `just lint` (кандидат) — без находок.
- VAC 3 `5f6fda8`: гейт зелёный (test 27 passed); строка полей есть в README, в `72a7e3d` — нет (`git grep` exit 1).
- VAC 2 `3765724`: гейт зелёный (rust-test 116, test 372 passed); новые тесты на `1fad15d` — 2 failed; `shared::fs::`
  гейт не выбрал (нет в карте влияния) — `just run rust-test -- shared::fs` вручную: 2 passed.
- VAC 1 `ea5ba17`: гейт зелёный.

## Blockers

Нет.

## Next action

Закоммитить VAC 4 через хук; затем AC3 (`#[unsafe(..)]` в аргументах макроса), AC2 (корни и манифест Rust из
`Cargo.toml`), AC7 (навыки ремонта). Свободные ID: D111+, I022+, P070+, B066+.
