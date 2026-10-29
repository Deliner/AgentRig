# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [B029](Backlog/029.md)
([P036](Plan/036.md)): синтаксические линты сложности. Делается по под-правилам: 4.1
глубина вложенности — в работе; 4.2 запрет булевых флаг-параметров и 4.3 бюджет ветвлений —
дальше.

## Workspace

Branch: feature/complexity-lints (от master). Revision: `a885bf2` (база ветки; VAC по
ветке ещё не коммичены). Пин `tooling/distribution/stable.txt` = `ace5511` (прошлая карта).

Машина — macOS. Сборка/clippy/коммит/мерж — через `just scratch -- '...'` с PATH; `cargo`
в `~/.zshenv` (`~/.cargo/bin`).

Правило 4.1 требует двух коммитов с бустрапом между ними (установленный бинарь валидирует
`.agentrig/lint.yaml` и клинит на незнакомом `kind:`): **A1 enabling** — вариант `Kind`,
дескриптор, обработчики, измерение, навык и юнит-тесты, но БЕЗ `catalog::ALL` и БЕЗ правила
в `lint.yaml` (I020 сбалансирован); затем пин+бустрап; **A2 live** — `NestingDepth` в `ALL`
+ правило `nesting-depth` (`error: 4`) в `lint.yaml` + догфуд нашего кода.

## Progress

A1 (nesting-depth) готов в рабочем дереве, не закоммичен:

- Движок: `Kind::NestingDepth` (`kind.rs`), дескриптор `NESTINGDEPTH` (`error: 4`,
  навык `flatten-nesting`), в обоих обработчиках `rules` (`registry.rs`); измерение
  `nesting`/`descend` + enum `Nest` (`languages/mod.rs`); `classify` в `rust.rs`/`python.rs`
  (if/match/loop/while/for → +1; Rust else-if и Python elif — тот же уровень; вложенные
  функции/замыкания — границы, меряются отдельно).
- Навык `flatten-nesting`: `SKILL.md` + `architecture.yaml`, в `assets.rs` (`SKILLS`) и в
  `Project/shared/skills/architecture.yaml` — обе стороны синхронно (I020).
- Юнит-тесты движка: `rust_nesting_...`, `python_nesting_...` в `languages/tests/mod.rs`.
- Ledger: создан [P036](Plan/036.md), строка в [Plan](Plan.md); [B029](Backlog/029.md) выбрана.
- НЕ сделано в A1 осознанно: `catalog::ALL` и правило в `.agentrig/lint.yaml` (это A2).

## Verification

Кандидат собран (`agentrig 0.3.0`). Rust lib-тесты: 79 PASS (включая `rust_nesting_...`,
`python_nesting_...`). Python: test_delivery (I020), test_lint_defaults (I017),
test_capabilities, test_language_delivery, test_language — 40 PASS. I020 сбалансирован.
Полный коммит-гейт ещё не прогонялся.

## Blockers

Нет.

## Next action

Застейджить только файлы A1, закоммитить A1 через хук (гейт прогонит affected-тесты), затем
пин+бустрап одной командой scratch (`stable.txt` = голова A1 + `just bootstrap`, D051). Затем
A2: добавить `NestingDepth` в `catalog::ALL` и правило `nesting-depth` в `lint.yaml`, догфуд
нашего кода к зелёному, обновить State. Не пушить без просьбы.
