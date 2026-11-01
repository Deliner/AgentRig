# State

## Focus

**P054 (B045) — закрыть четыре medium-находки повторного воркфлоу-ревью — ВЫПОЛНЕН и ротирован.** AC1
(тела макросов), AC2 (честный тест-контекст), AC3 (worker управляем сигналами после форвардинга) и AC4
(ограниченный дренаж `execute`) доставлены и зафиксированы. Карточка и источник — в
[Archive/Plan/054.md](Archive/Plan/054.md) и [Archive/Backlog/045.md](Archive/Backlog/045.md). Plan пуст.

## Workspace

Ветка `feature/medium-findings` (база master `125bd20`), наблюдаемый HEAD `3f64de5` (VAC-D) до
Completion-VAC; сохраняется после интеграции. `stable.txt` = `58e1041…` до финального пина (AC1–AC4 не
вводят новый `kind:`, мид-бранч пин не требовался). macOS;
коммиты/cargo — через `just scratch` с восстановленным PATH (`cargo`/`uv`), иначе гейт падает на
`build: cargo not found`.

## Progress

VAC-A (`9a632df`): Backlog [B045](Archive/Backlog/045.md) → Plan P054.
VAC-B (`3c2758f`) — **AC2**: честный тест-контекст, `structural_lint/languages/test_context.rs`.
VAC-C (`740b85b`) — **AC1**: `panic-path`/`eager-fallback`/`disabled-raii` видят паттерн в телах макросов;
[D094](Decisions/094.md); исправлен eager-fallback-баг `jobs/mod.rs`.
VAC-D (`3f64de5`) — **AC3+AC4**: `streams.rs` дренаж через `libc::poll` + флаг `exited`; `process.rs`
сбрасывает SIGINT/SIGTERM/SIGHUP в `SIG_DFL` после join форвардинга; [D095](Decisions/095.md) (расширяет
D066); два поведенческих теста.
Completion-VAC (этот коммит): P054 → `Archive/Plan`, B045 → `Archive/Backlog`; ссылки card в D094/D095
переведены на `Archive/Backlog/045.md`; индексы Plan/Backlog/Archive обновлены; Delivery P054 заполнен
по AC.

## Verification

Кандидат-линт по всему репозиторию → EXIT=0 (на VAC-C/VAC-D). Два новых теста
(`test_foreground_drain_is_bounded_when_descendant_holds_pipe`,
`test_worker_stays_signal_controllable_after_child_exits`) проходят. Полный гейт зелёный на VAC-D
(`3f64de5`, test 207 passed, вкл. оба новых теста). Completion-VAC — память-онли, проходит через гейт
(exported-index + затронутые тест-группы + smoke).

## Blockers

Нет.

## Next action

Финальный пин D051: `git rev-parse HEAD > tooling/distribution/stable.txt` + `just bootstrap` одним
`just scratch`-коммитом (голова ветки после Completion-VAC), коммит пина через гейт до
`just feature-merge`. Затем `just feature-merge` (ребейз на master, проверка интегрированного кандидата,
merge без уплощения); ветку сохранить, не пушить. Custom-allocator (B034) припаркован. Свободные ID:
D096+, P055+.
