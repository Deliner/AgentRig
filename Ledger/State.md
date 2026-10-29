# State

## Focus

Реализация набора улучшений качества (кластеры 1–8, [B026](Backlog/026.md)–
[B033](Backlog/033.md)) через догфудинг, по одной карточке. Текущая — [B029](Backlog/029.md)
([P036](Plan/036.md)): синтаксические линты сложности. Делается по под-правилам: 4.1
глубина вложенности — в работе; 4.2 запрет булевых флаг-параметров и 4.3 бюджет ветвлений —
дальше.

## Workspace

Branch: feature/complexity-lints (от master). Revision: A1 (enabling) закоммичен — `e4bb96a`;
A2 (live) в рабочем дереве, не закоммичен. Установленный бинарь забустрапен на `e4bb96a`
(знает `NestingDepth`), поэтому `lint.yaml` с `nesting-depth` не клинит хук. Коммитнутый пин
`stable.txt` всё ещё `ace5511` — финальный пин (голова A2) ставится перед merge (D051).

Машина — macOS. Сборка/clippy/коммит/мерж — через `just scratch -- '...'` с PATH; `cargo`
в `~/.zshenv` (`~/.cargo/bin`).

Правило 4.1 требует двух коммитов с бустрапом между ними (установленный бинарь валидирует
`.agentrig/lint.yaml` и клинит на незнакомом `kind:`): **A1 enabling** — вариант `Kind`,
дескриптор, обработчики, измерение, навык и юнит-тесты, но БЕЗ `catalog::ALL` и БЕЗ правила
в `lint.yaml` (I020 сбалансирован); затем пин+бустрап; **A2 live** — `NestingDepth` в `ALL`
+ правило `nesting-depth` (`error: 4`) в `lint.yaml` + догфуд нашего кода.

## Progress

A1 (enabling) закоммичен `e4bb96a` — весь гейт зелёный, забустрапен:

- Движок: `Kind::NestingDepth` (`kind.rs`), дескриптор `NESTINGDEPTH` (`error: 4`,
  навык `flatten-nesting`), в обоих обработчиках `rules` (`registry.rs`); измерение
  `nesting`/`descend` + enum `Nest` (`languages/mod.rs`); `classify` в `rust.rs`/`python.rs`
  (if/match/loop/while/for → +1; Rust else-if и Python elif — тот же уровень; вложенные
  функции/замыкания — границы, меряются отдельно). Навык `flatten-nesting` (обе стороны
  I020). Юнит-тесты `rust_nesting_...`/`python_nesting_...`. Ledger: [P036](Plan/036.md).

A2 (live) в рабочем дереве, не закоммичен:

- `NestingDepth` добавлен в `catalog::ALL`; правило `nesting-depth` (`error: 4`) в
  `.agentrig/lint.yaml`. Решение [D061](Decisions/061.md) записано (строка + карточка),
  маркеры `// DECISION: D061` на владельцах (`descriptor.rs`, `languages/mod.rs`,
  `rust.rs`, `python.rs`, `languages/tests/mod.rs`, `lint.yaml`) — зеркало D037.
- Догфуд: наш код уже зелёный под порогом `error: 4` (глубина ≤4 везде), рефактор не нужен;
  порог не ослаблялся (совпадает с `parameter-count`).

## Verification

A1-коммит: build/lint/memory/rustfmt/clippy/rust-test 56/ruff/ruff-format/mypy/test 264/
typos/vulture — PASS; бустрап `e4bb96a` успешен. A2 (рабочее дерево): `config-check` валиден,
кандидат-`lint` = 0 ошибок (маркеры D061 принимаются, nesting без находок). Полный A2-гейт
ещё не прогонялся.

## Blockers

Нет.

## Next action

Застейджить A2 (без `stable.txt`) и закоммитить через хук. Затем финальный пин: голова A2 →
`stable.txt` + `just bootstrap` одной командой scratch, закоммитить пин (D051). Затем
`just feature-merge` (ветка сохраняется). Потом — следующая карточка набора (B026, B028,
B030–B033) и под-правила B029 4.2/4.3. Не пушить без просьбы.
