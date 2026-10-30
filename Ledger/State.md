# State

## Focus

Набор улучшений качества (кластеры 1–8), по одной карточке. Текущая —
[P038](Plan/038.md) ([B028](Backlog/028.md)): дисциплина ошибок и ресурсов. 3.1 запрет
тихого глотания ошибок, 3.2 запрет игнорирования `Result`/`#[must_use]`, 3.3 ресурсы через
контекст-менеджер/RAII. По выбору пользователя 2026-10-30 (вариант A): 3.1 — структурное
правило для Python `except: pass` (уезжает в init), Rust `let _ = <Result>` — на dev-гейте
через clippy (это п.3.2).

## Workspace

Ветка: feature/error-discipline (от master `9ca3be2`, куда влит P037). Коммитов на ветке
пока нет (VAC-A в рабочем дереве). stable.txt = `743a2bb` (голова P037, установлена).

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

B028 промоутнут в P038 (строка Plan + карточка `Plan/038.md`). VAC-A готов в рабочем дереве,
не закоммичен: в clippy-профиль (`.agentrig/commands.yaml`) добавлен `-D
clippy::let_underscore_must_use` (п.3.2 + Rust-часть 3.1 по варианту A); единственное место
`command_runner/jobs/process.rs` (teardown после сбоя attach) переведено с `let _ =` на
`.ok()`; записан D068. **3.3 уже обеспечен**: группа `SIM` (вкл. `SIM115`) уже в ruff-select —
изменений не требует. `unused_must_use` уже ловится через `-D warnings`.

## Verification

`cargo clippy` с новым денаем — чисто (0 находок после фикса process.rs). Полный гейт — на
коммите VAC-A.

## Blockers

Нет.

## Next action

1) Закоммитить VAC-A (промоушен + clippy + process.rs + D068) через хук. 2) VAC-B (3.1):
двухкоммитная схема для структурного правила `silent-error` (Python `except:`-тело из одного
`pass`, escape `DECISION:`/`INVARIANT:`): B1 enabling (Kind+дескриптор+Python-хендлер+
измерение+тест, не в `catalog::ALL`/lint.yaml, без маркеров) → пин+бустрап → B2 live
(`catalog::ALL`, lint.yaml, навык ремонта, assets, маркеры, догфуд, D069, State). 3) Архив
P038/B028, финальный пин (D051), `just feature-merge`. Не пушить, ветку сохранить.
