# State

## Focus

**P054 (B045) — закрыть четыре medium-находки повторного воркфлоу-ревью, активна.** AC1 (тела макросов),
AC2 (честный тест-контекст), AC3 (worker управляем сигналами после форвардинга) и AC4 (ограниченный дренаж
`execute`) — все реализованы. Остался Completion-VAC. Контракт — карточка [B045](Backlog/045.md).

## Workspace

Ветка `feature/medium-findings` (база master `125bd20`), наблюдаемый HEAD `740b85b` (VAC-C), сохраняется
после интеграции. `stable.txt` = `58e1041…` (не менялся; AC1–AC4 не вводят новый `kind:`, мид-бранч пин для
десериализации `lint.yaml` не нужен — только финальный пин D051). macOS; коммиты/cargo — через
`just scratch` с восстановленным PATH (`cargo`/`uv`), иначе гейт падает на `build: cargo not found`.

## Progress

VAC-A (`9a632df`): Backlog [B045](Backlog/045.md) → Plan [P054](Plan/054.md).
VAC-B (`3c2758f`) — **AC2**: честный тест-контекст, кластер в `structural_lint/languages/test_context.rs`.
VAC-C (`740b85b`) — **AC1**: `panic-path`/`eager-fallback`/`disabled-raii` видят паттерн внутри тел макросов;
решение [D094](Decisions/094.md); исправлен eager-fallback-баг `jobs/mod.rs`.
VAC-D (этот коммит) — **AC3+AC4**: в `command_runner/jobs/streams.rs` читатель ждёт готовность пайпа через
`libc::poll` и останавливается, когда общий флаг `exited` (ставится после `child.wait()`) истинен и данных
нет — `execute` возвращается, когда буфер осушён, не дожидаясь write-end, который держит неконтейнеризованный
detached-потомок, и не убивая его (AC4); после join форвардинга `execute` сбрасывает SIGINT/SIGTERM/SIGHUP в
`SIG_DFL` через `libc::signal`, т.к. `signal-hook` при drop оставляет свой OS-обработчик и иначе глотает
сигналы (AC3). Оба новых `unsafe` помечены `// DECISION: D095`, расширяющим D066; добавлены два поведенческих
теста. Решение [D095](Decisions/095.md).

## Verification

Кандидат-линт по всему репозиторию → EXIT=0. Два новых теста
(`test_foreground_drain_is_bounded_when_descendant_holds_pipe`,
`test_worker_stays_signal_controllable_after_child_exits`) проходят (0.54s). Эмпирически до фикса: worker
висит после выхода главного ребёнка и SIGTERM глотается (только SIGKILL); после фикса: worker возвращается
сам (code 0, AC4) и SIGTERM завершает его в дренаже (code −15, AC3). Полный гейт — на коммите VAC-D.

## Blockers

Нет.

## Next action

Completion-VAC: совместно проверить AC1–AC4; ротировать P054 → `Archive/Plan` и B045 → `Archive/Backlog` с
миграцией ссылок (в т.ч. `[card]` в D094 → `Archive/Backlog/045.md` и `[card]` в D095 → туда же); обновить
State; финальный пин D051 (`git rev-parse HEAD > tooling/distribution/stable.txt` + `just bootstrap` одним
коммитом) до `just feature-merge`; ветку сохранить, не пушить. Custom-allocator (B034) припаркован.
Свободные ID: D096+, P055+.
