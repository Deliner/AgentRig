# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [B029](Backlog/029.md)
([P036](Plan/036.md)): синтаксические линты сложности, по под-правилам. 4.1 глубина
вложенности и 4.2 запрет булевых параметров — **сделаны и влиты в master**. Сейчас — 4.3
бюджет ветвлений на функцию (число if/elif/тернарников + ветвей match/case). Это последнее
под-правило, оно закрывает P036.

## Workspace

Ветка: feature/branch-budget (от master `88c5e48`). Двухкоммитная схема (установленный бинарь
жадно валидирует lint.yaml и падает на незнакомом `kind`, поэтому правило уезжает в lint.yaml
только после бустрапа A1). A1 (enabling) готов в рабочем дереве, ещё НЕ закоммичен.

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

B029 4.1 (nesting-depth, [D061](Decisions/061.md)) и 4.2 (boolean-parameter,
[D062](Decisions/062.md)) влиты в master (merge `88c5e48`).

B029 4.3 (branch-budget) — A1 готов (не закоммичен): `Kind::BranchBudget`, дескриптор
(`BRANCHBUDGET`, Thresholds), измерение `branching`/`accumulate` + классификаторы `branch`
в rust.rs/python.rs (if/else-if/тернарник = 1, match/case = число ветвей; вложенные
функции/замыкания — отдельно), оба хендлера в registry, ремонтный навык `reduce-branching`
в НОВОЙ группе `control-flow/` (туда же перенесён `flatten-nesting`; плоские симлинки
сохраняют пути `.agents/skills/...`), юнит-тесты Rust+Python. БЕЗ: `catalog::ALL`, правила в
lint.yaml, маркеров `D063`, догфуда — всё это A2 после бустрапа A1. Порог: дескриптор пока
10; окончательный выбирается в A2 по обзору распределения (обзор возможен только после
бустрапа A1 — установленный бинарь иначе отвергает `kind: branch-budget`).

## Verification

A1 локально зелёный: `cargo fmt` применён; `clippy` (strict deny-set) чисто; юнит-тесты
`structural_lint` **49 passed** (включая 2 новых — счёт ветвлений Rust и Python);
`just config-check` валиден. Гейт установленным бинарём ещё не прогонялся (будет в pre-commit
коммита A1).

## Blockers

Нет.

## Next action

Застейджить A1 (движок, навык, control-flow-группа, тесты) и этот State; коммит A1 через
pre-commit хук. Затем пин: записать head A1 в `tooling/distribution/stable.txt` + `just
bootstrap` **одним** scratch (не отдельным Edit — иначе launcher-хук заклинит), коммит пина.
Затем A2: добавить `catalog::ALL` + правило в lint.yaml, обзор распределения через `just
candidate lint`, выбрать порог, догфуд-правки, маркеры `// DECISION: D063`, запись D063,
State; коммит A2; финальный пин+бустрап, коммит пина, `just feature-merge`. Не пушить, ветку
сохранить.
