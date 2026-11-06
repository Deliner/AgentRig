# State

## Focus

**P070 / [B067](Archive/Backlog/067.md) — `delivery-repairs`: поставка D1 без расхождений с контрактом B065 и D103
(ремонты по критике D1).** Авторизация — /goal пользователя 2026-11-05 (бриф D3, закрытый список AC1–AC5). AC1–AC5
выполнены; текущий — финальный VAC: Delivery и ротация P070/B067 в Archive.

## Workspace

Branch: `feature/delivery-repairs`
Revision: `29fdc07f24a94dc7e927750386d6594449d5d140`

База — master `44613fd` (слияние P069; stable.txt и установленная ревизия — `e3fc0c6`). VAC 1 — `da1a126`, VAC 2 —
`52b39d4`, VAC 3 — `3b0d874`, VAC 4 — `8029148`, VAC 5 — `29fdc07`; финальный VAC в рабочем дереве.
macOS: в PATH `just write`/`just run` нет cargo — коммиты и проверки с cargo через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; поставка — `just candidate <команда> --root <каталог>`.

## Progress

- AC1–AC5 доставлены: VAC 1 `da1a126` (выбор, AC5), VAC 2 `52b39d4` (AC4), VAC 3 `3b0d874` (AC3), VAC 4 `8029148`
  (AC2), VAC 5 `29fdc07` (AC1); свидетельства по AC — Delivery в [Archive/Plan/070.md](Archive/Plan/070.md).
- Финальный VAC (рабочее дерево): Delivery заполнен; строка и карточка P070 перенесены в `Archive/Plan.md` и
  `Archive/Plan/070.md`, строка и карточка B067 без изменений — в `Archive/Backlog.md` и `Archive/Backlog/067.md`;
  живые Plan и Backlog без P070/B067 (Plan пуст).
- Не начато: пин D051 (stable.txt на голову ветки + `just bootstrap`), `just feature-merge`.

## Verification

- 2026-11-06, гейт VAC 1–5 зелёный (rust-test/test passed: 129/114, не выбран/189, 105/284, 130/216, 11/192); VAC 4 —
  со второй попытки (первая: `ERROR [architecture]`, бюджет `dependents` каталога `Project/shared/testing` 11 > 10 из-за
  импорта в `test_lint.py`; импорт заменён локальным помощником, бюджет не менялся).
- 2026-11-06, VAC 5 `29fdc07`: до правки три новых теста `test_layout.py` падали (`**` — только `crates/c/src/main.rs`;
  `..` — `rust_roots: []`; exclude — в корнях `crates/skip/src/lib.rs`), после — 16 passed.
- 2026-11-06, проба AC2 на `29fdc07`-кандидате и `e3fc0c6`: проектный `lint --json` при удалённом навыке — код 2 на
  обоих; база — stdout пуст, текст в stderr; кандидат — JSON-находка `configuration` с путём, stderr пуст; текстовый
  режим одинаков (код 2, stdout пуст, `ERROR [lint] …`).

## Blockers

Нет.

## Next action

Закоммитить финальный VAC через хук; затем пин D051 одной командой (голова ветки → `tooling/distribution/stable.txt`,
`just bootstrap`), коммит пина через хук и `just feature-merge` (ветку сохранить, без push). Свободные ID: D112+, I022+,
P071+, B068+ (B066 и D111 зарезервированы).
