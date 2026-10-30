# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [B029](Backlog/029.md)
([P036](Plan/036.md)): синтаксические линты сложности, по под-правилам. 4.1 глубина
вложенности и 4.2 запрет булевых параметров — **сделаны и влиты в master**. Сейчас — 4.3
бюджет ветвлений на функцию. По решению пользователя `match`/`case` считается как ОДНО
многоходовое ветвление (не по ветвям), наряду с каждым `if`/`else-if`/тернарником. Это
последнее под-правило, оно закрывает P036.

## Workspace

Ветка: feature/branch-budget (от master `88c5e48`). Revision: `239c9f5` (A1, HEAD).
Трёхкоммитная последовательность из-за того, что гейт `[lint]` гоняет УСТАНОВЛЕННЫЙ бинарь:
A1 (enabling, arms-метрика) закоммичен и забустрапен → A1′ (коррекция на метрику «match = 1»)
→ бустрап A1′ → A2 (live: правило в lint.yaml + маркеры + catalog::ALL + догфуд + D063).

Машина — macOS. Сборка/clippy/fmt/тесты/коммит/мерж/бустрап — через `just scratch -- '...'` с
PATH (`cargo` в `~/.cargo/bin`; для гейта с Python-тестами venv `tooling/.venv/bin` первым).

## Progress

B029 4.1 (nesting-depth, [D061](Decisions/061.md)) и 4.2 (boolean-parameter,
[D062](Decisions/062.md)) влиты в master (merge `88c5e48`).

B029 4.3 (branch-budget):
- A1 `239c9f5` закоммичен и забустрапен (установленный бинарь знает `kind: branch-budget`,
  но по СТАРОЙ метрике «match = число ветвей»; правила в lint.yaml и в `catalog::ALL` нет).
- A1′ готов в рабочем дереве (НЕ закоммичен): измерение переписано на «match/case = 1»
  (`Branch{Boundary,Decision,Plain}` + `accumulate`; классификаторы `branch` в rust.rs/python.rs
  возвращают `Decision` для if/else-if/тернарника/match/case; helper'ы `arms()`/`cases()`
  удалены), строка `metric` дескриптора и навык `reduce-branching` приведены к этой метрике,
  юнит-тесты обновлены (8 passed), порог дескриптора выставлен `error: 5`. БЕЗ lint.yaml,
  `catalog::ALL`, маркеров `D063`, догфуда — это A2.

Порог выбран по обзору распределения (метрика «match = 1»): плотный кластер 4 (21 функция)
и 5 (5), тонкий хвост 6 (3) и 7 (1). `error: 5` (находка при >5) бьёт по хвосту — 4 функции
≥6: `outer_control`(7, agent_hooks/guard.rs), `run_output`(6, structural_lint/mod.rs),
`arrange`(6, command_runner/jobs/process.rs), питон-тест `test_local_crates_cannot_hide_incomplete_analysis`(6).

## Verification

A1′ локально: юнит-тесты `structural_lint` ветвлений (Rust+Python) проходили до компакта
(8 passed); правка порога 10→5 на тесты не влияет. Гейт на A1′ ещё не прогонялся.

## Blockers

Нет.

## Next action

Застейджить A1′ (6 файлов движка/навыка/тестов + этот State; БЕЗ stable.txt) и закоммитить
через pre-commit. Затем пин: head A1′ в `tooling/distribution/stable.txt` + `just bootstrap`
**одним** scratch (не отдельным Edit — иначе launcher-хук заклинит). Затем A2: правило
`branch-budget` (`error: 5`, `# DECISION: D063`) в lint.yaml, `Kind::BranchBudget` в
`catalog::ALL`, маркеры `// DECISION: D063` на владельцах движка, догфуд 4 функций ≥6, запись
D063, State; коммит A2 через гейт; финальный пин+бустрап, коммит пина, `just feature-merge`.
Не пушить, ветку сохранить.
