# State

## Focus

**P054 (B045) — закрыть четыре medium-находки повторного воркфлоу-ревью, активна.** AC1 — правила
`panic-path`/`eager-fallback-with-?`/`disabled-raii` видят паттерн внутри тел макросов (сегодня слепы:
детекторы на `call_expression`, тело макроса парсится как `token_tree`); AC2 — честный тест-контекст
вместо подстроки `"test"` в тексте атрибута; AC3 — worker управляем сигналами между/после выполнений
команд; AC4 — ограниченный по времени дренаж `execute()`. Контракт — карточка [B045](Backlog/045.md).

## Workspace

Ветка `feature/medium-findings` (база master `125bd20`), сохраняется после интеграции. `stable.txt` =
`58e1041…` (ещё не менялся; AC1/AC2 новых `kind:` не вводят, поэтому мид-бранч пин для десериализации
lint.yaml не нужен — только финальный пин D051). macOS; коммиты — `just scratch` с восстановленным PATH
(`cargo`/`uv`), иначе гейт падает на `build: cargo not found`.

## Progress

VAC-A (`9a632df`): Backlog [B045](Backlog/045.md) промоутнут в Plan [P054](Plan/054.md).
VAC-B (`3c2758f`) — **AC2**: честный тест-контекст, кластер `cfg`-разбора вынесен в
`structural_lint/languages/test_context.rs`.
VAC-C (этот коммит) — **AC1 выполнен**: `panic-path`/`eager-fallback`/`disabled-raii` видят паттерн
внутри тел макросов. Новый `macro_body` (`rust.rs`) на диспетче `macro_invocation` переразбирает
содержимое `token_tree` как список аргументов (`fn probe(){call(<inner>)}`), и только при чистом парсе
гоняет те три детектора по поддереву, сдвигая строки на стартовую строку `token_tree`. Тест-контекст и
обоснование (комментарий внутри макроса) соблюдаются; невалидный Rust (DSL) пропускается. Записано
решение [D094](Decisions/094.md). Исправлен реальный баг `command_runner/jobs/mod.rs` (ленивый `now()?`
через `match`). Дальше AC3/AC4.

## Verification

markers: 15/15 (+`rust_flags_eager_fallback_inside_macro_bodies`, `rust_flags_panic_paths_inside_macro_bodies`:
json!/format!/println! → находка; обоснование внутри макроса и `#[cfg(test)]` → исключение; `html!` DSL →
пропуск). Догфуд `just candidate lint` по всему репозиторию → ровно одна находка (`jobs/mod.rs:261`,
реальный eager-fallback-баг) до фикса, после фикса EXIT=0. FP не обнаружены — сужение AC1 не потребовалось.

## Blockers

Нет.

## Next action

1. AC3 — worker управляем сигналами после `execute()`: восстановить обработчики (SIG_DFL/эмуляция) в
   `command_runner/jobs/process.rs` (~стр. 30); поведенческий тест worker.
2. AC4 — ограниченный по времени дренаж/закрытие пайпов после терминации группы (`streams.rs`/`process.rs`);
   поведенческий тест. Completion-VAC: верификация AC1–AC4, ротация P054/B045 в Archive (с миграцией
   ссылок, в т.ч. [card] в D094 на `Archive/Backlog/045.md`), финальный пин D051 + `just bootstrap` одним
   коммитом до `just feature-merge`, ветку сохранить, не пушить. Custom-allocator (B034) припаркован.
   Свободные ID: D095+, P055+.
