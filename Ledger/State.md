# State

## Focus

**P070 / [B067](Backlog/067.md) — `delivery-repairs`: поставка D1 без расхождений с контрактом B065 и D103 (ремонты
по критике D1).** Авторизация — /goal пользователя 2026-11-05 (бриф D3, закрытый список AC1–AC5). Текущий — VAC 2:
AC4, навык split-boolean-parameter описывает реализованный предикат.

## Workspace

Branch: `feature/delivery-repairs`
Revision: `da1a1265933292c1a7ad283155595d604319928a`

База — master `44613fd` (слияние P069; stable.txt и установленная ревизия — `e3fc0c6`). VAC 1 — `da1a126`; VAC 2
в рабочем дереве.
macOS: в PATH `just write`/`just run` нет cargo — коммиты и проверки с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; поставка — `just candidate -- <команда> --root <каталог>`.

## Progress

- P069 (B065) слит в master `44613fd` и заархивирован; критика D1 дала закрытый список — B067 AC1–AC5.
- VAC 1 `da1a126`: карточка B067 и строка индекса (`Backlog.md` оканчивается переводом строки), строка P070
  (active) и `Plan/070.md`, группа `Project/shared/fs/**` → `shared::fs::` и модули-потребители в карте влияния
  `rust-test` (`.agentrig/checks.yaml`) — AC5 в части карты и перевода строки.
- VAC 2 (рабочее дерево, AC4): канонический `signature/split-boolean-parameter/SKILL.md` — вызов передаёт любое
  булево значение; правило читает только объявления; предикат Rust и Python по `rust.rs::flag`/`python.rs::flag`.
- Не начато: AC3 (`cfg_attr` unsafe), AC2 (JSON в проектном режиме), AC1 (Cargo), финальный VAC (Delivery, ротация
  P070/B067), пин D051, `just feature-merge`.

## Verification

- 2026-11-06, VAC 1 `da1a126`: гейт зелёный (rust-test `::` 129 passed; test 114 passed). Проба AC5 — файл в
  `Project/shared/fs/` в индексе, `just check --only rust-test --staged`: на карте `44613fd` → `SELECT [rust-test]: []`;
  после `da1a126` — выбор содержит `shared::fs::`, выполнены оба теста `shared::fs::tests` (126 passed); проба снята.
- 2026-11-06, VAC 2 (рабочее дерево): проба кандидата `init` + `lint --json` — Python флагует `flag: bool`,
  `flag: bool = read_flag()`, `flag=True`, `count: int = False`, lambda `flag=False`, не флагует `flag=read_flag()`,
  `bool | None`, `"bool"`; Rust флагует `bool` у fn, сигнатуры трейта и типизированного замыкания, не флагует
  `Option<bool>`, `&bool`, нетипизированное замыкание; вызовы не флагуются. `git grep` трёх прежних утверждений:
  `44613fd` — строки 9, 14, 36; рабочее дерево — 0.

## Blockers

Нет.

## Next action

Закоммитить VAC 2 через хук; затем VAC AC3 (`cfg_attr` в `languages/unsafe_site.rs`, фикстура, дескриптор).
Свободные ID: D112+, I022+, P071+, B068+ (B066 и D111 зарезервированы).
