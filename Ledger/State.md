# State

## Focus

**P070 / [B067](Backlog/067.md) — `delivery-repairs`: поставка D1 без расхождений с контрактом B065 и D103 (ремонты
по критике D1).** Авторизация — /goal пользователя 2026-11-05 (бриф D3, закрытый список AC1–AC5). Текущий — VAC 5:
AC1, обнаружение Cargo в `init --language rust` (`**` на любую глубину, `..` внутри корня, `workspace.exclude`).

## Workspace

Branch: `feature/delivery-repairs`
Revision: `8029148d9f5bdbea35ef6e13cac01f48aea7ade4`

База — master `44613fd` (слияние P069; stable.txt и установленная ревизия — `e3fc0c6`). VAC 1 — `da1a126`, VAC 2 —
`52b39d4`, VAC 3 — `3b0d874`, VAC 4 — `8029148`; VAC 5 в рабочем дереве.
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
- VAC 4 `8029148` (AC2): `command_line/main.rs::run_lint` откладывает ошибку разрешения проектной конфигурации;
  `structural_lint/cli.rs::failure` под `--json` печатает находку `configuration` (path — `.agentrig/project.yaml`,
  message — цепочка ошибки с путём навыка, rerun — та же команда), код 2; текстовый режим прежний.
- VAC 5 (рабочее дерево, AC1): `project_setup/cargo.rs` — сегмент `**` раскрывается в сам каталог и всех потомков
  (`descendants`), компонент `..` в пути цели снимает уровень и отбрасывает путь при выходе за корень (`existing`),
  члены под префиксом `workspace.exclude` отбрасываются (`members`); три CLI-сценария в
  `initialization/tests/test_layout.py`, прежние сценарии AC2 не менялись.
- Не начато: финальный VAC (Delivery `Plan/070.md`, ротация P070/B067 в Archive), пин D051, `just feature-merge`.

## Verification

- 2026-11-06, VAC 1 `da1a126`: гейт зелёный (rust-test `::` 129 passed; test 114 passed). Проба AC5 — файл в
  `Project/shared/fs/` в индексе, `just check --only rust-test --staged`: на карте `44613fd` → `SELECT [rust-test]: []`;
  после `da1a126` — выбор содержит `shared::fs::`, выполнены оба теста `shared::fs::tests` (126 passed); проба снята.
- 2026-11-06, VAC 2 `52b39d4`: гейт зелёный (rust-test не выбран; test 189 passed, включая `test_delivery.py` —
  оракул I020). `git grep` трёх прежних утверждений навыка: `44613fd` — строки 9, 14, 36; дерево — 0.
- 2026-11-06, VAC 3 `3b0d874`: до правки фикстура падала (`[1, 3, 6, 9, 11, 13, 16, 17, 18, 19]` против
  `[11, 13, 16, 17, 18, 19]`); гейт зелёный (rust-test 105 passed; test 284 passed).
- 2026-11-06, VAC 4 `8029148`: до правки `test_project_mode_reports_a_missing_skill_as_a_json_finding` падал
  (`JSONDecodeError`, stdout пуст). Гейт: попытка 1 — `ERROR [architecture]` (`Project/shared/testing` — 11
  зависимых при бюджете 10 из-за нового импорта в `test_lint.py`; импорт заменён локальным `run`), попытка 2 зелёная
  (rust-test 130 passed; test 216 passed).
- 2026-11-06, VAC 5 (рабочее дерево): до правки три новых теста падали (`**` — roots `['crates/c/src/main.rs']`;
  `..` — roots `[]`; exclude — в roots есть `crates/skip/src/lib.rs`); после — `test_layout.py` 16 passed; rustfmt,
  clippy и ruff-format чистые.

## Blockers

Нет.

## Next action

Закоммитить VAC 5 через хук; затем финальный VAC (Delivery в `Plan/070.md`, ротация строк и карточек P070/B067 в
Archive по edit-plan/edit-backlog, State), пин D051 и `just feature-merge`. Свободные ID: D112+, I022+, P071+, B068+
(B066 и D111 зарезервированы).
