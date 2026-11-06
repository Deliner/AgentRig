# State

## Focus

**P070 / [B067](Backlog/067.md) — `delivery-repairs`: поставка D1 без расхождений с контрактом B065 и D103 (ремонты
по критике D1).** Авторизация — /goal пользователя 2026-11-05 (бриф D3, закрытый список AC1–AC5). Текущий — VAC 3:
AC3, `unsafe-justification` не флагует `unsafe`, открывающий атрибут `#[cfg_attr(cond, unsafe(..))]`.

## Workspace

Branch: `feature/delivery-repairs`
Revision: `52b39d4bb4e1040377f8de55b28f2372871eaaef`

База — master `44613fd` (слияние P069; stable.txt и установленная ревизия — `e3fc0c6`). VAC 1 — `da1a126`, VAC 2 —
`52b39d4`; VAC 3 в рабочем дереве.
macOS: в PATH `just write`/`just run` нет cargo — коммиты и проверки с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- P069 (B065) слит в master `44613fd` и заархивирован; критика D1 дала закрытый список — B067 AC1–AC5.
- VAC 1 `da1a126`: карточка B067 и строка индекса (`Backlog.md` оканчивается переводом строки), строка P070
  (active) и `Plan/070.md`, группа `Project/shared/fs/**` → `shared::fs::` и модули-потребители в карте влияния
  `rust-test` (`.agentrig/checks.yaml`) — AC5 в части карты и перевода строки.
- VAC 2 `52b39d4` (AC4): канонический `signature/split-boolean-parameter/SKILL.md` описывает реализованный предикат
  Rust и Python; правило читает только объявления.
- VAC 3 (рабочее дерево, AC3): `languages/unsafe_site.rs::attribute_opener` пропускает `unsafe`, стоящий сразу после
  запятой верхнего уровня в аргументах `cfg_attr` и открывающий группу `(..)`; фикстура
  `rust_skips_conditional_unsafe_attribute_openers`; метрика дескриптора и зеркальные фразы `justify-unsafe` называют
  обе формы атрибута. Новое решение не нужно: D103 уже выводит атрибуты `#[unsafe(..)]` из правила.
- Не начато: AC2 (JSON в проектном режиме), AC1 (Cargo), финальный VAC (Delivery, ротация P070/B067), пин D051,
  `just feature-merge`.

## Verification

- 2026-11-06, VAC 1 `da1a126`: гейт зелёный (rust-test `::` 129 passed; test 114 passed). Проба AC5 — файл в
  `Project/shared/fs/` в индексе, `just check --only rust-test --staged`: на карте `44613fd` → `SELECT [rust-test]: []`;
  после `da1a126` — выбор содержит `shared::fs::`, выполнены оба теста `shared::fs::tests` (126 passed); проба снята.
- 2026-11-06, VAC 2 `52b39d4`: гейт зелёный (rust-test не выбран; test 189 passed, включая `test_delivery.py` —
  оракул I020). Проба кандидата `init` + `lint --json` подтвердила описанный предикат Python и Rust; `git grep` трёх
  прежних утверждений: `44613fd` — строки 9, 14, 36; дерево — 0.
- 2026-11-06, VAC 3 (рабочее дерево): `just run rust-test -- unsafe_site` до правки — новая фикстура падает
  (`[1, 3, 6, 9, 11, 13, 16, 17, 18, 19]` против `[11, 13, 16, 17, 18, 19]`), после правки — 5 passed.

## Blockers

Нет.

## Next action

Закоммитить VAC 3 через хук; затем VAC AC2 (`command_line/main.rs::run_lint`: ошибка конфигурации проектного режима
под `--json` — JSON-находка `configuration`; CLI-тест). Свободные ID: D112+, I022+, P071+, B068+ (B066 и D111
зарезервированы).
