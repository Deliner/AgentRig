# State

## Focus

**P069 / [B065](Archive/Backlog/065.md) — `shipped-contract`: поставка `init` и диагностика lint соответствуют
записанным решениям (находки раунда 8).** Авторизация — /goal пользователя 2026-11-05 (бриф D1, закрытый список
AC1–AC7). AC1–AC7 выполнены; текущий — финальный VAC (Delivery в [Archive/Plan/069.md](Archive/Plan/069.md), ротация
P069/B065 в Archive), затем пин D051 и `just feature-merge`.

## Workspace

Branch: `feature/shipped-contract`
Revision: `dacbeb03006466682f14d9776e15490d010bd5aa`

База — master `72a7e3d`; VAC 1 — `ea5ba17`, VAC 2 — `3765724`, VAC 3 — `5f6fda8`, VAC 4 — `bf23c83`, VAC 5 —
`c4f7d2f`, VAC 6 — `57ef5c2`, VAC 7 — `dacbeb0`; финальный VAC в рабочем дереве; stable.txt → `1fad15d`. macOS: в
PATH `just write`/`just run` нет cargo — коммиты и тесты с кандидатом через
`just scratch -- 'export PATH=/Users/admin/.cargo/bin:$PATH && …'`; поставка — `just candidate -- init --root <каталог>`.

## Progress

- VAC 1–7 закоммичены; по каждому AC — результат и свидетельство в Delivery
  [Archive/Plan/069.md](Archive/Plan/069.md): AC4, AC5 — `3765724`; AC6 — `5f6fda8`; AC1 — `bf23c83`; AC3 —
  `c4f7d2f`; AC2 — `57ef5c2`; AC7 — `dacbeb0`.
- Финальный VAC (рабочее дерево): Delivery заполнен; строка и карточка P069 перенесены в `Archive/Plan.md` и
  `Archive/Plan/069.md`, строка и карточка B065 — в `Archive/Backlog.md` и `Archive/Backlog/065.md`; живые строки
  удалены, Plan пуст.
- Не начато: пин D051 (stable.txt → голова ветки после финального VAC, `just bootstrap`), `just feature-merge`.

## Verification

- 2026-11-06, VAC 7 `dacbeb0`: гейт зелёный (rust-test не выбран, test 189 passed, включая `test_delivery.py` —
  I020); `git grep -F` 19 формулировок AC7 — `72a7e3d` и `57ef5c2` 0 из 19, `dacbeb0` 19 из 19.
- 2026-11-06, проба AC5 по сценарию карточки (`init --language rust`, удалён `.agentrig/skills/remove-debug/SKILL.md`):
  `lint --root <r> --config .agentrig/lint.yaml --json` — кандидат называет путь в `message` находки
  `configuration`, `1fad15d` — нет; проектный `lint --json` пишет ошибку текстом в stderr (без JSON), путь назван.
- VAC 6 `57ef5c2`: гейт зелёный (rust-test 11, test 189 passed); четыре теста `rust_init` на `1fad15d` — 4 failed.
- VAC 5 `c4f7d2f`: гейт зелёный (rust-test 104, test 280 passed); фикстура через `init`+`lint --json`: `1fad15d`
  флагует строки [2, 5, 7, 9], кандидат — [7, 9].
- VAC 4 `bf23c83`: гейт зелёный со второй попытки (первая — ruff-format; rust-test 103, test 280 passed); три новых
  теста на `1fad15d` — 3 failed.
- VAC 3 `5f6fda8`: гейт зелёный (test 27 passed); строка полей есть в README, в `72a7e3d` — нет (`git grep` exit 1).
- VAC 2 `3765724`: гейт зелёный (rust-test 116, test 372 passed); новые тесты на `1fad15d` — 2 failed; `shared::fs::`
  гейт не выбрал (нет в карте влияния) — `just run rust-test -- shared::fs` вручную: 2 passed.
- VAC 1 `ea5ba17`: гейт зелёный.

## Blockers

Нет.

## Next action

Закоммитить финальный VAC через хук; затем пин D051 одной командой (`git rev-parse HEAD` → `stable.txt` и
`just bootstrap`), коммит пина, `just feature-merge` (ветку сохранить, без push). Свободные ID: D111+, I022+, P070+,
B066+.
