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

VAC-A (`9a632df`): оформлен Backlog [B045](Backlog/045.md) и промоутнут в Plan [P054](Plan/054.md).
VAC-B (этот коммит) — **AC2 выполнен**: честный тест-контекст вместо подстроки `"test"`. Кластер
определения тест-онли кода (`in_test_context` и предикатный разбор `cfg`) вынесен из `rust.rs` в новый
сосед `structural_lint/languages/test_context.rs` (repair refactor-large-file под лимит 500 строк),
`rust.rs` зовёт его через `use super::test_context::in_test_context`. Негативная фикстура
`rust_test_context_exempts_only_real_test_code` в `languages/tests/markers.rs`. Дальше AC1/AC3/AC4.

## Verification

markers: 13/13 проходят, включая новую фикстуру (`cfg(not(test))`/`cfg(feature="latest")` → находка;
`cfg(all(unix,test))`/`cfg(test)`/`#[tokio::test]` → исключение). Догфуд `just candidate lint` → EXIT=0,
ноль находок (устранены предыдущие named-if-condition и file-size 513>500).

## Blockers

Нет.

## Next action

1. AC1 — детекторы `panic_call`/`eager_fallback`/`disabled_raii`/`*_macro` видят паттерн внутри тел
   макросов (`token_tree`); замерить FP `just candidate lint`; поправить/обосновать `jobs/mod.rs:263`
   (`unwrap_or(now()?)` в `json!`); негативная фикстура.
2. Затем AC3 (восстановление сигналов после `execute()` в `command_runner/jobs/process.rs`), AC4
   (ограниченный дренаж `streams.rs`/`process.rs`). Completion-VAC: ротация P054/B045, финальный пин D051,
   `just feature-merge`. Custom-allocator (B034) припаркован. Свободные ID: D094+, P055+.
