# State

## Focus

**P069 / [B065](Backlog/065.md) — `shipped-contract`: поставка `init` и диагностика lint соответствуют записанным
решениям (находки раунда 8).** Авторизация — /goal пользователя 2026-11-05 (бриф D1, закрытый список AC1–AC7). Текущий
VAC 1 (Ledger): карточка B065 и строка индекса, строка P069 и `Plan/069.md`, этот снимок.

## Workspace

Branch: `feature/shipped-contract`
Revision: `72a7e3d3fd640f15573004b6558c702700e043bc`

База — master `72a7e3d` (слияние `feature/norm-facets`); VAC 1 не закоммичен; stable.txt → `1fad15d`. macOS:
манифест — `Project/Cargo.toml`; поставка — `just candidate -- init --root <каталог> …`; pytest одного файла —
`just run test -- <файл>`; Rust-тесты — `just run rust-test -- <фильтр>`.

## Progress

- В рабочем дереве: `Ledger/Backlog/065.md` и строка B065 в `Backlog.md` (текст брифа без изменений), строка P069
  (active) в `Plan.md` и `Plan/069.md`, этот снимок.
- Не начато: AC1–AC7 (код, навыки, README), финальный VAC с Delivery и ротацией, пин D051, `just feature-merge`.

## Verification

Для ветки ещё нет: гейт VAC 1 не запускался.

## Blockers

Нет.

## Next action

Закоммитить VAC 1 через хук pre-commit; затем VAC по порядку брифа: AC4 (`rerun` с `--json`/`--warnings`), AC5 (путь
отсутствующего навыка), AC6 (поля JSON в `Project/README.md`), AC1, AC3, AC2, AC7. Свободные ID: D111+, I022+, P070+,
B066+.
