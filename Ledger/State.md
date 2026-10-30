# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [B029](Backlog/029.md)
([P036](Plan/036.md)): синтаксические линты сложности, по под-правилам. 4.1 глубина
вложенности — **сделана и влита в master**. Сейчас — 4.2 запрет булевых/флаговых
параметров. Дальше: 4.3 бюджет ветвлений.

## Workspace

Ветка: feature/boolean-parameters (от master `7c80a43`). Двухкоммитная схема. A1 (enabling)
влит — `306ab9c` — и забустрапен (установленный бинарь знает `kind: boolean-parameter`). A2
(live) полностью готов в рабочем дереве: `catalog::ALL`, правило в `.agentrig/lint.yaml`,
33 догфуд-конверсии, запись решения D062. `stable.txt` пока = `306ab9c` (перепишется на head
A2 при финальном пине).

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.zshenv`). Python тестов — `./tooling/.venv/bin/python3 -m pytest`.

## Progress

B029 4.1 (nesting-depth) **влита в master**: A1 `e4bb96a` → A2 `e8553a8` → пин `341425a` →
merge `7c80a43`. Решение [D061](Decisions/061.md).

B029 4.2 (boolean-parameter): A2-исходники готовы. Все 33 находки убраны — полный бан
([D062](Decisions/062.md), выбор пользователя «чиню все 33»): 24 Rust-параметра → именованные
mode-enum'ы (`Capture`/`Access`/`Grouping`/`Stop`/`Sink`/`Channel`/`Tree`/`Index`/`Mode`/
`Format`/`Exclusion`/`Entries`/`Reminding`/`Enforcement`), 9 Python — строковые метки в
parametrize (`edition`/`vcs`/`setting`+`code`/`outcome`, хелпер `git` → `expect=`). Конверсии
вскрыли и устранены 19 находок `named-if-condition` (именованные локали / `match`) и один
цикл каталогов `architecture` (`Capture` перенесён в `command_runner::jobs` к владельцу-
исполнителю). Пороги/селекторы не ослаблялись.

## Verification

Кандидат зелёный: `just candidate lint --root .` — **0 находок** (boolean-parameter, named-if-
condition, architecture и остальные); `clippy` (strict deny-set) чисто; `rustfmt` применён;
нативные тесты **81 passed**; затронутые pytest-файлы **75 passed**; `memory-check` exit 0
(D062 записан, маркеры сверены). Гейт установленным A1-бинарём ещё не прогонялся (будет в
pre-commit).

## Blockers

Нет.

## Next action

Застейджить исходники + Ledger (State, Decisions, Decisions/062.md) **без** `stable.txt`;
коммит A2 через pre-commit хук (не обходить). Затем финальный пин: записать head A2 в
`tooling/distribution/stable.txt` + бустрап одним scratch, коммит пина, `feature-merge` через
`tooling/worker/run feature-merge`. Не пушить, ветку сохранить. Затем 4.3 (бюджет ветвлений).
