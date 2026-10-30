# State

## Focus

Набор улучшений качества (кластеры 1–8, [B026](Archive/Backlog/026.md),
[B028](Backlog/028.md), [B030](Backlog/030.md)–[B033](Backlog/033.md)) через догфудинг, по
одной карточке. [P037](Archive/Plan/037.md) ([B026](Archive/Backlog/026.md), линты
безопасности) завершён и архивирован в рабочем дереве. Следующая карточка набора — B028.

## Workspace

Ветка: feature/security-lint (от master `f4f853d`). Коммиты: `474ff9b` VAC-A (ruff `S`, D064);
`2ca56c9` VAC-B1; `a4f1c80` VAC-B2 (1.1 `unsafe-justification`, D065/D066); `fc804a2` VAC-C1
(машинерия `hardcoded-secret`); `8c2b451` VAC-C2 (1.3 live, D067). `tooling/distribution/
stable.txt` = `fc804a2` (промежуточный пин, НЕ закоммичен); финальный пин ещё не записан.

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

Все три под-правила P037 доставлены и влиты в ветку: 1.2 ruff `S` (`474ff9b`, D064); 1.1
`unsafe-justification` (`2ca56c9`+`a4f1c80`, D065/D066, 11 FFI-мест помечены); 1.3
`hardcoded-secret` (`fc804a2`+`8c2b451`, D067) — `Kind` в `catalog::ALL`, правило в
`.agentrig/lint.yaml`, матчер `shared::syntax::secret`, навык `remove-secret`, `refactor-*`
сгруппированы под `refactor/`. Архивация выполнена в рабочем дереве (не закоммичена): карточки
`026.md`/`037.md` перенесены в `Archive/Backlog`/`Archive/Plan` (относительные ссылки
исправлены, у P037 добавлен Delivery), индексы Backlog/Plan и их архивы обновлены, ссылки card
в D064/D065/D067 переведены на `Archive/Backlog/026.md`. Plan.md пуст (валидно).

## Verification

VAC-C2 (`8c2b451`) — полный гейт зелёный: build/lint/memory/rustfmt/clippy/rust-test 85/ruff/
ruff-format/mypy/pytest 304 (вкл. I020 delivered==live)/typos/vulture. Архивный набор —
только Ledger; memory-check проверяется гейтом архивного коммита.

## Blockers

Нет.

## Next action

1) Закоммитить архивацию (без `tooling/distribution/stable.txt`) через хук. 2) Финальный пин
([D051](Decisions/051.md)): записать голову архивного коммита в `tooling/distribution/
stable.txt` + `just bootstrap` одним scratch; закоммитить пин. 3) `just feature-merge` (тесты,
затронутые всей веткой); ветку сохранить, не пушить. 4) Продолжить набор: промоутнуть
[B028](Backlog/028.md) на новой ветке `feature/...`.
