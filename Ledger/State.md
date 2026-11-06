# State

## Focus

**P070 / [B067](Backlog/067.md) — `delivery-repairs`: поставка D1 без расхождений с контрактом B065 и D103 (ремонты
по критике D1).** Авторизация — /goal пользователя 2026-11-05 (бриф D3, закрытый список AC1–AC5). Текущий — VAC 4:
AC2, ошибка конфигурации проектного режима под `lint --json` — JSON-находка `configuration`.

## Workspace

Branch: `feature/delivery-repairs`
Revision: `3b0d8740f70cb440c15b57bbc6119646f41ca041`

База — master `44613fd` (слияние P069; stable.txt и установленная ревизия — `e3fc0c6`). VAC 1 — `da1a126`, VAC 2 —
`52b39d4`, VAC 3 — `3b0d874`; VAC 4 в рабочем дереве.
macOS: в PATH `just write`/`just run` нет cargo — коммиты и проверки с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- P069 (B065) слит в master `44613fd` и заархивирован; критика D1 дала закрытый список — B067 AC1–AC5.
- VAC 1 `da1a126`: карточка B067 и строка индекса (`Backlog.md` оканчивается переводом строки), строка P070
  (active) и `Plan/070.md`, группа `Project/shared/fs/**` → `shared::fs::` и модули-потребители в карте влияния
  `rust-test` (`.agentrig/checks.yaml`) — AC5 в части карты и перевода строки.
- VAC 2 `52b39d4` (AC4): канонический `signature/split-boolean-parameter/SKILL.md` описывает реализованный предикат
  Rust и Python; правило читает только объявления.
- VAC 3 `3b0d874` (AC3): `languages/unsafe_site.rs::attribute_opener` пропускает `unsafe`, стоящий сразу после запятой
  верхнего уровня в аргументах `cfg_attr` и открывающий группу `(..)`; фикстура
  `rust_skips_conditional_unsafe_attribute_openers`; метрика дескриптора и зеркальные фразы `justify-unsafe` называют
  обе формы. Новое решение не нужно: D103 уже выводит атрибуты `#[unsafe(..)]` из правила.
- VAC 4 (рабочее дерево, AC2): `command_line/main.rs::run_lint` откладывает ошибку разрешения проектной конфигурации
  (`project_lint`); `structural_lint/cli.rs::failure` под `--json` печатает находку `configuration` (path —
  `.agentrig/project.yaml`, message — цепочка ошибки, rerun — та же команда проектного режима), код 2; без `--json` и
  для `lint-explain` ошибка уходит прежним путём в stderr.
- Не начато: AC1 (Cargo), финальный VAC (Delivery, ротация P070/B067), пин D051, `just feature-merge`.

## Verification

- 2026-11-06, VAC 1 `da1a126`: гейт зелёный (rust-test `::` 129 passed; test 114 passed). Проба AC5 — файл в
  `Project/shared/fs/` в индексе, `just check --only rust-test --staged`: на карте `44613fd` → `SELECT [rust-test]: []`;
  после `da1a126` — выбор содержит `shared::fs::`, выполнены оба теста `shared::fs::tests` (126 passed); проба снята.
- 2026-11-06, VAC 2 `52b39d4`: гейт зелёный (rust-test не выбран; test 189 passed, включая `test_delivery.py` —
  оракул I020). Проба кандидата подтвердила описанный предикат Python и Rust; `git grep` трёх прежних утверждений:
  `44613fd` — строки 9, 14, 36; дерево — 0.
- 2026-11-06, VAC 3 `3b0d874`: до правки фикстура падала (`[1, 3, 6, 9, 11, 13, 16, 17, 18, 19]` против
  `[11, 13, 16, 17, 18, 19]`); гейт зелёный (rust-test 105 passed; test 284 passed).
- 2026-11-06, VAC 4 (рабочее дерево): `test_project_mode_reports_a_missing_skill_as_a_json_finding` до правки падал
  (`JSONDecodeError`, stdout пуст), после — `test_lint.py` и `test_capabilities.py` 33 passed; rustfmt и clippy чистые.
  Проба кандидата: текстовый вывод проектного режима побайтно прежний (`ERROR [lint] …`, код 2), `--json` — находка
  `configuration` с путём навыка, код 2; режимы с `--config` без изменений.

## Blockers

Нет.

## Next action

Закоммитить VAC 4 через хук; затем VAC AC1 (`project_setup/cargo.rs`: `**` на любую глубину, `..` внутри корня,
`workspace.exclude`; CLI-тесты в `initialization/tests/test_layout.py`). Свободные ID: D112+, I022+, P071+, B068+
(B066 и D111 зарезервированы).
